use crate::domain::analytics::*;
use rusqlite::{OptionalExtension, params};

use super::SqliteAnalyticsStore;

fn map_session_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionRecord> {
    Ok(SessionRecord {
        id: row.get(0)?,
        session_uuid: row.get(1)?,
        project_id: row.get(2)?,
        cwd: row.get(3)?,
        model: row.get(4)?,
        started_at: row.get(5)?,
        ended_at: row.get(6)?,
        total_turns: row.get(7)?,
        total_cost_usd: row.get(8)?,
        total_duration_ms: row.get(9)?,
        first_message: row.get(10)?,
        source_file: row.get(11)?,
    })
}

pub(super) fn upsert_session_impl(
    store: &SqliteAnalyticsStore,
    session: &NewSession,
) -> anyhow::Result<i64> {
    let conn = store.lock()?;
    {
        let mut stmt = conn.prepare("SELECT id FROM sessions WHERE session_uuid = ?1")?;
        if let Some(row) = stmt
            .query_row(params![session.session_uuid], |r| r.get::<_, i64>(0))
            .optional()?
        {
            return Ok(row);
        }
    }
    conn.execute(
        "INSERT INTO sessions (session_uuid, project_id, source_file, cwd, model, first_message, started_at, source_kind)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![session.session_uuid, session.project_id, session.source_file,
                session.cwd, session.model, session.first_message, session.started_at,
                session.source_kind],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Freshness for `analytics status`. Domain-neutral: latest turn timestamp,
/// total turn count, and per-source last-seen (neutral `source_kind` label).
/// Empty/NULL values yield None, not errors.
pub(super) fn ingestion_freshness_impl(
    store: &SqliteAnalyticsStore,
) -> anyhow::Result<FreshnessReport> {
    let conn = store.lock()?;

    let (latest, total): (Option<String>, i64) =
        conn.query_row("SELECT MAX(started_at), COUNT(*) FROM turns", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

    let mut per_source: Vec<FreshnessSource> = Vec::new();
    // source_kind exists after schema migration v1; tolerate a DB that somehow
    // lacks it by falling back to no per-source breakdown.
    let has_source_kind = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name='source_kind'")?
        .query_row([], |row| row.get::<_, i64>(0))?
        > 0;
    if has_source_kind {
        let mut stmt = conn.prepare(
            "SELECT s.source_kind, MAX(t.started_at)
             FROM turns t JOIN sessions s ON t.session_id = s.id
             WHERE s.source_kind IS NOT NULL
             GROUP BY s.source_kind
             ORDER BY MAX(t.started_at) DESC",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let label: String = row.get(0)?;
            let last_seen: Option<String> = row.get(1)?;
            per_source.push(FreshnessSource { label, last_seen });
        }
    }

    Ok(FreshnessReport {
        latest_turn_at: latest.filter(|s| !s.is_empty()),
        total_turns: total,
        per_source,
    })
}

pub(super) fn update_session_completion_impl(
    store: &SqliteAnalyticsStore,
    session_id: i64,
    ended_at: &str,
    total_turns: i32,
    total_cost_usd: f64,
    total_duration_ms: i64,
) -> anyhow::Result<()> {
    store.lock()?.execute(
        "UPDATE sessions SET ended_at = ?1, total_turns = ?2, total_cost_usd = ?3, total_duration_ms = ?4
         WHERE id = ?5",
        params![ended_at, total_turns, total_cost_usd, total_duration_ms, session_id],
    )?;
    Ok(())
}

pub(super) fn get_sessions_impl(
    store: &SqliteAnalyticsStore,
    limit: u32,
    days: Option<u32>,
    project_id: Option<i64>,
) -> anyhow::Result<Vec<SessionRecord>> {
    let conn = store.lock()?;
    let mut result = Vec::new();

    let base_cols = "SELECT id, session_uuid, project_id, cwd, model, started_at, ended_at, total_turns, total_cost_usd, total_duration_ms, first_message, source_file FROM sessions";

    match (days, project_id) {
        (Some(d), Some(pid)) => {
            let sql = format!(
                "{base_cols} WHERE started_at > date('now', '-' || ?1 || ' days') AND project_id = ?2 ORDER BY started_at DESC LIMIT ?3"
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params![d, pid, limit])?;
            while let Some(row) = rows.next()? {
                result.push(map_session_row(row)?);
            }
        }
        (Some(d), None) => {
            let sql = format!(
                "{base_cols} WHERE started_at > date('now', '-' || ?1 || ' days') ORDER BY started_at DESC LIMIT ?2"
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params![d, limit])?;
            while let Some(row) = rows.next()? {
                result.push(map_session_row(row)?);
            }
        }
        (None, Some(pid)) => {
            let sql =
                format!("{base_cols} WHERE project_id = ?1 ORDER BY started_at DESC LIMIT ?2");
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params![pid, limit])?;
            while let Some(row) = rows.next()? {
                result.push(map_session_row(row)?);
            }
        }
        (None, None) => {
            let sql = format!("{base_cols} ORDER BY started_at DESC LIMIT ?1");
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params![limit])?;
            while let Some(row) = rows.next()? {
                result.push(map_session_row(row)?);
            }
        }
    }
    Ok(result)
}

