use crate::domain::analytics::{JsonlEvent, NewSession, NewTokenUsage, NewToolCall, NewTurn};
use crate::ports::analytics_ports::{AnalyticsStore, PricingStore};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let boundary = s.ceil_char_boundary(max);
        s[..boundary].to_string()
    }
}

fn redact_secrets(s: &str) -> String {
    let patterns = [
        ("sk-ant-api", "***REDACTED***"),
        ("ghp_", "***REDACTED***"),
        ("AKIA", "***REDACTED***"),
        ("xoxb-", "***REDACTED***"),
        ("xoxp-", "***REDACTED***"),
        ("xoxa-", "***REDACTED***"),
        ("xoxs-", "***REDACTED***"),
    ];
    let mut result = s.to_string();
    for (pat, repl) in patterns {
        if let Some(idx) = result.find(pat) {
            let end = (idx + 40).min(result.len());
            result.replace_range(idx..end, repl);
        }
    }
    result
}

pub struct IngestionStats {
    pub sessions_created: u32,
    pub turns_created: u32,
    pub token_records_created: u32,
    pub tool_calls_created: u32,
    /// Byte offset to resume from on the next ingest of this file (R1/R3).
    /// Only fully-read, newline-terminated lines advance it; a trailing partial
    /// line is left for the next run to re-read.
    pub byte_offset: i64,
}

/// Per-file inputs to [`parse_and_ingest`], grouped to keep the function's
/// argument count under clippy's threshold now that R1 added `start_byte_offset`.
pub struct IngestFileArgs<'a> {
    pub project_id: i64,
    pub file_path: &'a Path,
    pub path_str: &'a str,
    pub full: bool,
    pub source_kind: Option<&'a str>,
    pub start_byte_offset: i64,
}

