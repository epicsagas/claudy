use crate::domain::analytics::*;
use crate::ports::analytics_ports::AnalyticsStore;

pub fn check_cost_trajectory(store: &dyn AnalyticsStore) -> anyhow::Result<Vec<Recommendation>> {
    let sessions = store.get_sessions(10, None, None)?;
    let mut recs = Vec::new();

    let high_cost_sessions = sessions.iter().filter(|s| s.total_cost_usd > 0.5).count();
    if high_cost_sessions > 0 {
        recs.push(Recommendation {
            category: RecommendationCategory::CostOptimization,
            severity: Severity::Warning,
            title: "High Cost Sessions Detected".into(),
            description: format!(
                "You have {} recent sessions costing over $0.50. Consider using Haiku for simpler tasks.",
                high_cost_sessions
            ),
            action: Some("Try switching to haiku or similar model".into()),
        });
    }

    Ok(recs)
}

pub fn check_cache_efficiency(store: &dyn AnalyticsStore) -> anyhow::Result<Vec<Recommendation>> {
    let sessions = store.get_sessions(20, None, None)?;
    if sessions.is_empty() {
        return Ok(vec![]);
    }

    let total_cost: f64 = sessions.iter().map(|s| s.total_cost_usd).sum();
    if total_cost > 1.0 {
        // Simple logic for now
        return Ok(vec![Recommendation {
            category: RecommendationCategory::PromptEfficiency,
            severity: Severity::Info,
            title: "Cache Utilization".into(),
            description: "Your recent usage suggests prompt caching could save you ~30% in costs."
                .into(),
            action: Some("Review your prompt structure for static vs dynamic content".into()),
        }]);
    }
    Ok(vec![])
}

pub fn check_model_selection(_store: &dyn AnalyticsStore) -> anyhow::Result<Vec<Recommendation>> {
    Ok(vec![])
}

pub fn check_tool_overhead(store: &dyn AnalyticsStore) -> anyhow::Result<Vec<Recommendation>> {
    let sessions = store.get_sessions(10, None, None)?;
    let high_tool_use = sessions.iter().filter(|s| s.total_turns > 15).count();

    if high_tool_use > 0 {
        Ok(vec![Recommendation {
            category: RecommendationCategory::ToolUsage,
            severity: Severity::Info,
            title: "Long Conversations".into(),
            description: format!(
                "{} of your sessions have >15 turns. Summarizing context periodically can improve performance.",
                high_tool_use
            ),
            action: Some("Try a fresh session for new sub-tasks".into()),
        }])
    } else {
        Ok(vec![])
    }
}
