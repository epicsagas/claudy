pub mod incremental;
pub mod jsonl_parser;

use crate::domain::analytics::IngestionResult;
use crate::ports::analytics_ports::AnalyticsStore;
use std::path::Path;
use std::time::Instant;

pub fn run_ingestion(
    db_path: &str,
    full: bool,
    project_filter: Option<&str>,
) -> anyhow::Result<IngestionResult> {
    let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(db_path)?;
    store.initialize_schema()?;

    // Auto-trigger pricing sync before scanning JSONL files
    let cache_path = dirs::home_dir()
        .map(|h| h.join(".claudy").join("cache").join("models_dev.json"))
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    match crate::adapters::analytics::pricing::sync::run_pricing_sync(&store, &cache_path) {
        Ok(result) => {
            for warning in &result.warnings {
                eprintln!("[pricing] warning: {warning}");
            }
            eprintln!(
                "[pricing] synced {} models (source: {})",
                result.models_synced,
                result.source.label(),
            );
        }
        Err(e) => {
            eprintln!("[pricing] sync failed (ingestion continues): {e}");
        }
    }

    let claude_projects_dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("projects"))
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    if !claude_projects_dir.exists() {
        return Ok(IngestionResult {
            files_scanned: 0,
            files_ingested: 0,
            sessions_created: 0,
            turns_created: 0,
            token_records_created: 0,
            tool_calls_created: 0,
            elapsed_ms: 0,
        });
    }

    let start = Instant::now();
    let mut result = IngestionResult {
        files_scanned: 0,
        files_ingested: 0,
        sessions_created: 0,
        turns_created: 0,
        token_records_created: 0,
        tool_calls_created: 0,
        elapsed_ms: 0,
    };

    let entries = std::fs::read_dir(&claude_projects_dir)?;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let encoded_dir = entry.file_name().to_string_lossy().to_string();

        if let Some(filter) = project_filter
            && !encoded_dir.contains(filter)
            && !encoded_dir.to_lowercase().contains(&filter.to_lowercase())
        {
            continue;
        }

        let display_name = decode_project_name(&encoded_dir);
        let resolved_path = decode_encoded_dir(&encoded_dir);
        let resolved: Option<&str> = resolved_path.as_str().into();
        let project_id = store.upsert_project(&encoded_dir, &display_name, resolved)?;

        let jsonl_files = collect_jsonl_files(&entry.path())?;
        for file_path in jsonl_files {
            result.files_scanned += 1;
            let path_str = file_path.to_string_lossy().to_string();
            let modified = file_metadata(&file_path);

            if !full
                && let Some(cp) = store.get_checkpoint(&path_str)?
                && cp.file_modified == modified
            {
                continue;
            }

            match jsonl_parser::parse_and_ingest(&store, project_id, &file_path, &path_str, full, Some(&store)) {
                Ok(stats) => {
                    result.files_ingested += 1;
                    result.sessions_created += stats.sessions_created;
                    result.turns_created += stats.turns_created;
                    result.token_records_created += stats.token_records_created;
                    result.tool_calls_created += stats.tool_calls_created;
                    let line_count =
                        stats.turns_created as i64 + stats.token_records_created as i64;
                    store.upsert_checkpoint(&path_str, &modified, 0, line_count)?;
                }
                Err(e) => {
                    tracing::warn!(path = %path_str, error = %e, "failed to ingest file");
                }
            }
        }
    }

    result.elapsed_ms = start.elapsed().as_millis() as u64;
    Ok(result)
}

fn collect_jsonl_files(dir: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "jsonl") {
            files.push(path);
        }
    }
    // Also check subagents/ subdirectory
    let subagents = dir.join("subagents");
    if subagents.exists() {
        let entries = std::fs::read_dir(&subagents)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn decode_project_name(encoded: &str) -> String {
    let decoded = encoded.replace('-', "/");
    let name = decoded.rsplit('/').next().unwrap_or(encoded);
    name.to_string()
}

fn decode_encoded_dir(encoded: &str) -> String {
    encoded.replace('-', "/")
}

fn file_metadata(path: &Path) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.to_rfc3339()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}