pub fn parse_and_ingest(
    store: &dyn AnalyticsStore,
    pricing_store: Option<&dyn PricingStore>,
    args: IngestFileArgs<'_>,
) -> anyhow::Result<IngestionStats> {
    let IngestFileArgs {
        project_id,
        file_path,
        path_str,
        full,
        source_kind,
        start_byte_offset,
    } = args;

    // R1: resume from the last committed byte offset. Clamp to file length so a
    // file that shrank since the last ingest re-parses from a safe point.
    let mut file = std::fs::File::open(file_path)?;
    let file_len = file.metadata()?.len() as i64;
    let start = start_byte_offset.clamp(0, file_len) as u64;
    file.seek(SeekFrom::Start(start))?;
    let mut reader = BufReader::new(&mut file);

    let session_uuid = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut cursor: u64 = start;
    let mut stats = IngestionStats {
        sessions_created: 0,
        turns_created: 0,
        token_records_created: 0,
        tool_calls_created: 0,
        // Default to the *clamped* resume point, not the raw requested offset:
        // if the file shrank since the last ingest, persisting the unclamped
        // offset would leave the checkpoint stuck past EOF forever (a later
        // grow-back would then resume too far in and skip the gap). Only
        // fully-read, newline-terminated lines advance it further (R3).
        byte_offset: start as i64,
    };

    let mut turn_number: i32 = 0;
    let mut current_model: Option<String> = None;
    let mut session_cwd: Option<String> = None;
    let mut session_id: Option<i64> = None;
    let mut first_message: Option<String> = None;
    let mut total_cost: f64 = 0.0;
    let mut total_duration: i64 = 0;
    let mut last_timestamp: Option<String> = None;
    let mut first_timestamp: Option<String> = None;
    let mut pending_turn_id: Option<i64> = None;

    let mut buf = Vec::new();
    loop {
        buf.clear();
        let n = reader.read_until(b'\n', &mut buf)?;
        if n == 0 {
            break;
        }
        let ended_with_newline = buf.last() == Some(&b'\n');
        if !ended_with_newline {
            // Partial trailing line (a flush in progress): do NOT process it
            // this run. Leave byte_offset at the start of this line so the
            // next run re-reads it whole (R3) — never silently drop a line.
            break;
        }
        let line = String::from_utf8_lossy(&buf).into_owned();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            cursor += n as u64;
            stats.byte_offset = cursor as i64;
            continue;
        }

        let event: JsonlEvent = match serde_json::from_str(trimmed) {
            Ok(e) => e,
            Err(_) => {
                // Malformed JSON on a full line: skip it but advance so we
                // don't re-parse it forever.
                cursor += n as u64;
                stats.byte_offset = cursor as i64;
                continue;
            }
        };

        match event.event_type.as_str() {
            "user" => {
                let is_meta = event.is_meta.unwrap_or(false);
                let is_command = event
                    .message
                    .as_ref()
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .is_some_and(|c| c.starts_with('/'));
                // R4: neutral "author" flag derived purely from transcript
                // metadata the parser already inspects. Meta/command injections
                // are skipped below, so persisted turns are human-authored by
                // construction; this records that invariant explicitly.
                let human_authored = !(is_meta || is_command);

                if is_meta || is_command {
                    continue;
                }

                let text = extract_text_from_message(&event.message);
                if text.is_none() {
                    continue;
                }

                if session_id.is_none() {
                    let ts = event
                        .timestamp
                        .as_deref()
                        .or(first_timestamp.as_deref())
                        .map(std::string::ToString::to_string);
                    let sid = match store.upsert_session(&NewSession {
                        session_uuid: session_uuid.clone(),
                        project_id,
                        source_file: path_str.to_string(),
                        cwd: session_cwd.clone(),
                        model: current_model.clone(),
                        first_message: text.as_ref().map(|t| redact_secrets(t)),
                        started_at: ts,
                        source_kind: source_kind.map(|s| s.to_string()),
                    }) {
                        Ok(sid) => sid,
                        Err(e) => {
                            // Without a session row every downstream turn/token/tool
                            // insert would FK-violate. Abort just this file with what
                            // we have so far; the caller logs the file and continues
                            // to the next one instead of aborting the whole run.
                            tracing::warn!(
                                error = %e,
                                %session_uuid,
                                source_file = path_str,
                                "failed to create session; skipping file",
                            );
                            return Ok(stats);
                        }
                    };
                    session_id = Some(sid);
                    stats.sessions_created += 1;
                    if !full {
                        // Incremental resume: continue numbering after the turns
                        // already stored for this session so appended turns don't
                        // collide with turn_number 1 and get gated away (R1/#53).
                        turn_number = store.get_turn_count(sid)? as i32;
                        // Seed the running cost/duration totals from the session's
                        // previously-persisted values. Only this run's appended
                        // lines are parsed, so without this the end-of-function
                        // `update_session_completion` call would overwrite the
                        // session's true totals with just the appended portion.
                        // A "result" event later in this run (if any) still wins,
                        // since it assigns the authoritative cumulative value.
                        if let Some(existing) = store.get_session_by_uuid(&session_uuid)? {
                            total_cost = existing.total_cost_usd;
                            total_duration = existing.total_duration_ms;
                        }
                    }
                }

                if first_message.is_none() && text.is_some() {
                    first_message = text.as_ref().map(|t| redact_secrets(t));
                }

                turn_number += 1;
                let ts = event
                    .timestamp
                    .as_deref()
                    .map(std::string::ToString::to_string);
                let sid = session_id
                    .ok_or_else(|| anyhow::anyhow!("session_id not set before turn insertion"))?;
                match store.insert_turn(&NewTurn {
                    session_id: sid,
                    turn_number,
                    prompt_text: text.as_ref().map(|t| redact_secrets(t)),
                    response_text: None,
                    model: current_model.clone(),
                    duration_ms: None,
                    started_at: ts,
                    human_authored,
                }) {
                    Ok(Some(tid)) => {
                        // New turn — remember its id so the assistant/tool_result
                        // events that follow attach token-usage and tool-calls to it.
                        pending_turn_id = Some(tid);
                        stats.turns_created += 1;
                    }
                    Ok(None) => {
                        // Already ingested in a prior run (UNIQUE(session_id,
                        // turn_number) conflict on a re-parsed file). Drop the
                        // pending id so this turn's children are NOT re-inserted.
                        pending_turn_id = None;
                    }
                    Err(e) => {
                        // Skip just this turn's token usage / tool calls and
                        // continue with the next event, instead of letting a single
                        // insert failure (e.g. a transient FK violation) abort the
                        // whole ingestion run. Following assistant blocks see
                        // pending_turn_id == None and already skip their inserts.
                        tracing::warn!(
                            error = %e,
                            %sid,
                            turn_number,
                            "failed to insert turn; skipping turn"
                        );
                        pending_turn_id = None;
                    }
                }
                let _ = text; // text already used in insert_turn above
            }
            "assistant" => {
                if let Some(msg) = &event.message {
                    if let Some(model) = msg.get("model").and_then(|m| m.as_str()) {
                        current_model = Some(model.to_string());
                    }

                    if let Some(usage) = msg.get("usage") {
                        let input = usage
                            .get("input_tokens")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let output = usage
                            .get("output_tokens")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let cache_creation = usage
                            .get("cache_creation_input_tokens")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let cache_read = usage
                            .get("cache_read_input_tokens")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let model_str = current_model.as_deref().unwrap_or("unknown");
                        let cost = if let Some(ps) = pricing_store {
                            crate::adapters::analytics::analysis::cost::estimate_cost_with_store(
                                ps,
                                model_str,
                                input,
                                output,
                                cache_creation,
                                cache_read,
                            )
                        } else {
                            crate::adapters::analytics::analysis::cost::estimate_cost(
                                model_str,
                                input,
                                output,
                                cache_creation,
                                cache_read,
                            )
                        };
                        total_cost += cost;

                        if let Some(tid) = pending_turn_id {
                            if let Err(e) = store.insert_token_usage(&NewTokenUsage {
                                turn_id: tid,
                                model: model_str.to_string(),
                                input_tokens: input,
                                output_tokens: output,
                                cache_creation_input_tokens: cache_creation,
                                cache_read_input_tokens: cache_read,
                                estimated_cost_usd: cost,
                            }) {
                                tracing::warn!(error = %e, "failed to insert token usage");
                            } else {
                                stats.token_records_created += 1;
                            }
                        }
                    }

                    // Extract tool_use content blocks
                    if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                        for block in content {
                            if block.get("type").is_some_and(|t| t == "tool_use") {
                                let tool_name = block
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("unknown");
                                let tool_id =
                                    block.get("id").and_then(|i| i.as_str()).unwrap_or_default();
                                let input_summary = block
                                    .get("input")
                                    .map(|i| truncate_str(&i.to_string(), 500));

                                if let Some(tid) = pending_turn_id {
                                    if let Err(e) = store.insert_tool_call(&NewToolCall {
                                        turn_id: tid,
                                        tool_use_id: tool_id.to_string(),
                                        tool_name: tool_name.to_string(),
                                        input_summary: input_summary.clone(),
                                        is_error: false,
                                        result_summary: None,
                                        duration_ms: None,
                                    }) {
                                        tracing::warn!(error = %e, tool = %tool_name, "failed to insert tool call");
                                    } else {
                                        stats.tool_calls_created += 1;
                                    }
                                }
                            }
                        }
                    }

                    // Extract response text for turn
                    if let (Some(_tid), Some(content)) = (pending_turn_id, msg.get("content")) {
                        let _response_text = extract_response_text(content);
                    }
                }
            }
            "tool_result" => {
                if let Some(msg) = &event.message {
                    let is_error = msg
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let result_summary = msg.get("content").and_then(|c| {
                        c.as_str().map(|s| truncate_str(s, 500)).or_else(|| {
                            c.as_array().map(|arr| {
                                let text: Vec<String> = arr
                                    .iter()
                                    .filter_map(|b| {
                                        b.get("text")
                                            .and_then(|t| t.as_str())
                                            .map(std::string::ToString::to_string)
                                    })
                                    .collect();
                                text.join("\n")
                            })
                        })
                    });
                    let tool_use_id =
                        msg.get("tool_use_id").and_then(|v| v.as_str()).or_else(|| {
                            msg.get("content")
                                .and_then(|c| c.as_array())
                                .and_then(|arr| {
                                    arr.iter()
                                        .find_map(|b| b.get("tool_use_id").and_then(|v| v.as_str()))
                                })
                        });
                    if let Some(tuid) = tool_use_id
                        && !tuid.is_empty()
                        && let Err(e) =
                            store.update_tool_call_result(tuid, is_error, result_summary.as_deref())
                    {
                        tracing::warn!(error = %e, tool_use_id = %tuid, "failed to update tool call result");
                    }
                }
            }
            "result" => {
                if let Some(cost) = event.cost_usd {
                    total_cost = cost;
                }
                if let Some(dur) = event.duration_ms {
                    total_duration = dur;
                }
                if let Some(cwd) = event.cwd {
                    session_cwd = Some(cwd);
                }
            }
            _ => {}
        }

        if event.timestamp.is_some() {
            last_timestamp = event.timestamp.clone();
            if first_timestamp.is_none() {
                first_timestamp = event.timestamp.clone();
            }
        }

        cursor += n as u64;
        stats.byte_offset = cursor as i64;
    }

    // Update session completion
    if let Some(sid) = session_id {
        let ended = last_timestamp.as_deref().unwrap_or("unknown");
        if let Err(e) =
            store.update_session_completion(sid, ended, turn_number, total_cost, total_duration)
        {
            tracing::warn!(error = %e, session_id = %sid, "failed to update session completion");
        }

        // R4: best-effort backfill of any turns whose model is NULL (e.g. the
        // opening turn created before the first assistant event set the model)
        // from the session model captured during parsing.
        if let Some(model) = current_model.as_deref()
            && let Err(e) = store.backfill_null_turn_models(sid, model)
        {
            tracing::warn!(error = %e, session_id = %sid, "failed to backfill turn models");
        }
    }

    Ok(stats)
}

