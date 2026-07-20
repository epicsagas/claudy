use rusqlite::{Connection, params};
use std::sync::{Mutex, MutexGuard};

mod analytics_queries;
mod migrations;
mod pricing_repo;
mod project_repo;
mod session_repo;

pub(crate) const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS migration_version (
version INTEGER PRIMARY KEY
);INSERT OR IGNORE INTO migration_version (version) VALUES (0);

CREATE TABLE IF NOT EXISTS projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    encoded_dir     TEXT    NOT NULL UNIQUE,
    display_name    TEXT    NOT NULL,
    resolved_path   TEXT,
    first_seen_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    last_seen_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_projects_display ON projects(display_name);

CREATE TABLE IF NOT EXISTS sessions (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    session_uuid      TEXT    NOT NULL UNIQUE,
    project_id        INTEGER NOT NULL REFERENCES projects(id),
    cwd               TEXT,
    model             TEXT,
    first_message     TEXT,
    started_at        TEXT,
    ended_at          TEXT,
    total_turns       INTEGER DEFAULT 0,
    total_cost_usd    REAL    DEFAULT 0.0,
    total_duration_ms INTEGER DEFAULT 0,
    source_file       TEXT    NOT NULL,
    file_modified_at  TEXT,
    ingested_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    source_kind       TEXT
);
CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project_id);
CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_sessions_uuid ON sessions(session_uuid);