pub(super) fn get_session_by_uuid_impl(
    store: &SqliteAnalyticsStore,
    uuid: &str,
) -> anyhow::Result<Option<SessionRecord>> {
    let conn = store.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, session_uuid, project_id, cwd, model, started_at, ended_at, total_turns, total_cost_usd, total_duration_ms, first_message, source_file
         FROM sessions WHERE session_uuid = ?1",
    )?;
    let row = stmt.query_row(params![uuid], map_session_row).optional()?;
    Ok(row)
}

pub(super) fn insert_turn_impl(
    store: &SqliteAnalyticsStore,
    turn: &NewTurn,
) -> anyhow::Result<Option<i64>> {
    let conn = store.lock()?;
    // Idempotent insert keyed on UNIQUE(session_id, turn_number): returns the row
    // id when this turn is newly inserted, None when it already existed (prior
    // ingest re-parsed the file). The parser treats None as "skip this turn's
    // children too", which is what stops the hourly scheduler from duplicating
    // token-usage and tool-calls on actively-growing transcripts.
    let tid: Option<i64> = conn
        .query_row(
            "INSERT INTO turns (session_id, turn_number, prompt_text, response_text, model, duration_ms, started_at, human_authored)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(session_id, turn_number) DO NOTHING
             RETURNING id",
            params![turn.session_id, turn.turn_number, turn.prompt_text,
                    turn.response_text, turn.model, turn.duration_ms, turn.started_at,
                    turn.human_authored as i32],
            |row| row.get(0),
        )
        .optional()?;
    Ok(tid)
}

/// Backfill NULL model on a session's turns from the session model (R4).
pub(super) fn backfill_null_turn_models_impl(
    store: &SqliteAnalyticsStore,
    session_id: i64,
    model: &str,
) -> anyhow::Result<u64> {
    Ok(store.lock()?.execute(
        "UPDATE turns SET model = ?1 WHERE session_id = ?2 AND model IS NULL",
        params![model, session_id],
    )? as u64)
}

/// Count turns already stored for a session — used to keep numbering contiguous
/// across incremental resumes (R1 of #53).
pub(super) fn get_turn_count_impl(
    store: &SqliteAnalyticsStore,
    session_id: i64,
) -> anyhow::Result<i64> {
    Ok(store.lock()?.query_row(
        "SELECT COUNT(*) FROM turns WHERE session_id = ?1",
        params![session_id],
        |row| row.get(0),
    )?)
}

pub(super) fn get_turns_by_session_impl(
    store: &SqliteAnalyticsStore,
    session_id: i64,
) -> anyhow::Result<Vec<TurnRecord>> {
    let conn = store.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, session_id, turn_number, prompt_text, response_text, model, duration_ms, started_at
         FROM turns WHERE session_id = ?1 ORDER BY turn_number",
    )?;
    let mut rows = stmt.query(params![session_id])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(TurnRecord {
            id: row.get(0)?,
            session_id: row.get(1)?,
            turn_number: row.get(2)?,
            prompt_text: row.get(3)?,
            response_text: row.get(4)?,
            model: row.get(5)?,
            duration_ms: row.get(6)?,
            started_at: row.get(7)?,
        });
    }
    Ok(result)
}

