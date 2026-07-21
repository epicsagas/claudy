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
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM turns WHERE id NOT IN (
                SELECT MIN(id) FROM turns GROUP BY session_id, turn_number
             )",
            [],
        )?;
        tx.execute(
            "DELETE FROM token_usage WHERE turn_id NOT IN (SELECT id FROM turns)",
            [],
        )?;
        tx.execute(
            "DELETE FROM tool_calls WHERE turn_id NOT IN (SELECT id FROM turns)",
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
}
