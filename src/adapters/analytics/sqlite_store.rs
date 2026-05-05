
use crate::domain::analytics::*;
use crate::ports::analytics_ports::{AnalyticsStore, PricingStore};
use rusqlite::{Connection, OptionalExtension, params};
use std::sync::{Mutex, MutexGuard};

pub struct SqliteAnalyticsStore {
    conn: Mutex<Connection>,
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

    fn lock(&self) -> anyhow::Result<MutexGuard<'_, Connection>> {
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

impl AnalyticsStore for SqliteAnalyticsStore {
    fn initialize_schema(&self) -> anyhow::Result<()> {
        self.lock()?.execute_batch(SCHEMA)?;
        Ok(())
    }

    fn upsert_project(
        &self,
        encoded_dir: &str,
        display_name: &str,
        resolved_path: Option<&str>,
    ) -> anyhow::Result<i64> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO projects (encoded_dir, display_name, resolved_path)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(encoded_dir) DO UPDATE SET
               display_name = excluded.display_name,
               resolved_path = COALESCE(excluded.resolved_path, projects.resolved_path),
               last_seen_at = datetime('now')",
            params![encoded_dir, display_name, resolved_path],
        )?;
        // For ON CONFLICT, last_insert_rowid returns 0 — query the actual id
        let rowid = conn.last_insert_rowid();
        if rowid > 0 {
            return Ok(rowid);
        }
        let id: i64 = conn.query_row(
            "SELECT id FROM projects WHERE encoded_dir = ?1",
            params![encoded_dir],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    fn get_project_by_encoded_dir(
        &self,
        encoded_dir: &str,
    ) -> anyhow::Result<Option<ProjectRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, encoded_dir, display_name, resolved_path FROM projects WHERE encoded_dir = ?1",
        )?;
        let row = stmt
            .query_row(params![encoded_dir], |row| {
                Ok(ProjectRecord {
                    id: row.get(0)?,
                    encoded_dir: row.get(1)?,
                    display_name: row.get(2)?,
                    resolved_path: row.get(3)?,
                })
            })
            .optional()?;
        Ok(row)
    }

    fn list_projects(&self) -> anyhow::Result<Vec<ProjectRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, encoded_dir, display_name, resolved_path FROM projects ORDER BY display_name",
        )?;
        let mut rows = stmt.query([])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(ProjectRecord {
                id: row.get(0)?,
                encoded_dir: row.get(1)?,
                display_name: row.get(2)?,
                resolved_path: row.get(3)?,
            });
        }
        Ok(result)
    }

    fn upsert_session(&self, session: &NewSession) -> anyhow::Result<i64> {
        let conn = self.lock()?;
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

    fn update_session_completion(
        &self,
        session_id: i64,
        ended_at: &str,
        total_turns: i32,
        total_cost_usd: f64,
        total_duration_ms: i64,
    ) -> anyhow::Result<()> {
        self.lock()?.execute(
            "UPDATE sessions SET ended_at = ?1, total_turns = ?2, total_cost_usd = ?3, total_duration_ms = ?4
             WHERE id = ?5",
            params![ended_at, total_turns, total_cost_usd, total_duration_ms, session_id],
        )?;
        Ok(())
    }

    fn get_sessions(
        &self,
        limit: u32,
        days: Option<u32>,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<SessionRecord>> {
        let conn = self.lock()?;
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
                let sql = format!(
                    "{base_cols} WHERE project_id = ?1 ORDER BY started_at DESC LIMIT ?2"
                );
                let mut stmt = conn.prepare(&sql)?;
                let mut rows = stmt.query(params![pid, limit])?;
                while let Some(row) = rows.next()? {
                    result.push(map_session_row(row)?);
                }
            }
            (None, None) => {
                let sql = format!(
                    "{base_cols} ORDER BY started_at DESC LIMIT ?1"
                );
                let mut stmt = conn.prepare(&sql)?;
                let mut rows = stmt.query(params![limit])?;
                while let Some(row) = rows.next()? {
                    result.push(map_session_row(row)?);
                }
            }
        }
        Ok(result)
    }

    fn get_session_by_uuid(&self, uuid: &str) -> anyhow::Result<Option<SessionRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_uuid, project_id, cwd, model, started_at, ended_at, total_turns, total_cost_usd, total_duration_ms, first_message, source_file
             FROM sessions WHERE session_uuid = ?1",
        )?;
        let row = stmt.query_row(params![uuid], map_session_row).optional()?;
        Ok(row)
    }

    fn insert_turn(&self, turn: &NewTurn) -> anyhow::Result<i64> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO turns (session_id, turn_number, prompt_text, response_text, model, duration_ms, started_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![turn.session_id, turn.turn_number, turn.prompt_text,
                    turn.response_text, turn.model, turn.duration_ms, turn.started_at],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn get_turns_by_session(&self, session_id: i64) -> anyhow::Result<Vec<TurnRecord>> {
        let conn = self.lock()?;
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

    fn insert_token_usage(&self, usage: &NewTokenUsage) -> anyhow::Result<()> {
        self.lock()?.execute(
            "INSERT INTO token_usage (turn_id, model, input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens, estimated_cost_usd)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![usage.turn_id, usage.model, usage.input_tokens, usage.output_tokens,
                    usage.cache_creation_input_tokens, usage.cache_read_input_tokens, usage.estimated_cost_usd],
        )?;
        Ok(())
    }

    fn insert_tool_call(&self, call: &NewToolCall) -> anyhow::Result<()> {
        self.lock()?.execute(
            "INSERT INTO tool_calls (turn_id, tool_use_id, tool_name, input_summary, is_error, result_summary, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![call.turn_id, call.tool_use_id, call.tool_name, call.input_summary,
                    call.is_error as i32, call.result_summary, call.duration_ms],
        )?;
        Ok(())
    }

    fn update_tool_call_result(
        &self,
        tool_use_id: &str,
        is_error: bool,
        result_summary: Option<&str>,
    ) -> anyhow::Result<()> {
        self.lock()?.execute(
            "UPDATE tool_calls SET is_error = ?1, result_summary = ?2 WHERE tool_use_id = ?3",
            params![is_error as i32, result_summary, tool_use_id],
        )?;
        Ok(())
    }

    fn get_tool_calls_by_turn(&self, turn_id: i64) -> anyhow::Result<Vec<ToolCallRecord>> {
        let conn = self.lock()?;
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

    fn insert_channel_metric(&self, record: &ChannelMetricRecord) -> anyhow::Result<()> {
        self.lock()?.execute(
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

    fn get_checkpoint(&self, file_path: &str) -> anyhow::Result<Option<IngestionCheckpoint>> {
        let conn = self.lock()?;
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

    fn upsert_checkpoint(
        &self,
        file_path: &str,
        file_modified: &str,
        byte_offset: i64,
        line_count: i64,
    ) -> anyhow::Result<()> {
        self.lock()?.execute(
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

    fn clear_recommendations(&self) -> anyhow::Result<()> {
        self.lock()?.execute("DELETE FROM recommendations", [])?;
        Ok(())
    }

    fn insert_recommendation(&self, rec: &Recommendation) -> anyhow::Result<()> {
        let category = serde_json::to_string(&rec.category)?;
        let severity = serde_json::to_string(&rec.severity)?;
        self.lock()?.execute(
            "INSERT INTO recommendations (category, severity, title, description, action)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![category, severity, rec.title, rec.description, rec.action],
        )?;
        Ok(())
    }

    fn get_recommendations(&self) -> anyhow::Result<Vec<Recommendation>> {
        let conn = self.lock()?;
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

    fn aggregate_token_trends(&self, days: u32, project_id: Option<i64>) -> anyhow::Result<Vec<TokenTrendPoint>> {
        let conn = self.lock()?;
        let query = match project_id {
            Some(_) =>
                "SELECT date(sessions.started_at) as d, token_usage.model, SUM(input_tokens), SUM(output_tokens), SUM(estimated_cost_usd), COUNT(DISTINCT sessions.id)
                 FROM token_usage
                 JOIN turns ON token_usage.turn_id = turns.id
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.started_at > date('now', '-' || ?1 || ' days') AND sessions.project_id = ?2
                 GROUP BY d, token_usage.model ORDER BY d ASC",
            None =>
                "SELECT date(sessions.started_at) as d, token_usage.model, SUM(input_tokens), SUM(output_tokens), SUM(estimated_cost_usd), COUNT(DISTINCT sessions.id)
                 FROM token_usage
                 JOIN turns ON token_usage.turn_id = turns.id
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.started_at > date('now', '-' || ?1 || ' days')
                 GROUP BY d, token_usage.model ORDER BY d ASC",
        };
        let mut stmt = conn.prepare(query)?;
        let mut rows = if let Some(pid) = project_id {
            stmt.query(params![days, pid])?
        } else {
            stmt.query(params![days])?
        };
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(TokenTrendPoint {
                date: row.get(0)?,
                model: row.get(1)?,
                input_tokens: row.get(2)?,
                output_tokens: row.get(3)?,
                total_cost_usd: row.get(4)?,
                session_count: row.get(5)?,
            });
        }
        Ok(result)
    }

    fn aggregate_tool_distribution(
        &self,
        days: Option<u32>,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<ToolDistribution>> {
        let conn = self.lock()?;
        let query = match (days, project_id) {
            (Some(_), Some(_)) =>
                "SELECT tool_name, COUNT(*), SUM(CASE WHEN is_error THEN 1 ELSE 0 END), AVG(tool_calls.duration_ms)
                 FROM tool_calls
                 JOIN turns ON tool_calls.turn_id = turns.id
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.started_at > date('now', '-' || ?1 || ' days') AND sessions.project_id = ?2
                 GROUP BY tool_name",
            (Some(_), None) =>
                "SELECT tool_name, COUNT(*), SUM(CASE WHEN is_error THEN 1 ELSE 0 END), AVG(tool_calls.duration_ms)
                 FROM tool_calls
                 JOIN turns ON tool_calls.turn_id = turns.id
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.started_at > date('now', '-' || ?1 || ' days')
                 GROUP BY tool_name",
            (None, Some(_)) =>
                "SELECT tool_name, COUNT(*), SUM(CASE WHEN is_error THEN 1 ELSE 0 END), AVG(tool_calls.duration_ms)
                 FROM tool_calls
                 JOIN turns ON tool_calls.turn_id = turns.id
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.project_id = ?1
                 GROUP BY tool_name",
            (None, None) =>
                "SELECT tool_name, COUNT(*), SUM(CASE WHEN is_error THEN 1 ELSE 0 END), AVG(tool_calls.duration_ms)
                 FROM tool_calls
                 GROUP BY tool_name",
        };
        let mut stmt = conn.prepare(query)?;
        let mut rows = match (days, project_id) {
            (Some(d), Some(pid)) => stmt.query(params![d, pid])?,
            (Some(d), None) => stmt.query(params![d])?,
            (None, Some(pid)) => stmt.query(params![pid])?,
            (None, None) => stmt.query([])?,
        };

        let mut counts = Vec::new();
        let mut total_calls: i64 = 0;
        while let Some(row) = rows.next()? {
            let count: i64 = row.get(1)?;
            total_calls += count;
            counts.push((
                row.get::<_, String>(0)?,
                count,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<f64>>(3)?,
            ));
        }

        Ok(counts
            .into_iter()
            .map(|(name, count, errors, avg_dur)| ToolDistribution {
                tool_name: name,
                call_count: count,
                error_count: errors,
                avg_duration_ms: avg_dur,
                percentage: if total_calls > 0 {
                    (count as f64 / total_calls as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect())
    }

    fn aggregate_dashboard_stats(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<DashboardStats> {
        let conn = self.lock()?;

        // Query 1: session aggregates
        let session_sql = match project_id {
            Some(_) =>
                "SELECT COUNT(*), COALESCE(SUM(total_cost_usd), 0), COALESCE(SUM(total_turns), 0), COALESCE(SUM(total_duration_ms), 0)
                 FROM sessions WHERE started_at > date('now', '-' || ?1 || ' days') AND project_id = ?2",
            None =>
                "SELECT COUNT(*), COALESCE(SUM(total_cost_usd), 0), COALESCE(SUM(total_turns), 0), COALESCE(SUM(total_duration_ms), 0)
                 FROM sessions WHERE started_at > date('now', '-' || ?1 || ' days')",
        };
        let (total_sessions, total_cost_usd, total_turns, total_duration_ms): (i64, f64, i64, i64) =
            if let Some(pid) = project_id {
                conn.query_row(session_sql, params![days, pid], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?
            } else {
                conn.query_row(session_sql, params![days], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?
            };

        // Query 2: token / cache stats
        let token_sql = match project_id {
            Some(_) =>
                "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), COALESCE(SUM(cache_read_input_tokens), 0), COALESCE(SUM(cache_creation_input_tokens), 0)
                 FROM token_usage
                 JOIN turns ON token_usage.turn_id = turns.id
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.started_at > date('now', '-' || ?1 || ' days') AND sessions.project_id = ?2",
            None =>
                "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), COALESCE(SUM(cache_read_input_tokens), 0), COALESCE(SUM(cache_creation_input_tokens), 0)
                 FROM token_usage
                 JOIN turns ON token_usage.turn_id = turns.id
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.started_at > date('now', '-' || ?1 || ' days')",
        };
        let (total_input, total_output, cache_read, cache_creation): (i64, i64, i64, i64) =
            if let Some(pid) = project_id {
                conn.query_row(token_sql, params![days, pid], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?
            } else {
                conn.query_row(token_sql, params![days], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?
            };

        // Query 3: most used model (from turns, since sessions.model is often NULL)
        let model_sql = match project_id {
            Some(_) =>
                "SELECT turns.model FROM turns
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.started_at > date('now', '-' || ?1 || ' days')
                   AND sessions.project_id = ?2 AND turns.model IS NOT NULL
                 GROUP BY turns.model ORDER BY COUNT(*) DESC LIMIT 1",
            None =>
                "SELECT turns.model FROM turns
                 JOIN sessions ON turns.session_id = sessions.id
                 WHERE sessions.started_at > date('now', '-' || ?1 || ' days')
                   AND turns.model IS NOT NULL
                 GROUP BY turns.model ORDER BY COUNT(*) DESC LIMIT 1",
        };
        let most_used_model: Option<String> = if let Some(pid) = project_id {
            conn.query_row(model_sql, params![days, pid], |row| row.get(0))
                .optional()?
                .flatten()
        } else {
            conn.query_row(model_sql, params![days], |row| row.get(0))
                .optional()?
                .flatten()
        };

        let avg_tokens_per_session = if total_sessions > 0 {
            (total_input + total_output) as f64 / total_sessions as f64
        } else {
            0.0
        };
        let cache_hit_ratio = if (cache_read + cache_creation) > 0 {
            cache_read as f64 / (cache_read + cache_creation) as f64
        } else {
            0.0
        };

        Ok(DashboardStats {
            total_sessions,
            total_cost_usd,
            total_turns,
            total_duration_ms,
            avg_tokens_per_session,
            cache_hit_ratio,
            most_used_model,
            alert_count: 0,
        })
    }

    fn aggregate_cost_metrics(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<CostMetrics> {
        let conn = self.lock()?;

        // Query a: session aggregates
        let agg_sql = match project_id {
            Some(_) =>
                "SELECT COALESCE(SUM(total_cost_usd), 0), COUNT(*), COALESCE(SUM(total_turns), 0)
                 FROM sessions WHERE started_at > date('now', '-' || ?1 || ' days') AND project_id = ?2",
            None =>
                "SELECT COALESCE(SUM(total_cost_usd), 0), COUNT(*), COALESCE(SUM(total_turns), 0)
                 FROM sessions WHERE started_at > date('now', '-' || ?1 || ' days')",
        };
        let (total_cost, total_sessions, total_turns): (f64, i64, i64) =
            if let Some(pid) = project_id {
                conn.query_row(agg_sql, params![days, pid], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?
            } else {
                conn.query_row(agg_sql, params![days], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?
            };

        // Query b: weekly actual (last 7 days)
        let weekly_sql = match project_id {
            Some(_) =>
                "SELECT COALESCE(SUM(total_cost_usd), 0) FROM sessions WHERE started_at > date('now', '-7 days') AND project_id = ?1",
            None =>
                "SELECT COALESCE(SUM(total_cost_usd), 0) FROM sessions WHERE started_at > date('now', '-7 days')",
        };
        let weekly_total: f64 = if let Some(pid) = project_id {
            conn.query_row(weekly_sql, params![pid], |row| row.get(0))?
        } else {
            conn.query_row(weekly_sql, params![], |row| row.get(0))?
        };

        // Query c: model breakdown with cache
        let breakdown_sql = match project_id {
            Some(_) =>
                "SELECT tu.model,
                        COALESCE(SUM(tu.estimated_cost_usd), 0),
                        COALESCE(SUM(tu.input_tokens), 0),
                        COALESCE(SUM(tu.output_tokens), 0),
                        COALESCE(SUM(tu.cache_read_input_tokens), 0),
                        COUNT(DISTINCT turns.session_id)
                 FROM token_usage tu
                 JOIN turns ON tu.turn_id = turns.id
                 JOIN sessions s ON turns.session_id = s.id
                 WHERE s.started_at > date('now', '-' || ?1 || ' days') AND s.project_id = ?2
                 GROUP BY tu.model",
            None =>
                "SELECT tu.model,
                        COALESCE(SUM(tu.estimated_cost_usd), 0),
                        COALESCE(SUM(tu.input_tokens), 0),
                        COALESCE(SUM(tu.output_tokens), 0),
                        COALESCE(SUM(tu.cache_read_input_tokens), 0),
                        COUNT(DISTINCT turns.session_id)
                 FROM token_usage tu
                 JOIN turns ON tu.turn_id = turns.id
                 JOIN sessions s ON turns.session_id = s.id
                 WHERE s.started_at > date('now', '-' || ?1 || ' days')
                 GROUP BY tu.model",
        };
        // Fetch all model_pricing rows into a map so we can look them up inside
        // the loop below without re-acquiring the mutex (which would deadlock).
        let pricing_map: std::collections::HashMap<String, (f64, f64)> = {
            let mut ps = conn.prepare(
                "SELECT model_id, input, cache_read FROM model_pricing",
            )?;
            let iter = ps.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?, row.get::<_, f64>(2)?))
            })?;
            let mut map = std::collections::HashMap::new();
            for r in iter {
                let (id, inp, cr) = r?;
                map.insert(id, (inp, cr));
            }
            map
        };

        let mut stmt = conn.prepare(breakdown_sql)?;
        let mut rows = if let Some(pid) = project_id {
            stmt.query(params![days, pid])?
        } else {
            stmt.query(params![days])?
        };

        let mut by_model: Vec<ModelCostBreakdown> = Vec::new();
        let mut cache_savings_usd: f64 = 0.0;
        while let Some(row) = rows.next()? {
            let model: String = row.get(0)?;
            let cost: f64 = row.get(1)?;
            let input: i64 = row.get(2)?;
            let output: i64 = row.get(3)?;
            let cache_read: i64 = row.get(4)?;
            let session_count: i64 = row.get(5)?;

            // Use DB pricing when available; fall back to hardcoded estimates.
            let has_db_pricing = pricing_map.contains_key(&model);
            cache_savings_usd += if let Some(&(inp_rate, cr_rate)) = pricing_map.get(&model) {
                if inp_rate > cr_rate {
                    #[allow(clippy::cast_precision_loss)]
                    let tokens_m = cache_read as f64 / 1_000_000.0;
                    tokens_m * (inp_rate - cr_rate)
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let percentage = if total_cost > 0.0 {
                (cost / total_cost) * 100.0
            } else {
                0.0
            };
            let avg_cost_per_session = if session_count > 0 {
                cost / session_count as f64
            } else {
                0.0
            };

            by_model.push(ModelCostBreakdown {
                model,
                total_cost_usd: cost,
                total_input_tokens: input,
                total_output_tokens: output,
                total_cache_read_tokens: cache_read,
                session_count,
                avg_cost_per_session,
                percentage,
                pricing_source: Some(if has_db_pricing { "db" } else { "hardcoded" }.to_string()),
            });
        }

        // Query d: most expensive session
        let expensive_sql = match project_id {
            Some(_) =>
                "SELECT s.session_uuid, p.display_name, s.total_cost_usd, s.total_turns, s.model, s.started_at
                 FROM sessions s JOIN projects p ON s.project_id = p.id
                 WHERE s.started_at > date('now', '-' || ?1 || ' days') AND s.project_id = ?2
                 ORDER BY s.total_cost_usd DESC LIMIT 1",
            None =>
                "SELECT s.session_uuid, p.display_name, s.total_cost_usd, s.total_turns, s.model, s.started_at
                 FROM sessions s JOIN projects p ON s.project_id = p.id
                 WHERE s.started_at > date('now', '-' || ?1 || ' days')
                 ORDER BY s.total_cost_usd DESC LIMIT 1",
        };
        let mut exp_stmt = conn.prepare(expensive_sql)?;
        let most_expensive_session: Option<SessionCostHighlight> = {
            let mut exp_rows = if let Some(pid) = project_id {
                exp_stmt.query(params![days, pid])?
            } else {
                exp_stmt.query(params![days])?
            };
            if let Some(row) = exp_rows.next()? {
                Some(SessionCostHighlight {
                    session_uuid: row.get(0)?,
                    project_name: row.get(1)?,
                    cost_usd: row.get(2)?,
                    turns: row.get(3)?,
                    model: row.get(4)?,
                    started_at: row.get(5)?,
                })
            } else {
                None
            }
        };

        let avg_cost_per_session = if total_sessions > 0 {
            total_cost / total_sessions as f64
        } else {
            0.0
        };
        let avg_cost_per_turn = if total_turns > 0 {
            total_cost / total_turns as f64
        } else {
            0.0
        };
        let weekly_avg_cost = weekly_total / 7.0;

        let estimated_cost: f64 = by_model
            .iter()
            .filter(|m| m.pricing_source.as_deref() == Some("hardcoded"))
            .map(|m| m.total_cost_usd)
            .sum();
        let estimated_cost_portion = if total_cost > 0.0 {
            estimated_cost / total_cost
        } else {
            0.0
        };

        Ok(CostMetrics {
            total_cost_usd: total_cost,
            avg_cost_per_session,
            avg_cost_per_turn,
            weekly_avg_cost,
            total_sessions,
            total_turns,
            cache_savings_usd,
            estimated_cost_portion,
            by_model,
            most_expensive_session,
        })
    }

    fn recalculate_costs(&self) -> anyhow::Result<u64> {
        let pricing_rows = self.list_model_pricing()?;
        let pricing_map: std::collections::HashMap<String, (f64, f64, f64, f64)> = pricing_rows
            .into_iter()
            .map(|p| (p.model_id, (p.input, p.output, p.cache_write, p.cache_read)))
            .collect();

        let conn = self.lock()?;

        let mut stmt = conn.prepare(
            "SELECT tu.id, tu.model, tu.input_tokens, tu.output_tokens,
                    tu.cache_creation_input_tokens, tu.cache_read_input_tokens
             FROM token_usage tu",
        )?;
        let mut rows = stmt.query([])?;
        let mut updated: u64 = 0;

        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let model: String = row.get(1)?;
            let input: i64 = row.get(2)?;
            let output: i64 = row.get(3)?;
            let cache_creation: i64 = row.get(4)?;
            let cache_read: i64 = row.get(5)?;

            let new_cost = if let Some(&(inp, out, cw, cr)) = pricing_map.get(&model) {
                #[allow(clippy::cast_precision_loss)]
                let cost = (input as f64 / 1_000_000.0) * inp
                    + (output as f64 / 1_000_000.0) * out
                    + (cache_creation as f64 / 1_000_000.0) * cw
                    + (cache_read as f64 / 1_000_000.0) * cr;
                cost
            } else {
                crate::adapters::analytics::analysis::cost::estimate_cost(
                    &model, input, output, cache_creation, cache_read,
                )
            };

            conn.execute(
                "UPDATE token_usage SET estimated_cost_usd = ?1 WHERE id = ?2",
                params![new_cost, id],
            )?;
            updated += 1;
        }
        drop(rows);
        drop(stmt);

        // Recompute session totals from token_usage
        conn.execute(
            "UPDATE sessions SET total_cost_usd = (
                SELECT COALESCE(SUM(tu.estimated_cost_usd), 0)
                FROM token_usage tu
                JOIN turns t ON tu.turn_id = t.id
                WHERE t.session_id = sessions.id
            )",
            [],
        )?;

        Ok(updated)
    }
}

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

impl PricingStore for SqliteAnalyticsStore {
    fn upsert_model_pricing(&self, pricing: &ModelPricing) -> anyhow::Result<()> {
        self.lock()?.execute(
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
        Ok(())
    }

    fn batch_upsert_model_pricing(&self, pricings: &[ModelPricing]) -> anyhow::Result<()> {
        self.batch_upsert_model_pricing_impl(pricings)
    }

    fn get_model_pricing(&self, model_id: &str) -> anyhow::Result<Option<ModelPricing>> {
        let conn = self.lock()?;
        let result = conn
            .query_row(
                "SELECT model_id, input, output, cache_write, cache_read, source, synced_at
                 FROM model_pricing WHERE model_id = ?1",
                params![model_id],
                |row| {
                    Ok(ModelPricing {
                        model_id: row.get(0)?,
                        input: row.get(1)?,
                        output: row.get(2)?,
                        cache_write: row.get(3)?,
                        cache_read: row.get(4)?,
                        source: row.get(5)?,
                        synced_at: row.get(6)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    fn list_model_pricing(&self) -> anyhow::Result<Vec<ModelPricing>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT model_id, input, output, cache_write, cache_read, source, synced_at
             FROM model_pricing ORDER BY model_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ModelPricing {
                model_id: row.get(0)?,
                input: row.get(1)?,
                output: row.get(2)?,
                cache_write: row.get(3)?,
                cache_read: row.get(4)?,
                source: row.get(5)?,
                synced_at: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

const SCHEMA: &str = r"
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
    ingested_at       TEXT    NOT NULL DEFAULT (datetime('now'))
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
    ingested_at     TEXT    NOT NULL DEFAULT (datetime('now'))
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

#[cfg(test)]
mod tests {
    use super::*;
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
                "-Volumes-T5-projects-claudy",
                "claudy",
                Some("/Volumes/T5/projects/claudy"),
            )
            .unwrap();
        assert!(id > 0);
        let found = store
            .get_project_by_encoded_dir("-Volumes-T5-projects-claudy")
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

        let pid = store.upsert_project("-test-cost", "cost-proj", None).unwrap();
        let sid = store
            .upsert_session(&NewSession {
                session_uuid: "uuid-cost-db".into(),
                project_id: pid,
                source_file: "/cost.jsonl".into(),
                cwd: None,
                model: Some("claude-haiku-test".into()),
                first_message: None,
                started_at: Some("2026-01-01T00:00:00".into()),
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
}