/// Set a turn's duration. Keyed by (session, turn_number) — the natural key a
/// re-parse can always reconstruct — so a full re-ingest backfills durations
/// onto turns whose insert was a no-op conflict.
pub(super) fn update_turn_duration_impl(
    store: &SqliteAnalyticsStore,
    session_id: i64,
    turn_number: i32,
    duration_ms: i64,
) -> anyhow::Result<()> {
    store.lock()?.execute(
        "UPDATE turns SET duration_ms = ?1 WHERE session_id = ?2 AND turn_number = ?3",
        params![duration_ms, session_id, turn_number],
    )?;
    Ok(())
}

pub(super) fn insert_token_usage_impl(
    store: &SqliteAnalyticsStore,
    usage: &NewTokenUsage,
) -> anyhow::Result<()> {
    store.lock()?.execute(
        "INSERT INTO token_usage (turn_id, model, input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens, estimated_cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![usage.turn_id, usage.model, usage.input_tokens, usage.output_tokens,
                usage.cache_creation_input_tokens, usage.cache_read_input_tokens, usage.estimated_cost_usd],
    )?;
    Ok(())
}

pub(super) fn insert_tool_call_impl(
    store: &SqliteAnalyticsStore,
    call: &NewToolCall,
) -> anyhow::Result<()> {
    store.lock()?.execute(
        "INSERT INTO tool_calls (turn_id, tool_use_id, tool_name, input_summary, is_error, result_summary, duration_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(tool_use_id) DO UPDATE SET
           turn_id        = excluded.turn_id,
           tool_name      = excluded.tool_name,
           input_summary  = COALESCE(excluded.input_summary, tool_calls.input_summary),
           result_summary = COALESCE(excluded.result_summary, tool_calls.result_summary),
           duration_ms    = COALESCE(excluded.duration_ms, tool_calls.duration_ms)",
        params![call.turn_id, call.tool_use_id, call.tool_name, call.input_summary,
                call.is_error as i32, call.result_summary, call.duration_ms],
    )?;
    Ok(())
}

pub(super) fn update_tool_call_result_impl(
    store: &SqliteAnalyticsStore,
    tool_use_id: &str,
    is_error: bool,
    result_summary: Option<&str>,
) -> anyhow::Result<()> {
    store.lock()?.execute(
        "UPDATE tool_calls SET is_error = ?1, result_summary = ?2 WHERE tool_use_id = ?3",
        params![is_error as i32, result_summary, tool_use_id],
    )?;
    Ok(())
}

