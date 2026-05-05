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
        "INSERT INTO sessions (session_uuid, project_id, source_file, cwd, model, first_message, started_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![session.session_uuid, session.project_id, session.source_file,
                session.cwd, session.model, session.first_message, session.started_at],
    )?;
    Ok(conn.last_insert_rowid())
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
) -> anyhow::Result<i64> {
    let conn = store.lock()?;
    conn.execute(
        "INSERT INTO turns (session_id, turn_number, prompt_text, response_text, model, duration_ms, started_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![turn.session_id, turn.turn_number, turn.prompt_text,
                turn.response_text, turn.model, turn.duration_ms, turn.started_at],
    )?;
    Ok(conn.last_insert_rowid())
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
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
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
