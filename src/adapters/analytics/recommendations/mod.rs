pub mod rules;

use crate::domain::analytics::*;
use crate::ports::analytics_ports::AnalyticsStore;

pub fn generate(store: &dyn AnalyticsStore) -> anyhow::Result<Vec<Recommendation>> {
    let mut recs = Vec::new();
    recs.extend(rules::check_cost_trajectory(store)?);
    recs.extend(rules::check_cache_efficiency(store)?);
    recs.extend(rules::check_model_selection(store)?);
    recs.extend(rules::check_tool_overhead(store)?);

    store.clear_recommendations()?;
    for rec in &recs {
        store.insert_recommendation(rec)?;
    }
    Ok(recs)
}
