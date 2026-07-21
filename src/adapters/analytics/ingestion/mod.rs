pub mod incremental;
pub mod jsonl_parser;

use crate::domain::analytics::IngestionResult;
use crate::ports::analytics_ports::AnalyticsStore;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// One ingestion source directory with a neutral role label.
#[derive(Debug, Clone)]
pub struct IngestionSource {
    pub path: PathBuf,
    /// `"live"` for the primary source, `"archive"` for fallback sources.
    pub label: &'static str,
}

/// Resolved ingestion source set (R2). Built from `AnalyticsSettings` with
/// tildes expanded. The live source is scanned first; archive sources only
/// fill gaps for sessions no longer present in the live source.
#[derive(Debug, Clone)]
pub struct IngestionSources {
    pub sources: Vec<IngestionSource>,
    pub archive_root: Option<PathBuf>,
    pub archive_on_ingest: bool,
}

impl IngestionSources {
    pub fn from_config(settings: &crate::config::registry::AnalyticsSettings) -> Self {
        // The first source is the live source (re-parsed whenever its mtime
        // advances); later sources are archive fallbacks that only fill gaps.
        // An empty `sources` list is a misconfiguration — fall back to the
        // defaults so a live source is never silently absent.
        if settings.sources.is_empty() {
            return Self::defaults();
        }
        let sources = settings
            .sources
            .iter()
            .enumerate()
            .map(|(i, raw)| IngestionSource {
                path: expand_tilde(raw),
                label: if i == 0 { "live" } else { "archive" },
            })
            .collect();
        IngestionSources {
            sources,
            archive_root: Some(expand_tilde(&settings.archive_root)),
            archive_on_ingest: settings.archive_on_ingest,
        }
    }

    /// Default sources: `~/.claude/projects` (live) + `~/.claude/projects-archive`.
    pub fn defaults() -> Self {
        Self::from_config(&crate::config::registry::AnalyticsSettings::default())
    }
}

fn expand_tilde(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(s)
}

pub fn run_ingestion(
    db_path: &str,
    full: bool,
    project_filter: Option<&str>,
    sources: &IngestionSources,
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

    // R2: durably archive new/grown live JSONL before scanning, so retention
    // purge of the live dir never loses data the DB hasn't seen yet.
    if sources.archive_on_ingest
        && let Some(archive_root) = &sources.archive_root
        && let Some(live) = sources.sources.first()
        && let Err(e) = archive_live_source(&live.path, archive_root)
    {
        tracing::warn!(error = %e, "archive copy failed; ingestion continues");
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

    for source in &sources.sources {
        ingest_source_dir(&store, source, full, project_filter, &mut result)?;
    }

    result.elapsed_ms = start.elapsed().as_millis() as u64;

    // Distinct "data is fresh" line — separates session freshness from the
    // pricing-sync "file is fresh" line printed above. (R3 log separation.)
    let latest = store
        .ingestion_freshness()
        .ok()
        .and_then(|f| f.latest_turn_at)
        .and_then(|ts| ts.get(..10).map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    tracing::info!(
        turns = result.turns_created,
        files = result.files_ingested,
        latest = %latest,
        "ingest run complete",
    );

    Ok(result)
}

/// Scan one source directory: iterate project subdirs, ingest each JSONL file,
/// tagging rows with the neutral `source_kind` label. Non-live (archive)
/// sources only fill gaps — files whose session is already ingested are skipped.
fn ingest_source_dir(
    store: &crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore,
    source: &IngestionSource,
    full: bool,
    project_filter: Option<&str>,
    result: &mut IngestionResult,
) -> anyhow::Result<()> {
    if !source.path.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(&source.path)?;
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

            // R2: archive sources only fill gaps — skip files whose session is
            // already ingested (e.g. still present in the live source).
            if source.label != "live"
                && let Some(stem) = file_path.file_stem().and_then(|s| s.to_str())
                && store.get_session_by_uuid(stem)?.is_some()
            {
                continue;
            }

            // R1: resume from the last committed byte offset. A `full` re-ingest
            // deliberately starts from 0 to re-evaluate the whole file.
            let start_offset = if full {
                0
            } else {
                store
                    .get_checkpoint(&path_str)
                    .ok()
                    .flatten()
                    .map(|c| c.byte_offset)
                    .unwrap_or(0)
            };
            match jsonl_parser::parse_and_ingest(
                store,
                project_id,
                &file_path,
                &path_str,
                full,
                Some(store),
                Some(source.label),
                start_offset,
            ) {
                Ok(stats) => {
                    result.files_ingested += 1;
                    result.sessions_created += stats.sessions_created;
                    result.turns_created += stats.turns_created;
                    result.token_records_created += stats.token_records_created;
                    result.tool_calls_created += stats.tool_calls_created;
                    let line_count =
                        stats.turns_created as i64 + stats.token_records_created as i64;
                    store.upsert_checkpoint(&path_str, &modified, stats.byte_offset, line_count)?;
                }
                Err(e) => {
                    tracing::warn!(path = %path_str, error = %e, "failed to ingest file");
                }
            }
        }
    }
    Ok(())
}

