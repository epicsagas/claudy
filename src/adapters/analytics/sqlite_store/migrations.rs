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
        // FOREIGN KEY constraint failed. The set of duplicate turn ids is
        // materialized once into a TEMP VIEW so the same subquery isn't repeated
        // across all three DELETE statements — each child table joins against it
        // directly instead of re-running the GROUP BY.
        let tx = conn.transaction()?;
        tx.execute_batch(
            "CREATE TEMP VIEW IF NOT EXISTS _v2_dup_turn_ids AS
             SELECT id FROM turns WHERE id NOT IN (
                SELECT MIN(id) FROM turns GROUP BY session_id, turn_number
             );",
        )?;
        tx.execute(
            "DELETE FROM token_usage WHERE turn_id IN (SELECT id FROM _v2_dup_turn_ids)",
            [],
        )?;
        tx.execute(
            "DELETE FROM tool_calls WHERE turn_id IN (SELECT id FROM _v2_dup_turn_ids)",
            [],
        )?;
        tx.execute(
            "DELETE FROM turns WHERE id NOT IN (
                SELECT MIN(id) FROM turns GROUP BY session_id, turn_number
             )",
            [],
        )?;
        tx.execute_batch("DROP VIEW IF EXISTS _v2_dup_turn_ids;")?;
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

    if current < 3 {
        // v3: enforce tool_calls.tool_use_id uniqueness via a UNIQUE index + a
        // one-shot dedup pass. Gated on `current < 3` like every other step so we
        // don't run the full-table dedup scan on every initialize_schema.
        enforce_tool_use_id_unique(conn)?;
        conn.execute(
            "INSERT OR REPLACE INTO migration_version (version) VALUES (3)",
            [],
        )?;
    }

    if current < 4 {
        // v4: session_outcomes — per-session outcome counters written during ingestion
        // (commit/revert counts, tool-failure counts).
        //
        // The same table is declared in the base SCHEMA, which `initialize_schema`
        // runs first, so in that path this step is already a no-op. It is kept
        // self-contained — table AND index, matching SCHEMA exactly — so the
        // migration alone brings a pre-v4 DB fully up to date and does not
        // silently depend on SCHEMA having run.
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS session_outcomes (
                session_uuid TEXT PRIMARY KEY,
                repo TEXT NOT NULL,
                started_at TEXT,
                ended_at TEXT,
                n_tool_calls INTEGER DEFAULT 0,
                n_tool_fail INTEGER DEFAULT 0,
                commits_made INTEGER DEFAULT 0,
                reverts_made INTEGER DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_session_outcomes_repo ON session_outcomes(repo);
            "#,
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO migration_version (version) VALUES (4)",
            [],
        )?;
    }

    // Self-healing guard: runs on every initialize_schema, but it only reads one
    // row from sqlite_master (cheap) and no-ops when the index is already UNIQUE.
    // This repairs the one realistic failure mode the version gate can't catch —
    // a DB whose migration_version row was bumped to >=3 but whose index never
    // became UNIQUE (a prior binary set the row and crashed mid-migration, or the
    // DB was hand-edited). The expensive dedup pass only runs when truly needed.
    if !tool_use_id_index_is_unique(conn)? {
        enforce_tool_use_id_unique(conn)?;
    }

    Ok(())
}

/// Whether `idx_tool_calls_use_id` exists and is declared UNIQUE. Reads the live
/// index definition from `sqlite_master` so the answer reflects the actual on-disk
/// state, not the migration_version counter.
fn tool_use_id_index_is_unique(conn: &Connection) -> anyhow::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT sql FROM sqlite_master
             WHERE type='index' AND name='idx_tool_calls_use_id'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .is_some_and(|sql| sql.to_uppercase().contains("UNIQUE")))
}

