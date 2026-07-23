//! Idempotent schema migrations, gated by `migration_version`.
//!
//! The base `SCHEMA` creates tables for fresh databases; these migrations
//! upgrade pre-existing databases in place. Each `ALTER` is guarded by a
//! `pragma_table_info` check so re-running is always safe.

use rusqlite::Connection;

/// Apply all pending migrations.
pub(super) fn apply(conn: &mut Connection) -> anyhow::Result<()> {
    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM migration_version",
        [],
        |row| row.get(0),
    )?;

    if current < 1 {
        // v1 (R2 + R4): neutral source label on sessions; author flag on turns.
        add_column_if_missing(conn, "sessions", "source_kind", "TEXT")?;
        add_column_if_missing(conn, "turns", "human_authored", "INTEGER DEFAULT 0")?;
        conn.execute(
            "INSERT OR REPLACE INTO migration_version (version) VALUES (1)",
            [],
        )?;
    }

    if current < 2 {
        // v2: dedup existing turns and enforce UNIQUE(session_id, turn_number) so
        // the hourly ingestion scheduler cannot compound-duplicate turns on
        // actively-growing JSONL files. Token-usage / tool-call rows orphaned by
        // the turn dedup are removed too. Wrapped in a transaction so a failure
        // (e.g. the unique index hitting an unexpected dup) rolls back cleanly.
        //
        // FK order matters (PRAGMA foreign_keys=ON): children must be deleted
        // BEFORE the turn rows they reference, else the turns DELETE aborts with
        // FOREIGN KEY constraint failed. Delete orphaned token_usage / tool_calls
        // referencing the soon-to-be-removed duplicate turns first, then the turns.
        let tx = conn.transaction()?;
        let dup_turn_ids = "SELECT id FROM turns WHERE id NOT IN (
            SELECT MIN(id) FROM turns GROUP BY session_id, turn_number
         )";
        tx.execute(
            &format!("DELETE FROM token_usage WHERE turn_id IN ({dup_turn_ids})"),
            [],
        )?;
        tx.execute(
            &format!("DELETE FROM tool_calls WHERE turn_id IN ({dup_turn_ids})"),
            [],
        )?;
        tx.execute(
            "DELETE FROM turns WHERE id NOT IN (
                SELECT MIN(id) FROM turns GROUP BY session_id, turn_number
             )",
            [],
        )?;
        tx.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_turns_session_turn
             ON turns(session_id, turn_number)",
            [],
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO migration_version (version) VALUES (2)",
            [],
        )?;
        tx.commit()?;
    }

    // v3: enforce tool_calls.tool_use_id uniqueness. Self-healing — checks the
    // *actual* index state, not just the version counter, so a DB that reached
    // version>=3 without the work landing (e.g. a prior binary bumped the row
    // but failed mid-migration) is still repaired. Always safe to re-run.
    enforce_tool_use_id_unique(conn)?;

    Ok(())
}