CREATE TABLE IF NOT EXISTS turns (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      INTEGER NOT NULL REFERENCES sessions(id),
    turn_number     INTEGER NOT NULL,
    prompt_text     TEXT,
    response_text   TEXT,
    model           TEXT,
    duration_ms     INTEGER,
    started_at      TEXT,
    ingested_at     TEXT    NOT NULL DEFAULT (datetime('now')),
    human_authored  INTEGER DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_turns_session ON turns(session_id);
CREATE INDEX IF NOT EXISTS idx_turns_model ON turns(model);

CREATE TABLE IF NOT EXISTS token_usage (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    turn_id                     INTEGER NOT NULL REFERENCES turns(id),
    model                       TEXT    NOT NULL,
    input_tokens                INTEGER NOT NULL DEFAULT 0,
    output_tokens               INTEGER NOT NULL DEFAULT 0,
    cache_creation_input_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_input_tokens     INTEGER NOT NULL DEFAULT 0,
    estimated_cost_usd          REAL    DEFAULT 0.0,
    ingested_at                 TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_token_usage_turn ON token_usage(turn_id);
CREATE INDEX IF NOT EXISTS idx_token_usage_model ON token_usage(model);

CREATE TABLE IF NOT EXISTS tool_calls (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    turn_id         INTEGER NOT NULL REFERENCES turns(id),
    tool_use_id     TEXT    NOT NULL,
    tool_name       TEXT    NOT NULL,
    input_summary   TEXT,
    is_error        INTEGER DEFAULT 0,
    result_summary  TEXT,
    duration_ms     INTEGER,
    ingested_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_tool_calls_turn ON tool_calls(turn_id);
CREATE INDEX IF NOT EXISTS idx_tool_calls_name ON tool_calls(tool_name);
CREATE INDEX IF NOT EXISTS idx_tool_calls_use_id ON tool_calls(tool_use_id);

CREATE TABLE IF NOT EXISTS channel_metrics (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id          INTEGER REFERENCES sessions(id),
    platform            TEXT    NOT NULL,
    channel_id          TEXT    NOT NULL,
    user_id             TEXT    NOT NULL,
    profile             TEXT,
    stream_duration_ms  INTEGER,
    first_byte_ms       INTEGER,
    stream_timeout      INTEGER DEFAULT 0,
    error_type          TEXT,
    ingested_at         TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS recommendations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    category    TEXT    NOT NULL,
    severity    TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    description TEXT    NOT NULL,
    action      TEXT,
    computed_at TEXT    NOT NULL DEFAULT (datetime('now')),
    dismissed_at TEXT
);

CREATE TABLE IF NOT EXISTS ingestion_checkpoints (
    file_path       TEXT PRIMARY KEY,
    file_modified   TEXT NOT NULL,
    byte_offset     INTEGER NOT NULL DEFAULT 0,
    line_count      INTEGER NOT NULL DEFAULT 0,
    ingested_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS model_pricing (
    model_id    TEXT    PRIMARY KEY,
    input       REAL    NOT NULL,
    output      REAL    NOT NULL,
    cache_write REAL    NOT NULL,
    cache_read  REAL    NOT NULL,
    source      TEXT    NOT NULL,
    synced_at   TEXT    NOT NULL
);
";

pub struct SqliteAnalyticsStore {
    pub(crate) conn: Mutex<Connection>,
}

impl SqliteAnalyticsStore {
    pub fn open(db_path: &str) -> anyhow::Result<Self> {
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000; PRAGMA foreign_keys=ON;",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub(crate) fn lock(&self) -> anyhow::Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| anyhow::anyhow!("db lock poisoned: {}", e))
    }

    /// Upsert all pricing rows inside a single transaction.
    pub fn batch_upsert_model_pricing_impl(
        &self,
        pricings: &[crate::domain::analytics::ModelPricing],
    ) -> anyhow::Result<()> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        for pricing in pricings {
            tx.execute(
                "INSERT INTO model_pricing (model_id, input, output, cache_write, cache_read, source, synced_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(model_id) DO UPDATE SET
                   input       = excluded.input,
                   output      = excluded.output,
                   cache_write = excluded.cache_write,
                   cache_read  = excluded.cache_read,
                   source      = excluded.source,
                   synced_at   = excluded.synced_at",
                params![
                    pricing.model_id,
                    pricing.input,
                    pricing.output,
                    pricing.cache_write,
                    pricing.cache_read,
                    pricing.source,
                    pricing.synced_at,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::analytics::*;
    use crate::ports::analytics_ports::{AnalyticsStore, PricingStore};
    use tempfile::tempdir;

    fn test_store() -> SqliteAnalyticsStore {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("test.db");
        let store = SqliteAnalyticsStore::open(db_path.to_str().expect("path")).expect("open");
        store.initialize_schema().expect("schema");
        store
    }

    #[test]
    fn test_upsert_project() {
        let store = test_store();
        let id = store
            .upsert_project(
                "-home-user-projects-claudy",
                "claudy",
                Some("/home/user/projects/claudy"),
            )
            .unwrap();
        assert!(id > 0);
        let found = store
            .get_project_by_encoded_dir("-home-user-projects-claudy")
            .unwrap()
            .expect("found");
        assert_eq!(found.display_name, "claudy");
    }

    #[test]
    fn test_session_lifecycle() {
        let store = test_store();
        let pid = store.upsert_project("-test-proj", "test", None).unwrap();
        let sid = store
            .upsert_session(&NewSession {
                session_uuid: "uuid-123".into(),
                project_id: pid,
                source_file: "/test/a.jsonl".into(),
                cwd: Some("/test".into()),
                model: Some("claude-sonnet".into()),
                first_message: Some("hello".into()),
                started_at: Some("2026-01-01T00:00:00".into()),
                source_kind: None,
            })
            .unwrap();
        assert!(sid > 0);
        store
            .update_session_completion(sid, "2026-01-01T01:00:00", 3, 0.05, 3_600_000)
            .unwrap();
        let s = store
            .get_session_by_uuid("uuid-123")
            .unwrap()
            .expect("found");
        assert_eq!(s.total_turns, 3);
        assert!((s.total_cost_usd - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_insert_turn_and_token_usage() {
        let store = test_store();
        let pid = store.upsert_project("-test", "test", None).unwrap();
        let sid = store
            .upsert_session(&NewSession {
                session_uuid: "uuid-t".into(),
                project_id: pid,
                source_file: "/t.jsonl".into(),
                cwd: None,
                model: None,
                first_message: None,
                started_at: None,
                source_kind: None,
            })
            .unwrap();
        let tid = store
            .insert_turn(&NewTurn {
                session_id: sid,
                turn_number: 1,
                prompt_text: Some("prompt".into()),
                response_text: Some("response".into()),
                model: Some("sonnet".into()),
                duration_ms: Some(3_600_000),
                started_at: Some("2026-01-01".into()),
                human_authored: true,
            })
            .unwrap();
        store
            .insert_token_usage(&NewTokenUsage {
                turn_id: tid,
                model: "claude-sonnet-4-6".into(),
                input_tokens: 500,
                output_tokens: 200,
                cache_creation_input_tokens: 100,
                cache_read_input_tokens: 400,
                estimated_cost_usd: 0.01,
            })
            .unwrap();
        let turns = store.get_turns_by_session(sid).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].prompt_text.as_deref(), Some("prompt"));
    }

    #[test]
    fn test_tool_calls() {
        let store = test_store();
        let pid = store.upsert_project("-test", "test", None).unwrap();
        let sid = store
            .upsert_session(&NewSession {
                session_uuid: "uuid-tc".into(),
                project_id: pid,
                source_file: "/tc.jsonl".into(),
                cwd: None,
                model: None,
                first_message: None,
                started_at: None,
                source_kind: None,
            })
            .unwrap();
        let tid = store
            .insert_turn(&NewTurn {
                session_id: sid,
                turn_number: 1,
                prompt_text: None,
                response_text: None,
                model: None,
                duration_ms: None,
                started_at: None,
                human_authored: true,
            })
            .unwrap();
        store
            .insert_tool_call(&NewToolCall {
                turn_id: tid,
                tool_use_id: "tu-1".into(),
                tool_name: "Read".into(),
                input_summary: Some("file.rs".into()),
                is_error: false,
                result_summary: Some("content".into()),
                duration_ms: Some(50),
            })
            .unwrap();
        store
            .insert_tool_call(&NewToolCall {
                turn_id: tid,
                tool_use_id: "tu-2".into(),
                tool_name: "Edit".into(),
                input_summary: Some("file.rs".into()),
                is_error: true,
                result_summary: Some("error".into()),
                duration_ms: None,
            })
            .unwrap();
        let calls = store.get_tool_calls_by_turn(tid).unwrap();
        assert_eq!(calls.len(), 2);
        assert!(calls[1].is_error);
    }

    #[test]
    fn test_checkpoints() {
        let store = test_store();
        assert!(store.get_checkpoint("/foo.jsonl").unwrap().is_none());
        store
            .upsert_checkpoint("/foo.jsonl", "2026-01-01", 100, 5)
            .unwrap();
        let cp = store.get_checkpoint("/foo.jsonl").unwrap().expect("found");
        assert_eq!(cp.byte_offset, 100);
        store
            .upsert_checkpoint("/foo.jsonl", "2026-01-02", 200, 10)
            .unwrap();
        let cp = store.get_checkpoint("/foo.jsonl").unwrap().expect("found");
        assert_eq!(cp.byte_offset, 200);
    }

    #[test]
    fn test_recommendations() {
        let store = test_store();
        store
            .insert_recommendation(&Recommendation {
                category: RecommendationCategory::CostOptimization,
                severity: Severity::Warning,
                title: "High cost".into(),
                description: "Cost is increasing".into(),
                action: Some("Switch model".into()),
            })
            .unwrap();
        let recs = store.get_recommendations().unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].title, "High cost");
        store.clear_recommendations().unwrap();
        assert!(store.get_recommendations().unwrap().is_empty());
    }

    #[test]
    fn test_batch_upsert_model_pricing_impl_is_callable() {
        // Verify the renamed inherent method exists and works correctly.
        let store = test_store();
        let pricings = vec![crate::domain::analytics::ModelPricing {
            model_id: "claude-impl-test".into(),
            input: 3.0,
            output: 15.0,
            cache_write: 3.75,
            cache_read: 0.30,
            source: "test".into(),
            synced_at: "2026-01-01T00:00:00Z".into(),
        }];
        // Call the renamed inherent method directly.
        store.batch_upsert_model_pricing_impl(&pricings).unwrap();
        let fetched = store
            .get_model_pricing("claude-impl-test")
            .unwrap()
            .expect("found");
        assert!((fetched.input - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_aggregate_cost_metrics_uses_db_cache_savings() {
        use crate::ports::analytics_ports::PricingStore as _;
        let store = test_store();

        // Insert a model pricing row with exaggerated rates so the DB path produces
        // a detectably different result from the hardcoded fallback.
        // savings = cache_read_tokens * (input_rate - cache_read_rate) / 1_000_000
        //         = 1_000_000 * (10.0 - 1.0) / 1_000_000 = $9.0
        store
            .upsert_model_pricing(&crate::domain::analytics::ModelPricing {
                model_id: "claude-haiku-test".into(),
                input: 10.0,
                output: 30.0,
                cache_write: 5.0,
                cache_read: 1.0,
                source: "test".into(),
                synced_at: "2026-01-01T00:00:00Z".into(),
            })
            .unwrap();

        let pid = store
            .upsert_project("-test-cost", "cost-proj", None)
            .unwrap();
        let sid = store
            .upsert_session(&NewSession {
                session_uuid: "uuid-cost-db".into(),
                project_id: pid,
                source_file: "/cost.jsonl".into(),
                cwd: None,
                model: Some("claude-haiku-test".into()),
                first_message: None,
                started_at: Some("2026-01-01T00:00:00".into()),
                source_kind: None,
            })
            .unwrap();
        store
            .update_session_completion(sid, "2026-01-01T01:00:00", 1, 0.10, 60_000)
            .unwrap();
        let tid = store
            .insert_turn(&NewTurn {
                session_id: sid,
                turn_number: 1,
                prompt_text: None,
                response_text: None,
                model: Some("claude-haiku-test".into()),
                duration_ms: None,
                started_at: Some("2026-01-01T00:00:00".into()),
                human_authored: true,
            })
            .unwrap();
        store
            .insert_token_usage(&NewTokenUsage {
                turn_id: tid,
                model: "claude-haiku-test".into(),
                input_tokens: 1000,
                output_tokens: 500,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 1_000_000,
                estimated_cost_usd: 0.10,
            })
            .unwrap();

        // Use a large window so the 2026-01-01 session is always included.
        let metrics = store.aggregate_cost_metrics(9999, None).unwrap();

        // DB path: $9.0. Hardcoded fallback for an unknown model would be $0.0
        // (get_pricing returns defaults with equal input/cache_read for unknown models).
        assert!(
            (metrics.cache_savings_usd - 9.0).abs() < 0.001,
            "expected $9.0 cache savings from DB rates, got {}",
            metrics.cache_savings_usd
        );
    }

    // Seed a session with turns/tokens/tool_calls for the four aggregations.
    fn seed_for_aggregations(store: &SqliteAnalyticsStore) -> i64 {
        let pid = store
            .upsert_project("-t-proj", "tproj", Some("/t/proj"))
            .unwrap();
        let sid = store
            .upsert_session(&NewSession {
                session_uuid: "agg-1".into(),
                project_id: pid,
                source_file: "/t/a.jsonl".into(),
                cwd: Some("/t".into()),
                model: Some("claude-sonnet-4-6".into()),
                first_message: Some("hi".into()),
                started_at: Some("2026-01-01T00:00:00".into()),
                source_kind: None,
            })
            .unwrap();
        store
            .update_session_completion(sid, "2026-01-01T01:00:00", 2, 0.50, 3_600_000)
            .unwrap();
        let t1 = store
            .insert_turn(&NewTurn {
                session_id: sid,
                turn_number: 1,
                prompt_text: None,
                response_text: None,
                model: Some("claude-sonnet-4-6".into()),
                duration_ms: Some(1000),
                started_at: None,
                human_authored: true,
            })
            .unwrap();
        let t2 = store
            .insert_turn(&NewTurn {
                session_id: sid,
                turn_number: 2,
                prompt_text: None,
                response_text: None,
                model: Some("claude-sonnet-4-6".into()),
                duration_ms: Some(2000),
                started_at: None,
                human_authored: true,
            })
            .unwrap();
        for turn_id in [t1, t2] {
            store
                .insert_token_usage(&NewTokenUsage {
                    turn_id,
                    model: "claude-sonnet-4-6".into(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 500,
                    estimated_cost_usd: 0.25,
                })
                .unwrap();
        }
        store
            .insert_tool_call(&NewToolCall {
                turn_id: t1,
                tool_use_id: "tu1".into(),
                tool_name: "Read".into(),
                input_summary: None,
                is_error: false,
                result_summary: None,
                duration_ms: None,
            })
            .unwrap();
        store
            .insert_tool_call(&NewToolCall {
                turn_id: t1,
                tool_use_id: "tu2".into(),
                tool_name: "Edit".into(),
                input_summary: None,
                is_error: true,
                result_summary: None,
                duration_ms: None,
            })
            .unwrap();
        sid
    }

    #[test]
    fn test_aggregate_prompt_efficiency() {
        let store = test_store();
        seed_for_aggregations(&store);
        let rows = store.aggregate_prompt_efficiency(10).unwrap();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.project_name, "tproj");
        assert_eq!(r.total_input_tokens, 2000);
        assert_eq!(r.total_output_tokens, 1000);
        assert_eq!(r.tool_call_count, 2);
        assert!((r.cost_usd - 0.50).abs() < 0.001);
        // cache_hit_ratio = 1000 read / (2000 in + 1000 read) = 1/3
        assert!((r.cache_hit_ratio - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_detect_tool_patterns() {
        let store = test_store();
        seed_for_aggregations(&store);
        let rows = store.detect_tool_patterns(1).unwrap();
        // one adjacent bigram: Read -> Edit
        let found = rows
            .iter()
            .find(|p| p.sequence == vec!["Read".to_string(), "Edit".to_string()]);
        assert!(
            found.is_some(),
            "Read->Edit bigram expected, got {:?}",
            rows
        );
        let p = found.unwrap();
        assert_eq!(p.frequency, 1);
        assert!(
            p.error_rate > 0.0,
            "Edit had is_error=true -> error_rate > 0"
        );
        assert!(p.is_anti_pattern, "Read->Edit flagged as anti-pattern");
    }

    #[test]
    fn test_compare_model_performance() {
        let store = test_store();
        seed_for_aggregations(&store);
        let rows = store.compare_model_performance().unwrap();
        let m = rows
            .iter()
            .find(|m| m.model == "claude-sonnet-4-6")
            .expect("model row");
        assert_eq!(m.total_sessions, 1);
        assert!((m.avg_input_tokens - 1000.0).abs() < 0.01);
        assert!((m.avg_output_tokens - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_aggregate_session_comparisons() {
        let store = test_store();
        let sid = seed_for_aggregations(&store);
        let rows = store.aggregate_session_comparisons(10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].session_uuid, "agg-1");
        assert_eq!(rows[0].duration_ms, 3_600_000);
        assert!((rows[0].total_cost_usd - 0.50).abs() < 0.001);
        let _ = sid;
    }
}
