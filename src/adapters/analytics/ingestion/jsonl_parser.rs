use crate::domain::analytics::{JsonlEvent, NewSession, NewTokenUsage, NewToolCall, NewTurn};
use crate::ports::analytics_ports::{AnalyticsStore, PricingStore};
use std::io::{BufRead, BufReader};
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
}

pub fn parse_and_ingest(
    store: &dyn AnalyticsStore,
    project_id: i64,
    file_path: &Path,
    path_str: &str,
    _full: bool,
    pricing_store: Option<&dyn PricingStore>,
) -> anyhow::Result<IngestionStats> {
    let file = std::fs::File::open(file_path)?;
    let reader = BufReader::new(file);

    let session_uuid = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut stats = IngestionStats {
        sessions_created: 0,
        turns_created: 0,
        token_records_created: 0,
        tool_calls_created: 0,
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

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let event: JsonlEvent = match serde_json::from_str(trimmed) {
            Ok(e) => e,
            Err(_) => continue,
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
                    let sid = store.upsert_session(&NewSession {
                        session_uuid: session_uuid.clone(),
                        project_id,
                        source_file: path_str.to_string(),
                        cwd: session_cwd.clone(),
                        model: current_model.clone(),
                        first_message: text.as_ref().map(|t| redact_secrets(t)),
                        started_at: ts,
                    })?;
                    session_id = Some(sid);
                    stats.sessions_created += 1;
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
                let tid = store.insert_turn(&NewTurn {
                    session_id: sid,
                    turn_number,
                    prompt_text: text.as_ref().map(|t| redact_secrets(t)),
                    response_text: None,
                    model: current_model.clone(),
                    duration_ms: None,
                    started_at: ts,
                })?;
                pending_turn_id = Some(tid);
                let _ = text; // text already used in insert_turn above
                stats.turns_created += 1;
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
    }

    // Update session completion
    if let Some(sid) = session_id {
        let ended = last_timestamp.as_deref().unwrap_or("unknown");
        if let Err(e) =
            store.update_session_completion(sid, ended, turn_number, total_cost, total_duration)
        {
            tracing::warn!(error = %e, session_id = %sid, "failed to update session completion");
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
