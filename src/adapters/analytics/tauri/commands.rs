use crate::domain::analytics::*;
use crate::ports::analytics_ports::{AnalysisEngine, AnalyticsStore};
use serde::Serialize;
use tauri::Manager;

pub struct AppState {
    db_path: String,
}

impl AppState {
    fn open_store(
        &self,
    ) -> anyhow::Result<crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore> {
        let store =
            crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(&self.db_path)?;
        store.initialize_schema()?;
        Ok(store)
    }
}

fn get_state(app_handle: &tauri::AppHandle) -> AppState {
    // Always use the same DB as CLI commands (~/.claudy/analytics/analytics.db)
    // so the dashboard sees ingested data and synced pricing.
    let db_path = dirs::home_dir()
        .map(|h| {
            h.join(".claudy")
                .join("analytics")
                .join("analytics.db")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|| {
            app_handle
                .path()
                .app_data_dir()
                .map(|p: std::path::PathBuf| p.join("analytics.db").to_string_lossy().to_string())
                .unwrap_or_default()
        });
    AppState { db_path }
}

#[derive(Serialize)]
pub struct SessionSummary {
    pub session_uuid: String,
    pub project_name: String,
    pub model: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub total_turns: i32,
    pub total_cost_usd: f64,
    pub total_duration_ms: i64,
    pub first_message: Option<String>,
}

#[tauri::command]
pub fn get_sessions(
    app_handle: tauri::AppHandle,
    limit: Option<u32>,
    days: Option<u32>,
    project: Option<String>,
) -> Result<Vec<SessionSummary>, String> {
    let state = get_state(&app_handle);
    let store = state.open_store().map_err(|e| e.to_string())?;

    let projects = store.list_projects().map_err(|e| e.to_string())?;
    let project_map: std::collections::HashMap<i64, String> = projects
        .into_iter()
        .map(|p| (p.id, p.display_name))
        .collect();

    let project_id = project
        .map(|p| store.get_project_by_encoded_dir(&p))
        .transpose()
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|p| p.id);
    let capped = limit.unwrap_or(50).min(1000);
    let sessions = store
        .get_sessions(capped, days, project_id)
        .map_err(|e| e.to_string())?;
    Ok(sessions
        .into_iter()
        .map(|s| SessionSummary {
            session_uuid: s.session_uuid,
            project_name: project_map.get(&s.project_id).cloned().unwrap_or_default(),
            model: s.model,
            started_at: s.started_at,
            ended_at: s.ended_at,
            total_turns: s.total_turns,
            total_cost_usd: s.total_cost_usd,
            total_duration_ms: s.total_duration_ms,
            first_message: s.first_message,
        })
        .collect())
}

#[tauri::command]
pub fn get_token_trends(
    app_handle: tauri::AppHandle,
    days: Option<u32>,
    project: Option<String>,
) -> Result<Vec<TokenTrendPoint>, String> {
    let state = get_state(&app_handle);
    let store = state.open_store().map_err(|e| e.to_string())?;
    let project_id = project
        .map(|p| store.get_project_by_encoded_dir(&p))
        .transpose()
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|p| p.id);

    let engine =
        crate::adapters::analytics::analysis::SqliteAnalysisEngine::new(std::sync::Arc::new(store));
    engine
        .compute_token_trends(days.unwrap_or(30), project_id)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub fn get_tool_distribution(
    app_handle: tauri::AppHandle,
    days: Option<u32>,
    project: Option<String>,
) -> Result<Vec<ToolDistribution>, String> {
    let state = get_state(&app_handle);
    let store = state.open_store().map_err(|e| e.to_string())?;
    let project_id = project
        .map(|p| store.get_project_by_encoded_dir(&p))
        .transpose()
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|p| p.id);

    let engine =
        crate::adapters::analytics::analysis::SqliteAnalysisEngine::new(std::sync::Arc::new(store));
    engine
        .compute_tool_distribution(days, project_id)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub fn get_cost_metrics(
    app_handle: tauri::AppHandle,
    days: Option<u32>,
    project: Option<String>,
) -> Result<CostMetrics, String> {
    let state = get_state(&app_handle);
    let store = state.open_store().map_err(|e| e.to_string())?;
    let project_id = project
        .map(|p| store.get_project_by_encoded_dir(&p))
        .transpose()
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|p| p.id);

    let engine =
        crate::adapters::analytics::analysis::SqliteAnalysisEngine::new(std::sync::Arc::new(store));
    engine
        .compute_cost_metrics(days.unwrap_or(30), project_id)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub fn get_dashboard_stats(
    app_handle: tauri::AppHandle,
    days: Option<u32>,
    project: Option<String>,
) -> Result<DashboardStats, String> {
    let state = get_state(&app_handle);
    let store = state.open_store().map_err(|e| e.to_string())?;
    let project_id = project
        .map(|p| store.get_project_by_encoded_dir(&p))
        .transpose()
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|p| p.id);

    let engine =
        crate::adapters::analytics::analysis::SqliteAnalysisEngine::new(std::sync::Arc::new(store));
    engine
        .compute_dashboard_stats(days.unwrap_or(30), project_id)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub fn get_recommendations(app_handle: tauri::AppHandle) -> Result<Vec<Recommendation>, String> {
    let state = get_state(&app_handle);
    let store = state.open_store().map_err(|e| e.to_string())?;
    store.get_recommendations().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn trigger_ingestion(
    app_handle: tauri::AppHandle,
    full: Option<bool>,
) -> Result<IngestionResult, String> {
    let state = get_state(&app_handle);

    // Sync pricing before ingestion so cost estimates use current rates.
    if let Ok(store) = state.open_store() {
        let cache_path = dirs::home_dir()
            .map(|h| h.join(".claudy").join("cache").join("models_dev.json"))
            .unwrap_or_default();
        let _ = crate::adapters::analytics::pricing::sync::run_pricing_sync(&store, &cache_path);
    }

    let result = crate::adapters::analytics::ingestion::run_ingestion(
        &state.db_path,
        full.unwrap_or(false),
        None,
    )
    .map_err(|e| e.to_string())?;

    // Generate recommendations from the newly ingested data
    if let Ok(store) = state.open_store() {
        let _ = crate::adapters::analytics::recommendations::generate(&store);
    }

    Ok(result)
}

#[tauri::command]
pub fn get_model_comparison(app_handle: tauri::AppHandle) -> Result<Vec<ModelPerformance>, String> {
    let _state = get_state(&app_handle);
    Ok(vec![])
}

#[tauri::command]
pub fn get_projects(app_handle: tauri::AppHandle) -> Result<Vec<ProjectRecord>, String> {
    let state = get_state(&app_handle);
    let store = state.open_store().map_err(|e| e.to_string())?;
    store.list_projects().map_err(|e| e.to_string())
}