/// Write a session's outcome counters.
///
/// [`OutcomeWriteMode::Replace`] inserts or overwrites: the caller read the transcript
/// from byte 0, so its counts describe the whole session.
///
/// [`OutcomeWriteMode::Accumulate`] adds a resumed parse's tail counts to an existing
/// row and does nothing when no row exists — a tail alone is not a session, and
/// inserting it would leave a fragment indistinguishable from a complete count.
/// The row is created later by the next parse that starts at byte 0.
///
/// All sessions are stored: `repo` is the raw session cwd and may be empty when
/// the transcript carried none. An empty `repo` never overwrites a known one,
/// so a resumed parse that failed to observe the cwd can't erase it.
pub(super) fn upsert_session_outcome_impl(
    store: &SqliteAnalyticsStore,
    outcomes: &NewSessionOutcome,
    mode: OutcomeWriteMode,
) -> anyhow::Result<()> {
    let sql = match mode {
        OutcomeWriteMode::Replace => {
            "INSERT INTO session_outcomes
                (session_uuid, repo, started_at, ended_at,
                 n_tool_calls, n_tool_fail, commits_made, reverts_made, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
             ON CONFLICT(session_uuid) DO UPDATE SET
                repo=CASE WHEN excluded.repo <> '' THEN excluded.repo ELSE session_outcomes.repo END,
                started_at=COALESCE(excluded.started_at, session_outcomes.started_at),
                ended_at=COALESCE(excluded.ended_at, session_outcomes.ended_at),
                n_tool_calls=excluded.n_tool_calls,
                n_tool_fail=excluded.n_tool_fail,
                commits_made=excluded.commits_made,
                reverts_made=excluded.reverts_made,
                updated_at=datetime('now')"
        }
        OutcomeWriteMode::Accumulate => {
            "UPDATE session_outcomes SET
                repo=CASE WHEN ?2 <> '' THEN ?2 ELSE repo END,
                started_at=COALESCE(started_at, ?3),
                ended_at=COALESCE(?4, ended_at),
                n_tool_calls=n_tool_calls + ?5,
                n_tool_fail=n_tool_fail + ?6,
                commits_made=commits_made + ?7,
                reverts_made=reverts_made + ?8,
                updated_at=datetime('now')
             WHERE session_uuid = ?1"
        }
    };
    store.lock()?.execute(
        sql,
        params![
            outcomes.session_uuid,
            outcomes.repo,
            outcomes.started_at,
            outcomes.ended_at,
            outcomes.n_tool_calls,
            outcomes.n_tool_fail,
            outcomes.commits_made,
            outcomes.reverts_made,
        ],
    )?;
    Ok(())
}

pub(super) fn get_tool_calls_by_turn_impl(
    store: &SqliteAnalyticsStore,
    turn_id: i64,
) -> anyhow::Result<Vec<ToolCallRecord>> {
    let conn = store.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, turn_id, tool_use_id, tool_name, input_summary, is_error, result_summary, duration_ms
         FROM tool_calls WHERE turn_id = ?1",
    )?;
    let mut rows = stmt.query(params![turn_id])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(ToolCallRecord {
            id: row.get(0)?,
            turn_id: row.get(1)?,
            tool_use_id: row.get(2)?,
            tool_name: row.get(3)?,
            input_summary: row.get(4)?,
            is_error: row.get::<_, i32>(5)? != 0,
            result_summary: row.get(6)?,
            duration_ms: row.get(7)?,
        });
    }
    Ok(result)
}

pub(super) fn insert_channel_metric_impl(
    store: &SqliteAnalyticsStore,
    record: &ChannelMetricRecord,
) -> anyhow::Result<()> {
    store.lock()?.execute(
        "INSERT INTO channel_metrics (session_id, platform, channel_id, user_id, profile, stream_duration_ms, first_byte_ms, stream_timeout, error_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            record.session_id,
            record.platform,
            record.channel_id,
            record.user_id,
            record.profile,
            record.stream_duration_ms,
            record.first_byte_ms,
            record.stream_timeout as i32,
            record.error_type,
        ],
    )?;
    Ok(())
}

pub(super) fn get_checkpoint_impl(
    store: &SqliteAnalyticsStore,
    file_path: &str,
) -> anyhow::Result<Option<IngestionCheckpoint>> {
    let conn = store.lock()?;
    let mut stmt = conn.prepare(
        "SELECT file_path, file_modified, byte_offset, line_count FROM ingestion_checkpoints WHERE file_path = ?1",
    )?;
    let row = stmt
        .query_row(params![file_path], |row| {
            Ok(IngestionCheckpoint {
                file_path: row.get(0)?,
                file_modified: row.get(1)?,
                byte_offset: row.get(2)?,
                line_count: row.get(3)?,
            })
        })
        .optional()?;
    Ok(row)
}

pub(super) fn upsert_checkpoint_impl(
    store: &SqliteAnalyticsStore,
    file_path: &str,
    file_modified: &str,
    byte_offset: i64,
    line_count: i64,
) -> anyhow::Result<()> {
    store.lock()?.execute(
        "INSERT INTO ingestion_checkpoints (file_path, file_modified, byte_offset, line_count)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(file_path) DO UPDATE SET
           file_modified = excluded.file_modified,
           byte_offset = excluded.byte_offset,
           line_count = excluded.line_count,
           ingested_at = datetime('now')",
        params![file_path, file_modified, byte_offset, line_count],
    )?;
    Ok(())
}

pub(super) fn clear_recommendations_impl(store: &SqliteAnalyticsStore) -> anyhow::Result<()> {
    store.lock()?.execute("DELETE FROM recommendations", [])?;
    Ok(())
}

