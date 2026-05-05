pub mod commands;

use crate::config::layout::AppPaths;

pub fn launch_dashboard(_paths: &AppPaths) -> anyhow::Result<i32> {
    #[cfg(feature = "analytics-ui")]
    {
        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![
                commands::get_sessions,
                commands::get_token_trends,
                commands::get_tool_distribution,
                commands::get_cost_metrics,
                commands::get_dashboard_stats,
                commands::get_recommendations,
                commands::trigger_ingestion,
                commands::get_model_comparison,
                commands::get_projects,
                commands::get_config,
                commands::update_config,
            ])
            .run(tauri::generate_context!("tauri.conf.json"))
            .map_err(|e| anyhow::anyhow!("dashboard error: {}", e))?;
        Ok(0)
    }
    #[cfg(not(feature = "analytics-ui"))]
    {
        Ok(1)
    }
}
