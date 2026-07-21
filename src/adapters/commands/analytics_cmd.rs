use crate::domain::analytics::{
    InsightsCacheEfficiency, InsightsCostAnalysis, InsightsDailyCost, InsightsOverview,
    InsightsPeriod, InsightsSummary, SessionCostHighlight,
};
use crate::domain::commands::{AnalyticsAction, ScheduleAction};
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
        AnalyticsAction::Insights {
            days,
            from,
            to,
            project,
        } => run_insights(
            ctx,
            days,
            from.as_deref(),
            to.as_deref(),
            project.as_deref(),
        ),
        AnalyticsAction::Recalculate => run_recalculate(ctx),
        AnalyticsAction::Status { stale_days, json } => run_status(ctx, stale_days, json),
        AnalyticsAction::Schedule { action } => run_schedule(ctx, action),
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
    match dirs::home_dir() {
        None => {
            ctx.output
                .warn("Pricing sync skipped: cannot determine home directory");
        }
        Some(home) => {
            let cp = home.join(".claudy").join("cache").join("models_dev.json");
            let store_result =
                crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)
                    .and_then(|s| s.initialize_schema().map(|_| s));
            match store_result {
                Ok(store) => {
                    match crate::adapters::analytics::pricing::sync::run_pricing_sync(&store, &cp) {
                        Ok(result) => {
                            for w in &result.warnings {
                                ctx.output.warn(&format!("Pricing sync: {w}"));
                            }
                            ctx.output.info(&format!(
                                "Pricing sync: {} models synced (source: {})",
                                result.models_synced,
                                result.source.label(),
                            ));
                        }
                        Err(_) => {
                            ctx.output
                                .warn("Pricing sync skipped: could not fetch pricing data");
                        }
                    }
                }
                Err(_) => {
                    ctx.output
                        .warn("Pricing sync skipped: analytics store unavailable");
                }
            }
        }
    }

    ctx.output.info("Starting ingestion...");

    let sources =
        crate::adapters::analytics::ingestion::IngestionSources::from_config(&ctx.config.analytics);
    let result =
        crate::adapters::analytics::ingestion::run_ingestion(db_path, full, project, &sources)?;

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