fn extract_text_from_message(message: &Option<serde_json::Value>) -> Option<String> {
    let msg = message.as_ref()?;
    let content = msg.get("content")?;

    if let Some(s) = content.as_str() {
        return Some(truncate_str(s, 500));
    }

    if let Some(arr) = content.as_array() {
        let texts: Vec<String> = arr
            .iter()
            .filter(|b| b.get("type").is_some_and(|t| t == "text"))
            .filter_map(|b| {
                b.get("text")
                    .and_then(|t| t.as_str())
                    .map(std::string::ToString::to_string)
            })
            .collect();
        if !texts.is_empty() {
            return Some(truncate_str(&texts.join("\n"), 500));
        }
    }

    None
}

fn extract_response_text(content: &serde_json::Value) -> Option<String> {
    if let Some(s) = content.as_str() {
        return Some(truncate_str(s, 500));
    }

    if let Some(arr) = content.as_array() {
        let texts: Vec<String> = arr
            .iter()
            .filter(|b| b.get("type").is_some_and(|t| t == "text"))
            .filter_map(|b| {
                b.get("text")
                    .and_then(|t| t.as_str())
                    .map(std::string::ToString::to_string)
            })
            .collect();
        if !texts.is_empty() {
            return Some(truncate_str(&texts.join("\n"), 500));
        }
    }

    None
}