/// Dedup tool_calls by tool_use_id (keeping the earliest id) and recreate
/// `idx_tool_calls_use_id` as UNIQUE. The pre-v3 schema carried a non-unique
/// index, and `parse_and_ingest` re-inserting the same tool_use_id on a
/// re-parsed file produced duplicate rows (which also made
/// `update_tool_call_result` overwrite the wrong turn's result).
fn enforce_tool_use_id_unique(conn: &mut Connection) -> anyhow::Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM tool_calls WHERE id NOT IN (
            SELECT MIN(id) FROM tool_calls GROUP BY tool_use_id
         )",
        [],
    )?;
    tx.execute_batch(
        "DROP INDEX IF EXISTS idx_tool_calls_use_id;
         CREATE UNIQUE INDEX idx_tool_calls_use_id ON tool_calls(tool_use_id);",
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

    /// AC (gate): once migration_version >= 3 on a healthy DB, re-running
    /// initialize_schema must NOT re-execute the v3 dedup pass — the dedup DELETE
    /// is gated on `current < 3`, and the self-healing guard no-ops because the
    /// index is already UNIQUE. Re-running on a populated DB must preserve every
    /// tool_calls row (the expensive scan must not fire).
    #[test]
    fn test_v3_gate_skips_redundant_dedup_on_healthy_db() {
        use crate::domain::analytics::{NewSession, NewToolCall, NewTurn};
        let db = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db.path().to_str().unwrap()).expect("open");
        store.initialize_schema().unwrap();

        // Seed distinct tool_calls (each a unique tool_use_id — none should ever
        // be considered a duplicate).
        let pid = store.upsert_project("-g", "g", None).expect("project");
        let sid = store
            .upsert_session(&NewSession {
                session_uuid: "uuid-gate".into(),
                project_id: pid,
                source_file: "/g.jsonl".into(),
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
        for n in 0..5 {
            store
                .insert_tool_call(&NewToolCall {
                    turn_id: tid,
                    tool_use_id: format!("tu-{n}"),
                    tool_name: "Read".into(),
                    input_summary: None,
                    is_error: false,
                    result_summary: None,
                    duration_ms: None,
                })
                .unwrap();
        }

        // Re-run initialize_schema twice — version is already >=3 and the index
        // is UNIQUE, so the dedup pass must be skipped both times.
        store.initialize_schema().unwrap();
        store.initialize_schema().unwrap();

        let conn = store.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_calls", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 5, "gated v3 must not re-run dedup on a healthy DB");

        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM migration_version",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(version >= 3);
    }

    /// AC (v4): a fresh DB has session_outcomes (declared in the base SCHEMA) and the
    /// migration version advances to >= 4.
    #[test]
    fn test_migration_v4_creates_session_outcomes_on_fresh_db() {
        let db = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db.path().to_str().unwrap()).expect("open");
        store.initialize_schema().expect("schema");

        let conn = store.lock().unwrap();
        let table: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='table' AND name='session_outcomes'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(table, 1, "session_outcomes must exist on a fresh DB");

        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM migration_version",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(version >= 4);
    }

    /// AC (v4 upgrade): a pre-v4 DB (version 3, no session_outcomes) gains the table
    /// AND its index from the migration itself.
    ///
    /// This drives `apply` directly rather than `initialize_schema`, because
    /// `initialize_schema` executes the base SCHEMA first — which already
    /// contains `CREATE TABLE IF NOT EXISTS session_outcomes`. Going through it would
    /// pass even if the v4 step were deleted, testing nothing.
    #[test]
    fn test_migration_v4_adds_session_outcomes_to_existing_v3_db() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("v3.db");
        let mut conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        // Minimal pre-v4 schema: version 3, no session_outcomes table. tool_calls and
        // its UNIQUE index are what a real v3 DB carries, and the self-healing
        // guard at the end of `apply` inspects them.
        conn.execute_batch(
            r#"
            CREATE TABLE migration_version (version INTEGER PRIMARY KEY);
            INSERT INTO migration_version (version) VALUES (3);
            CREATE TABLE tool_calls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                turn_id INTEGER,
                tool_use_id TEXT,
                tool_name TEXT
            );
            CREATE UNIQUE INDEX idx_tool_calls_use_id ON tool_calls(tool_use_id);
            "#,
        )
        .unwrap();

        apply(&mut conn).expect("v4 migration on a bare v3 DB");

        let table: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='table' AND name='session_outcomes'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            table, 1,
            "v4 migration must create session_outcomes on a v3 DB"
        );

        let index: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='index' AND name='idx_session_outcomes_repo'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(index, 1, "v4 must not diverge from SCHEMA on the index");

        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM migration_version",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(version >= 4);

        // Idempotent: a second pass over an already-migrated DB is a no-op.
        apply(&mut conn).expect("re-apply is safe");
    }
}