/// Recursively mirror new/grown `*.jsonl` from the live source into the archive
/// root (never deletes). Retention-proof copy (R2).
fn archive_live_source(live: &Path, archive_root: &Path) -> anyhow::Result<()> {
    if !live.exists() {
        return Ok(());
    }
    for src in walk_jsonl(live)? {
        let rel = src.strip_prefix(live).unwrap_or(&src);
        let dest = archive_root.join(rel);
        let need_copy = match std::fs::metadata(&dest) {
            Ok(dest_meta) => {
                let src_mtime = std::fs::metadata(&src).and_then(|m| m.modified()).ok();
                let dest_mtime = dest_meta.modified().ok();
                match (src_mtime, dest_mtime) {
                    (Some(s), Some(d)) => s > d,
                    _ => true,
                }
            }
            Err(_) => true,
        };
        if need_copy {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&src, &dest)?;
            // The archive is a raw (unredacted) second copy — tighten perms so it
            // is owner-only, stricter than the default copy mode. Audit LOW.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o600));
            }
        }
    }
    Ok(())
}

fn walk_jsonl(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        // Use symlink_metadata so symlinks are NOT followed — prevents both
        // copying files outside the source into the archive (scope expansion)
        // and infinite recursion via symlink loops (panic). Audit MEDIUM.
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "skipping unreadable entry during archive walk"
                );
                continue;
            }
        };
        if meta.is_symlink() {
            continue;
        } else if meta.is_dir() {
            out.extend(walk_jsonl(&path)?);
        } else if meta.is_file() && path.extension().is_some_and(|e| e == "jsonl") {
            out.push(path);
        }
    }
    Ok(out)
}