fn run_recalculate(ctx: &mut Context) -> anyhow::Result<i32> {
    let db_path = &ctx.paths.analytics_db;
    ctx.output.info("Syncing pricing data...");
    let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)?;
    store.initialize_schema()?;

    let cache_path = std::path::Path::new(&ctx.paths.cache_dir).join("models_dev.json");
    crate::adapters::analytics::pricing::sync::run_pricing_sync(&store, &cache_path)?;

    ctx.output
        .info("Recalculating costs with updated pricing...");
    let updated = store.recalculate_costs()?;

    ctx.output.info(&format!(
        "Recalculated {} token usage records and updated session totals.",
        updated
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

    let result = crate::adapters::analytics::pricing::sync::run_pricing_sync(&store, &cache_path)?;

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

fn run_insights(
    ctx: &mut Context,
    days: u32,
    from: Option<&str>,
    to: Option<&str>,
    project: Option<&str>,
) -> anyhow::Result<i32> {
    use chrono::{Local, NaiveDate, TimeDelta};

    let (effective_days, from_str, to_str) = match (from, to) {
        (Some(f), Some(t)) => {
            let from_date = NaiveDate::parse_from_str(f, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid --from date: {e}. Use YYYY-MM-DD."))?;
            let to_date = NaiveDate::parse_from_str(t, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid --to date: {e}. Use YYYY-MM-DD."))?;
            let diff = (to_date - from_date).num_days();
            if diff <= 0 {
                anyhow::bail!("--from must be before --to");
            }
            (diff as u32, f.to_string(), t.to_string())
        }
        (Some(_), None) | (None, Some(_)) => {
            anyhow::bail!("Both --from and --to must be provided together, or use --days");
        }
        (None, None) => {
            let today = Local::now().date_naive();
            let from_date = today - TimeDelta::days(days as i64);
            (
                days,
                from_date.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string(),
            )
        }
    };

    let db_path = &ctx.paths.analytics_db;
    let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)?;
    store.initialize_schema()?;

    let project_id = project
        .map(|p| store.get_project_by_encoded_dir(p))
        .transpose()?
        .flatten()
        .map(|p| p.id);

    let dashboard = store.aggregate_dashboard_stats(effective_days, project_id)?;
    let cost_metrics = store.aggregate_cost_metrics(effective_days, project_id)?;
    let token_trends = store.aggregate_token_trends(effective_days, project_id)?;
    let tool_dist = store.aggregate_tool_distribution(Some(effective_days), project_id)?;
    // Previously-stubbed aggregations, now backed by real SQL (multi-provider).
    let prompt_efficiency = store.aggregate_prompt_efficiency(20)?;
    let tool_patterns = store.detect_tool_patterns(3)?;
    let model_performance = store.compare_model_performance()?;
    let session_comparisons = store.aggregate_session_comparisons(20)?;

    let daily_costs: Vec<InsightsDailyCost> = token_trends
        .into_iter()
        .map(|p| InsightsDailyCost {
            date: p.date,
            cost_usd: p.total_cost_usd,
            sessions: p.session_count,
            model: p.model,
        })
        .collect();

    let mut top_tools = tool_dist;
    top_tools.sort_by_key(|b| std::cmp::Reverse(b.call_count));
    top_tools.truncate(10);

    let notable_sessions: Vec<SessionCostHighlight> = {
        let all_sessions = store.get_sessions(100, Some(effective_days), project_id)?;
        let mut sorted: Vec<_> = all_sessions.into_iter().collect();
        sorted.sort_by(|a, b| {
            b.total_cost_usd
                .partial_cmp(&a.total_cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let projects = store.list_projects()?;
        let proj_map: std::collections::HashMap<i64, String> = projects
            .into_iter()
            .map(|p| (p.id, p.display_name))
            .collect();

        sorted
            .into_iter()
            .take(5)
            .map(|s| SessionCostHighlight {
                session_uuid: s.session_uuid,
                project_name: proj_map.get(&s.project_id).cloned().unwrap_or_default(),
                cost_usd: s.total_cost_usd,
                turns: s.total_turns,
                model: s.model,
                started_at: s.started_at,
            })
            .collect()
    };

    let summary = InsightsSummary {
        period: InsightsPeriod {
            from: from_str,
            to: to_str,
            days: effective_days,
        },
        overview: InsightsOverview {
            total_sessions: dashboard.total_sessions,
            total_cost_usd: dashboard.total_cost_usd,
            total_turns: dashboard.total_turns,
            avg_tokens_per_session: dashboard.avg_tokens_per_session,
            most_used_model: dashboard.most_used_model,
        },
        daily_costs,
        model_distribution: cost_metrics.by_model,
        tool_usage: top_tools,
        notable_sessions,
        cost_analysis: InsightsCostAnalysis {
            total_cost_usd: cost_metrics.total_cost_usd,
            avg_cost_per_session: cost_metrics.avg_cost_per_session,
            avg_cost_per_turn: cost_metrics.avg_cost_per_turn,
            weekly_avg_cost: cost_metrics.weekly_avg_cost,
            cache_savings_usd: cost_metrics.cache_savings_usd,
            estimated_cost_portion: cost_metrics.estimated_cost_portion,
        },
        cache_efficiency: InsightsCacheEfficiency {
            hit_ratio: dashboard.cache_hit_ratio,
            savings_usd: cost_metrics.cache_savings_usd,
        },
        prompt_efficiency,
        tool_patterns,
        model_performance,
        session_comparisons,
    };

    let json = serde_json::to_string(&summary)?;
    println!("{json}");
    Ok(0)
}

/// `analytics status` — ingestion freshness / staleness alarm (R3).
/// Wording stays claudy-local ("sessions recorded"); no downstream terms.
fn run_status(ctx: &mut Context, stale_days: i64, json: bool) -> anyhow::Result<i32> {
    use chrono::{NaiveDate, Utc};

    let db_path = &ctx.paths.analytics_db;
    let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)?;
    store.initialize_schema()?;
    let report = store.ingestion_freshness()?;

    // days_stale derived from the leading YYYY-MM-DD of the latest turn (robust
    // to trailing time/zone variation). None when there is no data yet.
    let days_stale: Option<i64> = report
        .latest_turn_at
        .as_deref()
        .and_then(|ts| ts.get(..10))
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .map(|latest_date| (Utc::now().date_naive() - latest_date).num_days().max(0));

    // An empty DB is not "stale" — it just has no data yet. Staleness only
    // triggers when data exists but is older than the threshold.
    let stale = stale_days > 0 && days_stale.map(|d| d > stale_days).unwrap_or(false);

    if json {
        let body = serde_json::json!({
            "latest_turn_at": report.latest_turn_at,
            "total_turns": report.total_turns,
            "days_stale": days_stale,
            "stale": stale,
            "per_source": report.per_source,
        });
        println!("{body}");
    } else {
        match (&report.latest_turn_at, days_stale) {
            (Some(ts), Some(d)) => {
                let plural = if d == 1 { "" } else { "s" };
                ctx.output.info(&format!(
                    "last session recorded: {ts} ({d} day{plural} ago)"
                ));
            }
            _ => ctx.output.info("no sessions recorded yet"),
        }
        if !report.per_source.is_empty() {
            let parts: Vec<String> = report
                .per_source
                .iter()
                .map(|s| {
                    let when = s
                        .last_seen
                        .as_deref()
                        .and_then(|t| t.get(..10))
                        .unwrap_or("?");
                    format!("{} → {}", s.label, when)
                })
                .collect();
            ctx.output.info(&format!("sources: {}", parts.join(" · ")));
        }
        if stale {
            let d = days_stale.unwrap_or(0);
            ctx.output.warn(&format!(
                "STALE: no sessions recorded in {d} days — ingestion may have stopped"
            ));
        }
    }

    if stale { Ok(1) } else { Ok(0) }
}

/// `analytics schedule {install,uninstall,status}` — hourly ingestion (R1).
fn run_schedule(ctx: &mut Context, action: ScheduleAction) -> anyhow::Result<i32> {
    crate::adapters::analytics::schedule::run_schedule(ctx, action)
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

    /// Verifies the pricing-sync contract used by run_ingest's auto-trigger path:
    /// run_pricing_sync with a valid cache file must sync ≥1 model into the store (R7, AC7).
    #[test]
    fn test_pricing_sync_contract_used_by_ingest() {
        let tmp_dir = TempDir::new().unwrap();
        let cache_path = write_models_dev_cache(&tmp_dir);
        let db_file = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db_file.path().to_str().unwrap()).unwrap();
        store.initialize_schema().unwrap();

        let result = run_pricing_sync(&store, &cache_path).unwrap();

        assert!(
            result.models_synced >= 1,
            "at least one model should be synced"
        );
        assert!(
            result.warnings.is_empty(),
            "no warnings expected with valid cache"
        );

        let rows = store.list_model_pricing().unwrap();
        assert!(
            !rows.is_empty(),
            "model_pricing table must not be empty after sync"
        );
    }

    /// AC7: when run_ingest's auto-sync path succeeds, the model_pricing table is populated.
    /// Exercises the full sync → store path that run_ingest wires up internally.
    #[test]
    fn test_auto_sync_path_populates_model_pricing_table() {
        let tmp_dir = TempDir::new().unwrap();
        let cache_path = write_models_dev_cache(&tmp_dir);
        let db_file = NamedTempFile::new().unwrap();
        let store = SqliteAnalyticsStore::open(db_file.path().to_str().unwrap()).unwrap();
        store.initialize_schema().unwrap();

        run_pricing_sync(&store, &cache_path).unwrap();

        let rows = store.list_model_pricing().unwrap();
        assert!(
            !rows.is_empty(),
            "auto-sync must populate model_pricing (AC7)"
        );
        assert!(
            rows.iter().any(|r| r.model_id == "claude-haiku-4-5"),
            "synced model must appear in model_pricing table"
        );
    }
}