pub(super) fn insert_recommendation_impl(
    store: &SqliteAnalyticsStore,
    rec: &Recommendation,
) -> anyhow::Result<()> {
    let category = serde_json::to_string(&rec.category)?;
    let severity = serde_json::to_string(&rec.severity)?;
    store.lock()?.execute(
        "INSERT INTO recommendations (category, severity, title, description, action)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![category, severity, rec.title, rec.description, rec.action],
    )?;
    Ok(())
}

pub(super) fn get_recommendations_impl(
    store: &SqliteAnalyticsStore,
) -> anyhow::Result<Vec<Recommendation>> {
    let conn = store.lock()?;
    let mut stmt = conn.prepare(
        "SELECT category, severity, title, description, action FROM recommendations WHERE dismissed_at IS NULL ORDER BY rowid DESC",
    )?;
    let mut rows = stmt.query([])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let category: String = row.get(0)?;
        let severity: String = row.get(1)?;
        result.push(Recommendation {
            category: serde_json::from_str(&category)?,
            severity: serde_json::from_str(&severity)?,
            title: row.get(2)?,
            description: row.get(3)?,
            action: row.get(4)?,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::analytics::{NewSession, NewTurn};
    use crate::ports::analytics_ports::AnalyticsStore;
    use chrono::{Local, TimeDelta};
    use tempfile::NamedTempFile;

    fn fresh_store() -> SqliteAnalyticsStore {
        let db = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db.path().to_str().unwrap()).unwrap();
        store.initialize_schema().unwrap();
        store
    }

    fn seed_turn(store: &SqliteAnalyticsStore, started_at: &str, source_kind: Option<&str>) {
        let pid = store.upsert_project("proj", "proj", None).unwrap();
        let sid = store.upsert_session(&NewSession {
            session_uuid: format!("uuid-{started_at}"),
            project_id: pid,
            source_file: "f".into(),
            cwd: None,
            model: None,
            first_message: None,
            started_at: Some(started_at.into()),
            source_kind: source_kind.map(String::from),
        });
        // upsert_session returns Result; unwrap in test
        let sid = sid.unwrap();
        store
            .insert_turn(&NewTurn {
                session_id: sid,
                turn_number: 1,
                prompt_text: None,
                response_text: None,
                model: None,
                duration_ms: None,
                started_at: Some(started_at.into()),
                human_authored: true,
            })
            .unwrap();
    }

    #[test]
    fn test_freshness_empty_db_is_not_stale() {
        let store = fresh_store();
        let report = store.ingestion_freshness().unwrap();
        assert_eq!(report.total_turns, 0);
        assert!(report.latest_turn_at.is_none());
        assert!(report.per_source.is_empty());
    }

    #[test]
    fn test_freshness_detects_old_turn_for_staleness() {
        // AC-R3: a DB whose newest turn is ~30 days old must register as stale.
        let store = fresh_store();
        let old = (Local::now().date_naive() - TimeDelta::days(30))
            .format("%Y-%m-%dT12:00:00Z")
            .to_string();
        seed_turn(&store, &old, Some("live"));

        let report = store.ingestion_freshness().unwrap();
        assert_eq!(report.total_turns, 1);
        let latest = report.latest_turn_at.expect("latest turn present");
        let latest_date = chrono::NaiveDate::parse_from_str(&latest[..10], "%Y-%m-%d").unwrap();
        let days = (Local::now().date_naive() - latest_date).num_days();
        assert!(days >= 29, "expected ~30 days stale, got {days}");
        // staleness threshold check (mirrors run_status): days > 2 ⇒ stale
        assert!(days > 2);

        // per-source breakdown carries the neutral source label
        assert_eq!(report.per_source.len(), 1);
        assert_eq!(report.per_source[0].label, "live");
    }

    #[test]
    fn test_freshness_recent_turn_is_fresh() {
        let store = fresh_store();
        let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        seed_turn(&store, &now, Some("live"));

        let report = store.ingestion_freshness().unwrap();
        let latest = report.latest_turn_at.expect("latest turn present");
        let latest_date = chrono::NaiveDate::parse_from_str(&latest[..10], "%Y-%m-%d").unwrap();
        let days = (Local::now().date_naive() - latest_date).num_days();
        assert!(days <= 1, "recent turn must be fresh, got {days} days");
    }
}