/// Dedup tool_calls by tool_use_id (keeping the earliest id) and ensure the
/// `idx_tool_calls_use_id` index is UNIQUE. Idempotent and self-healing: it
/// inspects the live index definition and only acts if dedup is needed or the
/// index is missing/non-unique. The pre-v3 schema carried a non-unique index,
/// and `parse_and_ingest` re-inserting the same tool_use_id on a re-parsed file
/// produced duplicate rows (which also made `update_tool_call_result` overwrite
/// the wrong turn's result).
fn enforce_tool_use_id_unique(conn: &mut Connection) -> anyhow::Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM tool_calls WHERE id NOT IN (
            SELECT MIN(id) FROM tool_calls GROUP BY tool_use_id
         )",
        [],
    )?;

    let index_is_unique: bool = tx
        .query_row(
            "SELECT sql FROM sqlite_master
             WHERE type='index' AND name='idx_tool_calls_use_id'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .is_some_and(|sql| sql.to_uppercase().contains("UNIQUE"));

    if !index_is_unique {
        tx.execute_batch(
            "DROP INDEX IF EXISTS idx_tool_calls_use_id;
             CREATE UNIQUE INDEX idx_tool_calls_use_id ON tool_calls(tool_use_id);",
        )?;
    }

    tx.execute(
        "INSERT OR REPLACE INTO migration_version (version) VALUES (3)",
        [],
    )?;
    tx.commit()?;
    Ok(())
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    decl: &str,
) -> anyhow::Result<()> {
    let exists: i64 = conn
        .prepare(&format!(
            "SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = '{column}'"
        ))?
        .query_row([], |row| row.get(0))?;
    if exists == 0 {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"),
            [],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore;
    use crate::ports::analytics_ports::AnalyticsStore;
    use tempfile::NamedTempFile;

    fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = '{column}'"),
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
            > 0
    }

    #[test]
    fn test_migration_v1_adds_columns_on_fresh_db() {
        let db = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db.path().to_str().unwrap()).expect("open");
        store.initialize_schema().expect("schema");

        let conn = store.lock().unwrap();
        assert!(column_exists(&conn, "sessions", "source_kind"));
        assert!(column_exists(&conn, "turns", "human_authored"));

        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM migration_version",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(version >= 1);
    }

    #[test]
    fn test_migration_is_idempotent() {
        let db = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db.path().to_str().unwrap()).expect("open");
        // Running initialize_schema twice must not error (idempotent ALTERs).
        store.initialize_schema().expect("schema run 1");
        store.initialize_schema().expect("schema run 2");

        let conn = store.lock().unwrap();
        assert!(column_exists(&conn, "sessions", "source_kind"));
        assert!(column_exists(&conn, "turns", "human_authored"));
    }

    /// AC: re-inserting the same tool_use_id must upsert, not duplicate, and the
    /// tool_use_id index must be UNIQUE on a fresh DB.
    #[test]
    fn test_tool_call_idempotent_upsert_and_unique_index() {
        use crate::domain::analytics::{NewSession, NewToolCall, NewTurn};
        use crate::ports::analytics_ports::AnalyticsStore;
        let db = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db.path().to_str().unwrap()).expect("open");
        store.initialize_schema().unwrap();

        let pid = store.upsert_project("-t", "t", None).expect("project");
        let sid = store
            .upsert_session(&NewSession {
                session_uuid: "uuid-tc".into(),
                project_id: pid,
                source_file: "/t.jsonl".into(),
                cwd: None,
                model: None,
                first_message: None,
                started_at: None,
                source_kind: None,
            })
            .expect("session");
        let tid = store
            .insert_turn(&NewTurn {
                session_id: sid,
                turn_number: 1,
                prompt_text: None,
                response_text: None,
                model: None,
                duration_ms: None,
                started_at: None,
                human_authored: false,
            })
            .expect("turn")
            .expect("new turn id");

        // Same tool_use_id twice → upsert, not duplicate.
        for dur in [Some(10), Some(20)] {
            store
                .insert_tool_call(&NewToolCall {
                    turn_id: tid,
                    tool_use_id: "tu-dup".into(),
                    tool_name: "Read".into(),
                    input_summary: None,
                    is_error: false,
                    result_summary: None,
                    duration_ms: dur,
                })
                .unwrap();
        }
        let calls = store.get_tool_calls_by_turn(tid).unwrap();
        assert_eq!(calls.len(), 1, "duplicate tool_use_id must upsert");
        assert_eq!(calls[0].duration_ms, Some(20));
    }

    /// AC (self-healing): a DB seeded with a pre-v3 schema (non-unique
    /// idx_tool_calls_use_id + duplicate tool_use_id rows) must be repaired on
    // the next initialize_schema, even if migration_version was already bumped.
    #[test]
    fn test_v3_self_heals_non_unique_index_and_dupes() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("corrupt.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE migration_version (version INTEGER PRIMARY KEY);
            CREATE TABLE projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT, encoded_dir TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL, resolved_path TEXT,
                first_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_seen_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT, session_uuid TEXT NOT NULL UNIQUE,
                project_id INTEGER NOT NULL REFERENCES projects(id), cwd TEXT, model TEXT,
                first_message TEXT, started_at TEXT, ended_at TEXT, total_turns INTEGER DEFAULT 0,
                total_cost_usd REAL DEFAULT 0.0, total_duration_ms INTEGER DEFAULT 0,
                source_file TEXT NOT NULL, file_modified_at TEXT,
                ingested_at TEXT NOT NULL DEFAULT (datetime('now')), source_kind TEXT
            );
            CREATE TABLE turns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL REFERENCES sessions(id),
                turn_number INTEGER NOT NULL, prompt_text TEXT, response_text TEXT,
                model TEXT, duration_ms INTEGER, started_at TEXT,
                ingested_at TEXT NOT NULL DEFAULT (datetime('now')), human_authored INTEGER DEFAULT 0
            );
            CREATE TABLE tool_calls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                turn_id INTEGER NOT NULL REFERENCES turns(id), tool_use_id TEXT NOT NULL,
                tool_name TEXT NOT NULL, input_summary TEXT, is_error INTEGER DEFAULT 0,
                result_summary TEXT, duration_ms INTEGER,
                ingested_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_tool_calls_use_id ON tool_calls(tool_use_id);
            INSERT INTO projects (encoded_dir, display_name) VALUES ('p','p');
            INSERT INTO sessions (session_uuid, project_id, source_file) VALUES ('u', 1, '/x');
            INSERT INTO turns (session_id, turn_number) VALUES (1, 1);
            INSERT INTO tool_calls (turn_id, tool_use_id, tool_name) VALUES (1, 'dup', 'Read');
            INSERT INTO tool_calls (turn_id, tool_use_id, tool_name) VALUES (1, 'dup', 'Read');
            INSERT INTO migration_version (version) VALUES (3);
            "#,
        )
        .unwrap();
        drop(conn);

        // Open via the real store — initialize_schema runs v3 self-healing.
        let store = SqliteAnalyticsStore::open(db_path.to_str().unwrap()).unwrap();
        store.initialize_schema().unwrap();

        let conn = store.lock().unwrap();
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_calls", [], |r| r.get(0))
            .unwrap();
        assert_eq!(remaining, 1, "self-heal must dedup duplicate tool_use_id");

        let idx_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name='idx_tool_calls_use_id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            idx_sql.to_uppercase().contains("UNIQUE"),
            "self-heal must recreate the index as UNIQUE"
        );

        // A raw duplicate insert must now be rejected.
        let dup = conn.execute(
            "INSERT INTO tool_calls (turn_id, tool_use_id, tool_name) VALUES (1, 'dup', 'X')",
            [],
        );
        assert!(dup.is_err(), "UNIQUE must be enforced post-migration");
    }
}
