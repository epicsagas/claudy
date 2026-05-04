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
