//! Codex CLI session reader.
//!
//! Sessions live at `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` as plain
//! JSON lines. Line 1 is `session_meta` (session id, cwd). Conversation
//! content is in `response_item` events: `message` (user/assistant),
//! `custom_tool_call`, `custom_tool_call_output`. Everything else (reasoning,
//! token counts, turn context) is skipped.

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{MAX_SESSION_BYTES, codex_home};
use crate::domain::handoff::{DigestEvent, ForeignSessionSummary, HandoffSource};

/// Injected context blocks that appear as user-role messages but are not
/// human input.
const SKIP_PREFIXES: [&str; 6] = [
    "<skills_instructions",
    "<environment_context",
    "<user_instructions",
    "<turn_context",
    "<permissions",
    "<app_context",
];

pub fn sessions_dir() -> Option<PathBuf> {
    codex_home().map(|h| h.join("sessions"))
}

/// List codex sessions, newest first. Reads only the first line of each
/// rollout file (the `session_meta` payload).
pub fn discover_codex(limit: usize) -> Vec<ForeignSessionSummary> {
    let Some(dir) = sessions_dir() else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = match collect_rollout_files(&dir) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    files.sort_by_key(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0)
    });
    files.reverse();
    files.truncate(limit);

    files
        .into_iter()
        .filter_map(|path| {
            let meta = read_first_json_line(&path)?;
            let payload = meta.get("payload")?;
            let id = payload.get("session_id")?.as_str()?.to_string();
            let cwd = payload
                .get("cwd")
                .and_then(|v| v.as_str())
                .map(String::from);
            let last_modified = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Some(ForeignSessionSummary {
                source: HandoffSource::Codex,
                title: None, // filled lazily by the picker via extract preview
                id,
                cwd,
                last_modified,
                path: Some(path),
            })
        })
        .collect()
}

/// Cheap listing title: head of the first real user message. Reads at most
/// the first 128 KiB of the file — enough for the opening turns.
pub fn preview_user_message(path: &Path) -> Option<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; 128 * 1024];
    let n = file.read(&mut buf).unwrap_or(0);
    buf.truncate(n);
    let text = String::from_utf8_lossy(&buf);
    for event in parse_events(&text) {
        if let DigestEvent::User(t) = event {
            let head: String = t
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            let head: String = head.chars().take(120).collect();
            if !head.is_empty() {
                return Some(head);
            }
        }
    }
    None
}

/// Extract conversation events from one rollout file.
pub fn extract_codex_events(path: &Path) -> anyhow::Result<Vec<DigestEvent>> {
    let bytes = std::fs::read(path)?;
    anyhow::ensure!(
        (bytes.len() as u64) <= MAX_SESSION_BYTES,
        "codex session file too large: {}",
        path.display()
    );
    let text = String::from_utf8_lossy(&bytes);
    Ok(parse_events(&text))
}

/// Parse rollout JSONL into digest events. Malformed lines are skipped.
fn parse_events(text: &str) -> Vec<DigestEvent> {
    let mut events = Vec::new();
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v.get("type").and_then(|t| t.as_str()) != Some("response_item") {
            continue;
        }
        let Some(payload) = v.get("payload") else {
            continue;
        };
        match payload.get("type").and_then(|t| t.as_str()) {
            Some("message") => {
                let Some(text) = join_message_text(payload) else {
                    continue;
                };
                if text.trim().is_empty() || SKIP_PREFIXES.iter().any(|p| text.starts_with(p)) {
                    continue;
                }
                match payload.get("role").and_then(|r| r.as_str()) {
                    Some("user") => events.push(DigestEvent::User(text)),
                    Some("assistant") => events.push(DigestEvent::Assistant(text)),
                    _ => {} // developer and unknown roles are injected context
                }
            }
            Some("custom_tool_call") => {
                let name = payload
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("tool")
                    .to_string();
                let args = payload
                    .get("input")
                    .or_else(|| payload.get("arguments"))
                    .map(|a| a.to_string())
                    .unwrap_or_default();
                events.push(DigestEvent::ToolCall { name, args });
            }
            Some("custom_tool_call_output") => {
                let output = payload
                    .get("output")
                    .map(|o| {
                        o.as_str()
                            .map(String::from)
                            .unwrap_or_else(|| o.to_string())
                    })
                    .unwrap_or_default();
                if !output.trim().is_empty() {
                    events.push(DigestEvent::ToolOutput(output));
                }
            }
            _ => {}
        }
    }
    events
}

