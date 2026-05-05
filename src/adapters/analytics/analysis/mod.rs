pub mod cost;
pub mod efficiency;
pub mod patterns;
pub mod trends;

use crate::domain::analytics::*;
use crate::ports::analytics_ports::{AnalysisEngine, AnalyticsStore};
use std::sync::Arc;

pub struct SqliteAnalysisEngine {
    store: Arc<dyn AnalyticsStore>,
}

impl SqliteAnalysisEngine {
    pub fn new(store: Arc<dyn AnalyticsStore>) -> Self {
        Self { store }
    }
}

impl AnalysisEngine for SqliteAnalysisEngine {
    fn compute_token_trends(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<TokenTrendPoint>> {
        self.store.aggregate_token_trends(days, project_id)
    }

    fn compute_tool_distribution(
        &self,
        days: Option<u32>,
        project_id: Option<i64>,
    ) -> anyhow::Result<Vec<ToolDistribution>> {
        self.store.aggregate_tool_distribution(days, project_id)
    }

    fn compute_cost_metrics(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<CostMetrics> {
        self.store.aggregate_cost_metrics(days, project_id)
    }

    fn compute_dashboard_stats(
        &self,
        days: u32,
        project_id: Option<i64>,
    ) -> anyhow::Result<DashboardStats> {
        self.store.aggregate_dashboard_stats(days, project_id)
    }

    fn compute_prompt_efficiency(&self, _limit: u32) -> anyhow::Result<Vec<PromptEfficiency>> {
        Ok(vec![])
    }

    fn detect_tool_patterns(&self, _min_frequency: u32) -> anyhow::Result<Vec<ToolPattern>> {
        Ok(vec![])
    }

    fn compare_model_performance(&self) -> anyhow::Result<Vec<ModelPerformance>> {
        Ok(vec![])
    }

    fn get_session_comparisons(&self, _limit: u32) -> anyhow::Result<Vec<SessionComparison>> {
        Ok(vec![])
    }
}