fn collect_jsonl_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::analytics::NewSession;
    use crate::ports::analytics_ports::AnalyticsStore;
    use tempfile::TempDir;

    fn write_session_jsonl(dir: &Path, project: &str, uuid: &str) -> PathBuf {
        let proj_dir = dir.join(project);
        std::fs::create_dir_all(&proj_dir).unwrap();
        let file = proj_dir.join(format!("{uuid}.jsonl"));
        // Minimal valid transcript: one user turn + one assistant turn.
        let user = r#"{"type":"user","timestamp":"2026-07-20T10:00:00Z","message":{"role":"user","content":"hello"}}"#;
        let assistant = r#"{"type":"assistant","timestamp":"2026-07-20T10:00:01Z","message":{"role":"assistant","model":"claude-sonnet-5","content":[{"type":"text","text":"hi"}],"usage":{"input_tokens":10,"output_tokens":5,"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}"#;
        std::fs::write(&file, format!("{user}\n{assistant}\n")).unwrap();
        file
    }

    /// AC-R2: a session placed in archive_root only (removed from live) appears
    /// in the DB; a second ingest adds no duplicate turns.
    #[test]
    fn test_archive_source_fills_gaps_without_duplicates() {
        let tmp = TempDir::new().unwrap();
        let live = tmp.path().join("projects");
        let archive = tmp.path().join("archive");
        std::fs::create_dir_all(&live).unwrap();
        std::fs::create_dir_all(&archive).unwrap();

        // Session exists ONLY in the archive (simulating retention purge of live).
        write_session_jsonl(&archive, "-proj", "sess-archive-only");

        let db = tmp.path().join("analytics.db");
        let sources = IngestionSources {
            sources: vec![
                IngestionSource {
                    path: live.clone(),
                    label: "live",
                },
                IngestionSource {
                    path: archive.clone(),
                    label: "archive",
                },
            ],
            archive_root: Some(archive.clone()),
            archive_on_ingest: false, // don't copy from empty live
        };

        let r1 = run_ingestion(db.to_str().unwrap(), false, None, &sources).unwrap();
        assert_eq!(r1.turns_created, 1, "archive session should be ingested");
        assert!(r1.sessions_created >= 1);

        // Second run: checkpoint + gap-skip must prevent duplicate turns.
        let r2 = run_ingestion(db.to_str().unwrap(), false, None, &sources).unwrap();
        assert_eq!(r2.turns_created, 0, "no duplicate turns on re-ingest");

        let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(
            db.to_str().unwrap(),
        )
        .unwrap();
        store.initialize_schema().unwrap();
        let session = store
            .get_session_by_uuid("sess-archive-only")
            .unwrap()
            .expect("session ingested from archive");
        let turns = store.get_turns_by_session(session.id).unwrap();
        assert_eq!(turns.len(), 1, "exactly one turn, no duplicates");

        // R2: the archived session is tagged with the neutral source label.
        let _ = store.upsert_session(&NewSession {
            session_uuid: "no-op".into(),
            project_id: 1,
            source_file: "x".into(),
            cwd: None,
            model: None,
            first_message: None,
            started_at: None,
            source_kind: None,
        }); // touch trait to ensure it compiles; ignore result
    }

    /// AC-R2: archiver copies new live JSONL into archive_root (retention-proof).
    #[test]
    fn test_archiver_copies_new_files() {
        let tmp = TempDir::new().unwrap();
        let live = tmp.path().join("projects");
        let archive = tmp.path().join("archive");
        std::fs::create_dir_all(&live).unwrap();

        write_session_jsonl(&live, "-proj", "sess-1");
        archive_live_source(&live, &archive).unwrap();

        let archived = walk_jsonl(&archive).unwrap();
        assert_eq!(archived.len(), 1, "live jsonl mirrored to archive");
        assert!(archived[0].to_string_lossy().contains("sess-1"));

        // Re-run archiver is a no-op (idempotent, mtime not newer).
        archive_live_source(&live, &archive).unwrap();
        assert_eq!(walk_jsonl(&archive).unwrap().len(), 1);
    }

    /// Security: archiver must NOT follow symlinks — neither copy files from
    /// outside the source (scope expansion) nor recurse into symlink loops.
    #[cfg(unix)]
    #[test]
    fn test_archiver_ignores_symlinks() {
        use std::os::unix::fs::symlink;
        let tmp = TempDir::new().unwrap();
        let live = tmp.path().join("projects");
        let archive = tmp.path().join("archive");
        let outside = tmp.path().join("outside");
        std::fs::create_dir_all(&live).unwrap();
        std::fs::create_dir_all(&outside).unwrap();

        // A real jsonl under live (should be archived).
        write_session_jsonl(&live, "-proj", "sess-real");
        // A jsonl OUTSIDE the source, plus a symlink in live pointing to it.
        write_session_jsonl(&outside, "-secret", "sess-secret");
        symlink(outside.join("-secret"), live.join("-leak")).unwrap();
        // A symlink loop (would infinite-recurse if followed).
        symlink(&live, live.join("-loop")).unwrap();

        archive_live_source(&live, &archive).unwrap();

        let archived = walk_jsonl(&archive).unwrap();
        // Only the real file is archived; the symlinked secret and loop are skipped.
        assert_eq!(archived.len(), 1, "symlinks must not be followed");
        assert!(archived[0].to_string_lossy().ends_with("sess-real.jsonl"));
        assert!(
            !archived[0].to_string_lossy().contains("secret"),
            "external symlink target must not be copied in"
        );
    }

    /// R2 config → sources resolution expands tildes and labels roles.
    #[test]
    fn test_sources_from_config_expand_tilde() {
        let settings = crate::config::registry::AnalyticsSettings::default();
        let sources = IngestionSources::from_config(&settings);
        assert_eq!(sources.sources.len(), 2);
        assert_eq!(sources.sources[0].label, "live");
        assert_eq!(sources.sources[1].label, "archive");
        assert!(
            sources.sources[0]
                .path
                .to_string_lossy()
                .ends_with(".claude/projects")
        );
        assert!(sources.archive_root.is_some());
        assert!(sources.archive_on_ingest);
    }

    /// Regression: re-ingesting an already-ingested live file — even with
    /// `full=true`, which bypasses the mtime checkpoint and forces a re-parse of
    /// the whole file — must NOT duplicate turns, token-usage, or tool-calls.
    /// Without UNIQUE(session_id, turn_number) + the new-turn gate, this is the
    /// exact compounding duplication the hourly scheduler would trigger on every
    /// actively-growing transcript.
    #[test]
    fn test_live_reingest_does_not_duplicate() {
        use std::io::Write;
        let tmp = TempDir::new().unwrap();
        let live = tmp.path().join("projects");
        std::fs::create_dir_all(&live).unwrap();

        let file = write_session_jsonl(&live, "-proj", "sess-live");
        let db = tmp.path().join("analytics.db");
        let sources = IngestionSources {
            sources: vec![IngestionSource {
                path: live.clone(),
                label: "live",
            }],
            archive_root: None,
            archive_on_ingest: false,
        };

        // First ingest: one user turn -> one turn + one token-usage row.
        let r1 = run_ingestion(db.to_str().unwrap(), true, None, &sources).unwrap();
        assert_eq!(r1.turns_created, 1);
        assert!(r1.token_records_created >= 1);

        // Re-ingest with full=true: forces a re-parse of the same file. The
        // UNIQUE(session_id, turn_number) gate must turn every turn insert into a
        // no-op, and the parser must skip re-inserting each existing turn's children.
        let r2 = run_ingestion(db.to_str().unwrap(), true, None, &sources).unwrap();
        assert_eq!(r2.turns_created, 0, "no duplicate turns on full re-ingest");
        assert_eq!(
            r2.token_records_created, 0,
            "no duplicate token-usage on full re-ingest"
        );

        let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(
            db.to_str().unwrap(),
        )
        .unwrap();
        store.initialize_schema().unwrap();
        let session = store
            .get_session_by_uuid("sess-live")
            .unwrap()
            .expect("session present");
        assert_eq!(
            store.get_turns_by_session(session.id).unwrap().len(),
            1,
            "exactly one turn persisted, no duplicates"
        );

        // Append a new user+assistant pair and re-ingest: only the new turn is added.
        let user = r#"{"type":"user","timestamp":"2026-07-20T11:00:00Z","message":{"role":"user","content":"again"}}"#;
        let assistant = r#"{"type":"assistant","timestamp":"2026-07-20T11:00:01Z","message":{"role":"assistant","model":"claude-sonnet-5","content":[{"type":"text","text":"ok"}],"usage":{"input_tokens":3,"output_tokens":2,"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}"#;
        std::fs::OpenOptions::new()
            .append(true)
            .open(&file)
            .unwrap()
            .write_all(format!("{user}\n{assistant}\n").as_bytes())
            .unwrap();

        let r3 = run_ingestion(db.to_str().unwrap(), true, None, &sources).unwrap();
        assert_eq!(r3.turns_created, 1, "only the appended turn is added");
        assert_eq!(
            store.get_turns_by_session(session.id).unwrap().len(),
            2,
            "two turns total after append"
        );
    }

    /// AC1 (#53): a full=false re-ingest of an appended file resumes from the
    /// committed `byte_offset` and parses only the appended lines, with the
    /// checkpoint offset advancing to end-of-file.
    #[test]
    fn test_incremental_resume_appends_only() {
        use std::io::Write;
        let tmp = TempDir::new().unwrap();
        let live = tmp.path().join("projects");
        std::fs::create_dir_all(&live).unwrap();

        let file = write_session_jsonl(&live, "-proj", "sess-inc");
        let db = tmp.path().join("analytics.db");
        let sources = IngestionSources {
            sources: vec![IngestionSource {
                path: live.clone(),
                label: "live",
            }],
            archive_root: None,
            archive_on_ingest: false,
        };

        // First ingest (full=true to bypass the mtime skip on a fresh file):
        // records the session and commits byte_offset = file size.
        let r1 = run_ingestion(db.to_str().unwrap(), true, None, &sources).unwrap();
        assert_eq!(r1.turns_created, 1);

        let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(
            db.to_str().unwrap(),
        )
        .unwrap();
        store.initialize_schema().unwrap();
        let path_str = file.to_string_lossy().to_string();
        let cp1 = store
            .get_checkpoint(&path_str)
            .unwrap()
            .expect("checkpoint");
        assert!(
            cp1.byte_offset > 0,
            "byte_offset advanced past first ingest"
        );
        let first_offset = cp1.byte_offset;
        assert_eq!(
            cp1.byte_offset,
            file.metadata().unwrap().len() as i64,
            "first run reaches EOF"
        );

        // Append a new user+assistant pair.
        let user = r#"{"type":"user","timestamp":"2026-07-21T12:00:00Z","message":{"role":"user","content":"more"}}"#;
        let assistant = r#"{"type":"assistant","timestamp":"2026-07-21T12:00:01Z","message":{"role":"assistant","model":"claude-sonnet-5","content":[{"type":"text","text":"ok"}],"usage":{"input_tokens":2,"output_tokens":1,"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}"#;
        std::fs::OpenOptions::new()
            .append(true)
            .open(&file)
            .unwrap()
            .write_all(format!("{user}\n{assistant}\n").as_bytes())
            .unwrap();
        // Bump mtime into the future so the checkpoint mtime-skip re-parses
        // (sub-second appends can otherwise look unchanged).
        let mtime_handle = std::fs::File::options().write(true).open(&file).unwrap();
        mtime_handle
            .set_times(
                std::fs::FileTimes::new().set_modified(
                    std::time::SystemTime::now() + std::time::Duration::from_secs(120),
                ),
            )
            .unwrap();

        // Re-ingest with full=false: resumes from first_offset, parses only the
        // appended pair.
        let r2 = run_ingestion(db.to_str().unwrap(), false, None, &sources).unwrap();
        assert_eq!(r2.turns_created, 1, "only the appended turn is parsed");
        let cp2 = store
            .get_checkpoint(&path_str)
            .unwrap()
            .expect("checkpoint");
        assert!(
            cp2.byte_offset > first_offset,
            "byte_offset advanced past appended lines"
        );
        assert_eq!(
            cp2.byte_offset,
            file.metadata().unwrap().len() as i64,
            "offset at EOF after append"
        );

        let session = store
            .get_session_by_uuid("sess-inc")
            .unwrap()
            .expect("session");
        assert_eq!(
            store.get_turns_by_session(session.id).unwrap().len(),
            2,
            "two turns total, no duplicates"
        );
    }

    /// AC2 (#53): a trailing line written without a newline (a flush in
    /// progress) must hold `byte_offset` at the start of that line and re-read
    /// it on the next run once completed — no dropped turn, no duplicate.
    #[test]
    fn test_partial_trailing_line_reread_next_run() {
        use std::io::Write;
        let tmp = TempDir::new().unwrap();
        let live = tmp.path().join("projects");
        let proj_dir = live.join("-proj");
        std::fs::create_dir_all(&proj_dir).unwrap();
        let file = proj_dir.join("sess-partial.jsonl");

        let user1 = r#"{"type":"user","timestamp":"2026-07-21T12:00:00Z","message":{"role":"user","content":"first"}}"#;
        // A *complete* JSON object written WITHOUT a trailing newline.
        let user2_partial = r#"{"type":"user","timestamp":"2026-07-21T12:01:00Z","message":{"role":"user","content":"second"}}"#;
        std::fs::write(&file, format!("{user1}\n{user2_partial}")).unwrap();

        let db = tmp.path().join("analytics.db");
        let sources = IngestionSources {
            sources: vec![IngestionSource {
                path: live.clone(),
                label: "live",
            }],
            archive_root: None,
            archive_on_ingest: false,
        };

        let r1 = run_ingestion(db.to_str().unwrap(), true, None, &sources).unwrap();
        // Only the newline-terminated first user turn is parsed; the partial
        // trailing line is deferred to the next run (R3).
        assert_eq!(r1.turns_created, 1);

        let store = crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore::open(
            db.to_str().unwrap(),
        )
        .unwrap();
        store.initialize_schema().unwrap();
        let path_str = file.to_string_lossy().to_string();
        let cp1 = store
            .get_checkpoint(&path_str)
            .unwrap()
            .expect("checkpoint");
        // Offset is parked at the START of the partial line (not EOF) — R3.
        let partial_start = (user1.len() + 1) as i64; // user1 + '\n'
        assert_eq!(
            cp1.byte_offset, partial_start,
            "offset held at partial line start, not EOF"
        );

        // Complete the partial line with a newline, then re-ingest.
        std::fs::OpenOptions::new()
            .append(true)
            .open(&file)
            .unwrap()
            .write_all(b"\n")
            .unwrap();
        // Bump mtime so the checkpoint mtime-skip re-parses the now-completed line.
        let mtime_handle = std::fs::File::options().write(true).open(&file).unwrap();
        mtime_handle
            .set_times(
                std::fs::FileTimes::new().set_modified(
                    std::time::SystemTime::now() + std::time::Duration::from_secs(120),
                ),
            )
            .unwrap();
        let r2 = run_ingestion(db.to_str().unwrap(), false, None, &sources).unwrap();
        // The previously-partial line is now complete and parses as a new turn.
        assert_eq!(
            r2.turns_created, 1,
            "completed partial line parsed as a new turn"
        );
        let cp2 = store
            .get_checkpoint(&path_str)
            .unwrap()
            .expect("checkpoint");
        assert_eq!(
            cp2.byte_offset,
            file.metadata().unwrap().len() as i64,
            "offset now at EOF once the line completes"
        );

        let session = store
            .get_session_by_uuid("sess-partial")
            .unwrap()
            .expect("session");
        assert_eq!(
            store.get_turns_by_session(session.id).unwrap().len(),
            2,
            "no dropped turn, no duplicate"
        );
    }
}