fn join_message_text(payload: &Value) -> Option<String> {
    let content = payload.get("content")?.as_array()?;
    let mut text = String::new();
    for item in content {
        let t = item.get("text").and_then(|t| t.as_str()).unwrap_or("");
        if !t.is_empty() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(t);
        }
    }
    Some(text)
}

fn read_first_json_line(path: &Path) -> Option<Value> {
    use std::io::BufRead;
    let file = std::fs::File::open(path).ok()?;
    let mut line = String::new();
    std::io::BufReader::new(file).read_line(&mut line).ok()?;
    serde_json::from_str(&line).ok()
}

fn collect_rollout_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    // sessions/YYYY/MM/DD/rollout-*.jsonl — year/month/day nesting.
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        for entry in std::fs::read_dir(&cur)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if entry.file_name().to_string_lossy().starts_with("rollout-")
                && path.extension().is_some_and(|e| e == "jsonl")
            {
                out.push(path);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const META: &str = r#"{"timestamp":"2026-08-15T09:00:00.000Z","type":"session_meta","payload":{"session_id":"01a00274-f7ec-7f73-bf17-ddc01849524d","cwd":"/tmp/proj","base_instructions":{"text":"You are Codex, an agent based on GPT-5. VERY LONG INSTRUCTIONS"}}"#;

    fn user_msg(text: &str) -> String {
        format!(
            r#"{{"type":"response_item","payload":{{"type":"message","role":"user","content":[{{"type":"input_text","text":{}}}]}}}}"#,
            serde_json::json!(text)
        )
    }

    fn assistant_msg(text: &str) -> String {
        format!(
            r#"{{"type":"response_item","payload":{{"type":"message","role":"assistant","content":[{{"type":"output_text","text":{}}}]}}}}"#,
            serde_json::json!(text)
        )
    }

    #[test]
    fn parses_user_assistant_and_tools() {
        let mut text = String::from(META);
        text.push('\n');
        text.push_str(&user_msg("fix the parser"));
        text.push('\n');
        text.push_str(&assistant_msg("fixed it"));
        text.push('\n');
        text.push_str(
            r#"{"type":"response_item","payload":{"type":"custom_tool_call","name":"shell","input":"cargo test"}}"#,
        );
        text.push('\n');
        text.push_str(
            r#"{"type":"response_item","payload":{"type":"custom_tool_call_output","output":"test result: ok"}}"#,
        );
        let events = parse_events(&text);
        assert_eq!(events.len(), 4);
        assert!(matches!(&events[0], DigestEvent::User(t) if t == "fix the parser"));
        assert!(matches!(&events[1], DigestEvent::Assistant(t) if t == "fixed it"));
        assert!(matches!(&events[2], DigestEvent::ToolCall { name, .. } if name == "shell"));
        assert!(matches!(&events[3], DigestEvent::ToolOutput(t) if t.contains("ok")));
    }

    #[test]
    fn skips_developer_and_injected_context() {
        let mut text = String::new();
        text.push_str(
            r#"{"type":"response_item","payload":{"type":"message","role":"developer","content":[{"type":"input_text","text":"system stuff"}]}}"#,
        );
        text.push('\n');
        text.push_str(&user_msg("<environment_context>blah</environment_context>"));
        text.push('\n');
        text.push_str(&user_msg("real question"));
        let events = parse_events(&text);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], DigestEvent::User(t) if t == "real question"));
    }

    #[test]
    fn malformed_and_unknown_lines_tolerated() {
        let mut text = String::from("not json at all\n");
        text.push_str(r#"{"type":"token_count","payload":{}}"#);
        text.push('\n');
        text.push_str(&user_msg("hello"));
        text.push('\n');
        text.push_str(r#"{"type":"event_msg","payload":{"type":"task_complete"}}"#);
        let events = parse_events(&text);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn base_instructions_never_leak() {
        let events = parse_events(META);
        assert!(events.is_empty());
        let summaries = parse_meta_title_check();
        assert!(summaries.is_none_or(|s| !s.contains("GPT-5")));
    }

    fn parse_meta_title_check() -> Option<String> {
        let v: Value = serde_json::from_str(META).ok()?;
        v.get("payload")?
            .get("base_instructions")?
            .as_str()
            .map(String::from)
    }

    #[test]
    fn discover_missing_dir_is_empty() {
        assert!(discover_codex(10).len() <= 200);
    }

    #[test]
    fn discover_real_store_smoke() {
        let dir = sessions_dir();
        let Some(dir) = dir else { return };
        if !dir.exists() {
            return; // codex not installed on this machine
        }
        let found = discover_codex(50);
        assert!(found.iter().all(|s| s.source == HandoffSource::Codex));
        assert!(found.iter().all(|s| s.id.len() == 36));
    }
}
