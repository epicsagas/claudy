use crate::domain::analytics::*;

pub trait AnalyticsStore: Send + Sync {
    fn initialize_schema(&self) -> anyhow::Result<()>;

    fn upsert_project(
        &self,
        encoded_dir: &str,
        display_name: &str,
        resolved_path: Option<&str>,
    ) -> anyhow::Result<i64>;
    fn get_project_by_encoded_dir(
        &self,
        encoded_dir: &str,
    ) -> anyhow::Result<Option<ProjectRecord>>;
    fn list_projects(&self) -> anyhow::Result<Vec<ProjectRecord>>;

    fn upsert_session(&self, session: &NewSession) -> anyhow::Result<i64>;
    fn update_session_completion(
        &self,
        session_id: i64,
        ended_at: &str,
        total_turns: i32,
        total_cost_usd: f64,
        total_duration_ms: i64,
    ) -> anyhow::Result<()>;
    fn get_sessions(
        &self,
        limit: u32,
        days: Option<u32>,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<SessionRecord>>;
    fn get_session_by_uuid(&self, uuid: &str) -> anyhow::Result<Option<SessionRecord>>;

    fn insert_turn(&self, turn: &NewTurn) -> anyhow::Result<i64>;
    fn get_turns_by_session(&self, session_id: i64) -> anyhow::Result<Vec<TurnRecord>>;

    fn insert_token_usage(&self, usage: &NewTokenUsage) -> anyhow::Result<()>;

    fn insert_tool_call(&self, call: &NewToolCall) -> anyhow::Result<()>;
    fn get_tool_calls_by_turn(&self, turn_id: i64) -> anyhow::Result<Vec<ToolCallRecord>>;

    fn insert_channel_metric(&self, record: &ChannelMetricRecord) -> anyhow::Result<()>;

    fn get_checkpoint(&self, file_path: &str) -> anyhow::Result<Option<IngestionCheckpoint>>;
    fn upsert_checkpoint(
        &self,
        file_path: &str,
        file_modified: &str,
        byte_offset: i64,
        line_count: i64,
    ) -> anyhow::Result<()>;

    fn clear_recommendations(&self) -> anyhow::Result<()>;
    fn insert_recommendation(&self, rec: &Recommendation) -> anyhow::Result<()>;
    fn get_recommendations(&self) -> anyhow::Result<Vec<Recommendation>>;

    fn aggregate_token_trends(&self, days: u32, project_id: Option<i64>) -> anyhow::Result<Vec<TokenTrendPoint>>;
    fn aggregate_tool_distribution(
        &self,
        days: Option<u32>,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<ToolDistribution>>;
    fn aggregate_dashboard_stats(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<DashboardStats>;
    fn aggregate_cost_metrics(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<CostMetrics>;
}

pub trait PricingStore: Send + Sync {
    fn upsert_model_pricing(&self, pricing: &ModelPricing) -> anyhow::Result<()>;
    fn batch_upsert_model_pricing(&self, pricings: &[ModelPricing]) -> anyhow::Result<()>;
    fn get_model_pricing(&self, model_id: &str) -> anyhow::Result<Option<ModelPricing>>;
    fn list_model_pricing(&self) -> anyhow::Result<Vec<ModelPricing>>;
}

pub trait AnalysisEngine: Send + Sync {
    fn compute_token_trends(&self, days: u32, project_id: Option<i64>) -> anyhow::Result<Vec<TokenTrendPoint>>;
    fn compute_tool_distribution(
        &self,
        days: Option<u32>,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<ToolDistribution>>;
    fn compute_cost_metrics(&self, days: u32, project_id: Option<i64>) -> anyhow::Result<CostMetrics>;
    fn compute_dashboard_stats(&self, days: u32, project_id: Option<i64>) -> anyhow::Result<DashboardStats>;
    fn compute_prompt_efficiency(&self, limit: u32) -> anyhow::Result<Vec<PromptEfficiency>>;
    fn detect_tool_patterns(&self, min_frequency: u32) -> anyhow::Result<Vec<ToolPattern>>;
    fn compare_model_performance(&self) -> anyhow::Result<Vec<ModelPerformance>>;
    fn get_session_comparisons(&self, limit: u32) -> anyhow::Result<Vec<SessionComparison>>;
}
