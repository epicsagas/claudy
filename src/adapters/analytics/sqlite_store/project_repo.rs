use crate::domain::analytics::ProjectRecord;
use crate::ports::analytics_ports::AnalyticsStore;
use rusqlite::{OptionalExtension, params};

use super::SqliteAnalyticsStore;

impl AnalyticsStore for SqliteAnalyticsStore {
    fn initialize_schema(&self) -> anyhow::Result<()> {
        self.lock()?.execute_batch(super::SCHEMA)?;
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

    fn upsert_session(
        &self,
        session: &crate::domain::analytics::NewSession,
    ) -> anyhow::Result<i64> {
        crate::adapters::analytics::sqlite_store::session_repo::upsert_session_impl(self, session)
    }

    fn update_session_completion(
        &self,
        session_id: i64,
        ended_at: &str,
        total_turns: i32,
        total_cost_usd: f64,
        total_duration_ms: i64,
    ) -> anyhow::Result<()> {
        crate::adapters::analytics::sqlite_store::session_repo::update_session_completion_impl(
            self,
            session_id,
            ended_at,
            total_turns,
            total_cost_usd,
            total_duration_ms,
        )
    }

    fn get_sessions(
        &self,
        limit: u32,
        days: Option<u32>,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<crate::domain::analytics::SessionRecord>> {
        crate::adapters::analytics::sqlite_store::session_repo::get_sessions_impl(
            self, limit, days, project_id,
        )
    }

    fn get_session_by_uuid(
        &self,
        uuid: &str,
    ) -> anyhow::Result<Option<crate::domain::analytics::SessionRecord>> {
        crate::adapters::analytics::sqlite_store::session_repo::get_session_by_uuid_impl(self, uuid)
    }

    fn insert_turn(&self, turn: &crate::domain::analytics::NewTurn) -> anyhow::Result<i64> {
        crate::adapters::analytics::sqlite_store::session_repo::insert_turn_impl(self, turn)
    }

    fn get_turns_by_session(
        &self,
        session_id: i64,
    ) -> anyhow::Result<Vec<crate::domain::analytics::TurnRecord>> {
        crate::adapters::analytics::sqlite_store::session_repo::get_turns_by_session_impl(
            self, session_id,
        )
    }

    fn insert_token_usage(
        &self,
        usage: &crate::domain::analytics::NewTokenUsage,
    ) -> anyhow::Result<()> {
        crate::adapters::analytics::sqlite_store::session_repo::insert_token_usage_impl(self, usage)
    }

    fn insert_tool_call(&self, call: &crate::domain::analytics::NewToolCall) -> anyhow::Result<()> {
        crate::adapters::analytics::sqlite_store::session_repo::insert_tool_call_impl(self, call)
    }

    fn update_tool_call_result(
        &self,
        tool_use_id: &str,
        is_error: bool,
        result_summary: Option<&str>,
    ) -> anyhow::Result<()> {
        crate::adapters::analytics::sqlite_store::session_repo::update_tool_call_result_impl(
            self,
            tool_use_id,
            is_error,
            result_summary,
        )
    }

    fn get_tool_calls_by_turn(
        &self,
        turn_id: i64,
    ) -> anyhow::Result<Vec<crate::domain::analytics::ToolCallRecord>> {
        crate::adapters::analytics::sqlite_store::session_repo::get_tool_calls_by_turn_impl(
            self, turn_id,
        )
    }

    fn insert_channel_metric(
        &self,
        record: &crate::domain::analytics::ChannelMetricRecord,
    ) -> anyhow::Result<()> {
        crate::adapters::analytics::sqlite_store::session_repo::insert_channel_metric_impl(
            self, record,
        )
    }

    fn get_checkpoint(
        &self,
        file_path: &str,
    ) -> anyhow::Result<Option<crate::domain::analytics::IngestionCheckpoint>> {
        crate::adapters::analytics::sqlite_store::session_repo::get_checkpoint_impl(self, file_path)
    }

    fn upsert_checkpoint(
        &self,
        file_path: &str,
        file_modified: &str,
        byte_offset: i64,
        line_count: i64,
    ) -> anyhow::Result<()> {
        crate::adapters::analytics::sqlite_store::session_repo::upsert_checkpoint_impl(
            self,
            file_path,
            file_modified,
            byte_offset,
            line_count,
        )
    }

    fn clear_recommendations(&self) -> anyhow::Result<()> {
        crate::adapters::analytics::sqlite_store::session_repo::clear_recommendations_impl(self)
    }

    fn insert_recommendation(
        &self,
        rec: &crate::domain::analytics::Recommendation,
    ) -> anyhow::Result<()> {
        crate::adapters::analytics::sqlite_store::session_repo::insert_recommendation_impl(
            self, rec,
        )
    }

    fn get_recommendations(&self) -> anyhow::Result<Vec<crate::domain::analytics::Recommendation>> {
        crate::adapters::analytics::sqlite_store::session_repo::get_recommendations_impl(self)
    }

    fn aggregate_token_trends(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<crate::domain::analytics::TokenTrendPoint>> {
        crate::adapters::analytics::sqlite_store::analytics_queries::aggregate_token_trends_impl(
            self, days, project_id,
        )
    }

    fn aggregate_tool_distribution(
        &self,
        days: Option<u32>,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<crate::domain::analytics::ToolDistribution>> {
        crate::adapters::analytics::sqlite_store::analytics_queries::aggregate_tool_distribution_impl(
            self, days, project_id,
        )
    }

    fn aggregate_dashboard_stats(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<crate::domain::analytics::DashboardStats> {
        crate::adapters::analytics::sqlite_store::analytics_queries::aggregate_dashboard_stats_impl(
            self, days, project_id,
        )
    }

    fn aggregate_cost_metrics(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<crate::domain::analytics::CostMetrics> {
        crate::adapters::analytics::sqlite_store::analytics_queries::aggregate_cost_metrics_impl(
            self, days, project_id,
        )
    }

    fn recalculate_costs(&self) -> anyhow::Result<u64> {
        crate::adapters::analytics::sqlite_store::pricing_repo::recalculate_costs_impl(self)
    }
}
