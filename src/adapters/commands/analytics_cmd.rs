use crate::domain::commands::AnalyticsAction;
use crate::domain::context::Context;
use crate::ports::analytics_ports::AnalyticsStore;

pub fn run_analytics(ctx: &mut Context, action: AnalyticsAction) -> anyhow::Result<i32> {
    match action {
        AnalyticsAction::Dashboard => run_dashboard(ctx),
        AnalyticsAction::Ingest { full, project } => run_ingest(ctx, full, project.as_deref()),
        AnalyticsAction::Recommend => run_recommend(ctx),
        AnalyticsAction::Export {
            format,
            project,
            days,
        } => run_export(ctx, &format, project.as_deref(), days),
        AnalyticsAction::SyncPricing => run_sync_pricing(ctx),
    }
}

fn run_dashboard(ctx: &mut Context) -> anyhow::Result<i32> {
    #[cfg(feature = "analytics-ui")]
    {
        ctx.output.info("Launching analytics dashboard...");
        crate::adapters::analytics::tauri::launch_dashboard(&ctx.paths)
    }
    #[cfg(not(feature = "analytics-ui"))]
    {
        ctx.output
            .warn("Analytics dashboard requires --features analytics-ui build");
        ctx.output
            .info("Use 'claudy analytics ingest' for CLI-only analytics");
        Ok(1)
    }
}

fn run_ingest(ctx: &mut Context, full: bool, project: Option<&str>) -> anyhow::Result<i32> {
    let db_path = &ctx.paths.analytics_db;

    // Auto-trigger pricing sync before ingestion; network errors are warnings only.
    let cache_path = dirs::home_dir()
        .map(|h| h.join(".claudy").join("cache").join("models_dev.json"));
    if let Some(ref cp) = cache_path {
        let store_result =
            crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)
                .and_then(|s| s.initialize_schema().map(|_| s));
        match store_result {
            Ok(store) => {
                match crate::adapters::analytics::pricing::sync::run_pricing_sync(&store, cp) {
                    Ok(result) => {
                        for w in &result.warnings {
                            ctx.output.warn(&format!("pricing sync: {w}"));
                        }
                        ctx.output.info(&format!(
                            "Pricing sync: {} models synced (source: {})",
                            result.models_synced,
                            result.source.label(),
                        ));
                    }
                    Err(e) => {
                        ctx.output.warn(&format!("Pricing sync skipped: {e}"));
                    }
                }
            }
            Err(e) => {
                ctx.output.warn(&format!("Pricing sync skipped (db init failed): {e}"));
            }
        }
    }

    ctx.output.info("Starting ingestion...");

    let result = crate::adapters::analytics::ingestion::run_ingestion(db_path, full, project)?;

    ctx.output.info(&format!(
        "Ingestion complete in {}ms: {} files scanned, {} ingested | {} sessions, {} turns, {} token records, {} tool calls",
        result.elapsed_ms,
        result.files_scanned,
        result.files_ingested,
        result.sessions_created,
        result.turns_created,
        result.token_records_created,
        result.tool_calls_created,
    ));
    Ok(0)
}

fn run_recommend(ctx: &mut Context) -> anyhow::Result<i32> {
    let db_path = &ctx.paths.analytics_db;
    let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)?;
    store.initialize_schema()?;

    let recs = crate::adapters::analytics::recommendations::generate(&store)?;

    if recs.is_empty() {
        ctx.output.info(
            "No recommendations at this time. Run 'claudy analytics ingest' first to collect data.",
        );
        return Ok(0);
    }

    ctx.output.info(&format!("{} recommendations:", recs.len()));
    for (i, rec) in recs.iter().enumerate() {
        let severity = match &rec.severity {
            crate::domain::analytics::Severity::Info => "INFO",
            crate::domain::analytics::Severity::Warning => "WARN",
            crate::domain::analytics::Severity::Critical => "CRIT",
        };
        ctx.output.info(&format!(
            "  {}. [{}] {} — {}",
            i + 1,
            severity,
            rec.title,
            rec.description,
        ));
        if let Some(action) = &rec.action {
            ctx.output.info(&format!("     Action: {}", action));
        }
    }
    Ok(0)
}

fn run_export(
    ctx: &mut Context,
    format: &str,
    project: Option<&str>,
    _days: u32,
) -> anyhow::Result<i32> {
    let db_path = &ctx.paths.analytics_db;
    let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)?;

    let project_id = project
        .map(|p| store.get_project_by_encoded_dir(p))
        .transpose()?
        .flatten()
        .map(|p| p.id);

    let sessions = store.get_sessions(1000, None, project_id)?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&sessions)?;
            println!("{}", json);
        }
        "csv" => {
            println!(
                "session_uuid,project_id,cwd,model,started_at,ended_at,total_turns,total_cost_usd,total_duration_ms"
            );
            for s in &sessions {
                println!(
                    "{},{},{},{},{},{},{},{},{}",
                    s.session_uuid,
                    s.project_id,
                    s.cwd.as_deref().unwrap_or(""),
                    s.model.as_deref().unwrap_or(""),
                    s.started_at.as_deref().unwrap_or(""),
                    s.ended_at.as_deref().unwrap_or(""),
                    s.total_turns,
                    s.total_cost_usd,
                    s.total_duration_ms,
                );
            }
        }
        _ => {
            ctx.output.warn(&format!(
                "Unknown format '{}'. Use 'json' or 'csv'.",
                format
            ));
            return Ok(1);
        }
    }
    Ok(0)
}

fn run_sync_pricing(ctx: &mut Context) -> anyhow::Result<i32> {
    let db_path = &ctx.paths.analytics_db;
    let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)?;
    store.initialize_schema()?;

    let cache_path = dirs::home_dir()
        .map(|h| h.join(".claudy").join("cache").join("models_dev.json"))
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let result =
        crate::adapters::analytics::pricing::sync::run_pricing_sync(&store, &cache_path)?;

    for warning in &result.warnings {
        println!("Warning: {warning}");
    }

    println!(
        "Pricing sync complete: {} models synced (source: {})",
        result.models_synced,
        result.source.label(),
    );

    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::adapters::analytics::pricing::sync::run_pricing_sync;
    use crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore;
    use crate::ports::analytics_ports::{AnalyticsStore, PricingStore};
    use tempfile::{NamedTempFile, TempDir};

    fn write_models_dev_cache(dir: &TempDir) -> std::path::PathBuf {
        let cache_path = dir.path().join("models_dev.json");
        let json = r#"[{"id":"claude-haiku-4-5","name":"Claude Haiku 4.5","cost":{"input":0.80,"output":4.00,"cache_read":0.08,"cache_write":1.00}}]"#;
        std::fs::write(&cache_path, json).unwrap();
        cache_path
    }

    /// AC7: ingest auto-triggers sync-pricing; result is visible through the store.
    #[test]
    fn test_ingest_auto_triggers_pricing_sync_logs_result() {
        let tmp_dir = TempDir::new().unwrap();
        let cache_path = write_models_dev_cache(&tmp_dir);
        let db_file = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db_file.path().to_str().unwrap()).unwrap();
        store.initialize_schema().unwrap();

        let result = run_pricing_sync(&store, &cache_path).unwrap();

        assert!(result.models_synced >= 1, "at least one model should be synced");
        assert!(result.warnings.is_empty(), "no warnings expected with valid cache");

        let rows = store.list_model_pricing().unwrap();
        assert!(!rows.is_empty(), "model_pricing table must not be empty after auto-sync");
    }
}
