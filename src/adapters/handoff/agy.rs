//! Antigravity (agy) CLI session reader.
//!
//! Layout under `~/.gemini/antigravity-cli/`:
//! - `conversation_summaries.db` — SQLite index (conversation_id, title,
//!   preview, workspace_uris, last_modified_time). Authoritative listing.
//! - `history.jsonl` — one line per user prompt, keyed by conversationId.
//!   Used to enrich titles and as a Tier-1 fallback.
//! - `conversations/<uuid>.db` — SQLite with a `steps` table whose
//!   `step_payload` blobs are protobuf envelopes (undocumented format).
//!
//! Tier-2 extraction walks the protobuf wire format generically and collects
//! readable UTF-8 string fields, bucketing them into user / assistant / tool
//! events by heuristics. If the undocumented format changes and yields
//! nothing, callers fall back to the Tier-1 (history.jsonl) digest.

use std::path::Path;

use rusqlite::OpenFlags;

use super::agy_home;
use crate::domain::handoff::{DigestEvent, ForeignSessionSummary, HandoffSource};

/// List agy conversations from conversation_summaries.db, newest first.
pub fn discover_agy(limit: usize) -> Vec<ForeignSessionSummary> {
    let Some(dir) = agy_home() else {
        return Vec::new();
    };
    discover_agy_in(&dir, limit)
}

fn discover_agy_in(dir: &Path, limit: usize) -> Vec<ForeignSessionSummary> {
    let mut rows = match read_summaries(dir) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    rows.sort_by_key(|r| std::cmp::Reverse(r.last_modified));
    rows.truncate(limit);

    let history = read_history(dir);
    rows.into_iter()
        .map(|r| {
            let title = if r.title.as_deref().is_some_and(|t| !t.trim().is_empty()) {
                r.title
            } else {
                history.get(&r.id).and_then(|v| v.last().cloned())
            };
            let cwd = workspace_paths(&r.workspace_uris).into_iter().next();
            ForeignSessionSummary {
                source: HandoffSource::Agy,
                id: r.id.clone(),
                title,
                cwd,
                last_modified: r.last_modified,
                path: Some(dir.join("conversations").join(format!("{}.db", r.id))),
            }
        })
        .collect()
}

struct SummaryRow {
    id: String,
    title: Option<String>,
    workspace_uris: String,
    last_modified: u64,
}

fn read_summaries(dir: &Path) -> anyhow::Result<Vec<SummaryRow>> {
    let db = dir.join("conversation_summaries.db");
    let conn = rusqlite::Connection::open_with_flags(
        &db,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let mut stmt = conn.prepare(
        "SELECT conversation_id, title, preview, workspace_uris, last_modified_time
         FROM conversation_summaries",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SummaryRow {
            id: row.get(0)?,
            title: {
                let title: String = row.get(1)?;
                let preview: String = row.get(2)?;
                if title.trim().is_empty() {
                    Some(preview)
                } else {
                    Some(title)
                }
            },
            workspace_uris: row.get(3)?,
            last_modified: parse_db_time(&row.get::<_, String>(4)?),
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// agy stores datetimes like `2026-08-15 17:55:12.345+00:00`.
fn parse_db_time(s: &str) -> u64 {
    chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f%:z")
        .map(|dt| dt.timestamp().max(0) as u64)
        .unwrap_or(0)
}

/// conversationId → all user prompts in order (titles / Tier-1 fallback).
fn read_history(dir: &Path) -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();
    let Ok(text) = std::fs::read_to_string(dir.join("history.jsonl")) else {
        return map;
    };
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let (Some(id), Some(display)) = (
            v.get("conversationId").and_then(|x| x.as_str()),
            v.get("display").and_then(|x| x.as_str()),
        ) else {
            continue;
        };
        map.entry(id.to_string())
            .or_default()
            .push(display.to_string());
    }
    map
}

/// Extract events for one conversation. Tier-2 (steps.db protobuf scan)
/// carries the work log; Tier-1 (history.jsonl) carries the user prompts,
/// which the undocumented steps blobs don't reliably expose. Both are merged:
/// user prompts first (they define the task), then the extracted work log.
pub fn extract_agy_events(dir: &Path, id: &str) -> anyhow::Result<Vec<DigestEvent>> {
    let steps = extract_steps_events(dir, id).unwrap_or_default();
    let has_work = steps
        .iter()
        .any(|e| matches!(e, DigestEvent::Assistant(_) | DigestEvent::ToolCall { .. }));
    if !has_work {
        return Ok(tier1_events(dir, id));
    }
    let mut events = tier1_events(dir, id);
    events.extend(steps);
    Ok(events)
}

fn tier1_events(dir: &Path, id: &str) -> Vec<DigestEvent> {
    read_history(dir)
        .into_iter()
        .filter(|(k, _)| k == id)
        .flat_map(|(_, prompts)| {
            prompts
                .into_iter()
                .map(DigestEvent::User)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn extract_steps_events(dir: &Path, id: &str) -> anyhow::Result<Vec<DigestEvent>> {
    let db = dir.join("conversations").join(format!("{id}.db"));
    let conn = rusqlite::Connection::open_with_flags(
        &db,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let mut stmt = conn.prepare("SELECT step_payload FROM steps ORDER BY idx")?;
    let blobs = stmt.query_map([], |row| row.get::<_, Vec<u8>>(0))?;

    let mut events = Vec::new();
    for blob in blobs.filter_map(|b| b.ok()) {
        for text in scan_strings(&blob) {
            if let Some(event) = bucket_string(&text) {
                events.push(event);
            }
        }
    }
    Ok(events)
}

/// Bucket one extracted protobuf string into a digest event, or drop it.
fn bucket_string(text: &str) -> Option<DigestEvent> {
    let trimmed = text.trim();
    if trimmed.chars().count() < 12 {
        return None;
    }
    if trimmed.contains("_vrtx_") || looks_like_id(trimmed) || looks_like_base64(trimmed) {
        return None;
    }
    let first = trimmed.chars().next().unwrap_or(' ');
    if first == '{' || first == '[' {
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            // Tool arguments JSON: hand back as a tool line, name unknown.
            return Some(DigestEvent::ToolCall {
                name: "tool".into(),
                args: trimmed.to_string(),
            });
        }
        return None;
    }
    if looks_like_shell_command(trimmed) {
        return Some(DigestEvent::ToolCall {
            name: "shell".into(),
            args: trimmed.to_string(),
        });
    }
    if trimmed.contains(' ') {
        Some(DigestEvent::Assistant(trimmed.to_string()))
    } else {
        None // long single token — likely an id or path fragment
    }
}

/// Leading token is a well-known command → the string is a shell command
/// line the agent composed, not prose.
fn looks_like_shell_command(s: &str) -> bool {
    const COMMANDS: [&str; 24] = [
        "git", "cargo", "npm", "npx", "pnpm", "yarn", "node", "python", "python3", "pip", "make",
        "cmake", "go", "rustc", "brew", "apt", "sudo", "docker", "kubectl", "curl", "wget", "tar",
        "cp", "mv",
    ];
    let Some(first_token) = s.split_whitespace().next() else {
        return false;
    };
    COMMANDS.contains(&first_token) && s.contains(' ')
}

fn looks_like_id(s: &str) -> bool {
    s.len() >= 16 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

fn looks_like_base64(s: &str) -> bool {
    if s.len() < 40 || s.contains(char::is_whitespace) {
        return false;
    }
    let printable = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=' | '-' | '_'))
        .count();
    printable * 10 >= s.chars().count() * 9
}

/// Generic protobuf wire-format walk: collect every length-delimited field
/// that decodes as clean UTF-8, descending into nested submessages (depth ≤ 3).
/// Undocumented format — strings only, no schema assumptions.
pub fn scan_strings(buf: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    walk(buf, 0, &mut out);
    out
}

const MIN_STRING_LEN: usize = 4;
const MAX_DEPTH: usize = 3;

fn walk(buf: &[u8], depth: usize, out: &mut Vec<String>) {
    if depth > MAX_DEPTH {
        return;
    }
    let mut i = 0usize;
    while i < buf.len() {
        let Some((tag, consumed)) = read_varint(&buf[i..]) else {
            break;
        };
        i += consumed;
        let wire = (tag & 0x7) as u8;
        let field = tag >> 3;
        match wire {
            0 => {
                let Some((_, consumed)) = read_varint(&buf[i..]) else {
                    break;
                };
                i += consumed;
            }
            1 => i += 8,
            5 => i += 4,
            2 => {
                let Some((len, consumed)) = read_varint(&buf[i..]) else {
                    break;
                };
                i += consumed;
                let len = len as usize;
                let Some(chunk) = buf.get(i..i + len) else {
                    break;
                };
                i += len;
                if len >= MIN_STRING_LEN
                    && field != 0
                    && let Ok(text) = std::str::from_utf8(chunk)
                    && !text.chars().any(|c| c.is_control())
                {
                    out.push(text.to_string());
                    continue;
                }
                // Not clean text — try descending (it may be a nested message).
                walk(chunk, depth + 1, out);
            }
            _ => break, // groups (3/4) deprecated: stop this branch
        }
    }
}

fn read_varint(buf: &[u8]) -> Option<(u64, usize)> {
    let mut value = 0u64;
    for (i, byte) in buf.iter().take(10).enumerate() {
        value |= u64::from(byte & 0x7F) << (i * 7);
        if byte & 0x80 == 0 {
            return Some((value, i + 1));
        }
    }
    None
}

/// Decode agy `workspace_uris` (JSON array of `file://` URIs) into paths.
pub fn workspace_paths(summary_cwd_field: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(summary_cwd_field)
        .ok()
        .and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.iter()
                .filter_map(|u| u.as_str())
                .filter_map(|u| u.strip_prefix("file://"))
                .map(percent_decode)
                .collect()
        })
        .unwrap_or_default()
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(mut n: u64) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let b = (n & 0x7F) as u8;
            n >>= 7;
            if n == 0 {
                out.push(b);
                break;
            }
            out.push(b | 0x80);
        }
        out
    }

    fn pb_string(field: u64, s: &str) -> Vec<u8> {
        let mut out = varint((field << 3) | 2);
        out.extend(varint(s.len() as u64));
        out.extend_from_slice(s.as_bytes());
        out
    }

    #[test]
    fn scan_strings_finds_nested_and_flat_strings() {
        let mut buf = pb_string(1, "flat string field content here");
        let nested = pb_string(2, "nested string field content here too");
        let mut envelope = vec![(2 << 3 | 2) as u8, nested.len() as u8];
        envelope.extend_from_slice(&nested);
        buf.extend_from_slice(&envelope);
        let found = scan_strings(&buf);
        assert!(found.iter().any(|s| s.contains("flat string")));
        assert!(found.iter().any(|s| s.contains("nested string")));
    }

    #[test]
    fn scan_strings_stops_on_garbage() {
        let mut buf = vec![0x08, 0x2A]; // varint field 1 = 42
        buf.extend_from_slice(&[0xFF, 0xFE, 0x00, 0x01]); // trailing garbage
        let found = scan_strings(&buf);
        assert!(found.is_empty()); // no valid strings, no panic
    }

    #[test]
    fn bucket_drops_ids_and_base64() {
        assert!(bucket_string("0805fa96-5b8e-4277-b172-1dc149672d39").is_none());
        assert!(bucket_string("toolu_vrtx_016Etba8UPMzrnLr3NhNuqkg").is_none());
        assert!(bucket_string("EuUDCnMIEBACGAIqQCdbX+zrL5ND6+164hOf4wy7JoJBuuO2U").is_none());
        assert!(bucket_string("short").is_none());
    }

    #[test]
    fn bucket_classifies_json_and_prose() {
        let json = r#"{"DirectoryPath":"/tmp/proj","toolAction":"list files"}"#;
        assert!(matches!(
            bucket_string(json),
            Some(DigestEvent::ToolCall { .. })
        ));
        let prose = "The user is asking me to analyze the project for release readiness";
        assert!(matches!(
            bucket_string(prose),
            Some(DigestEvent::Assistant(_))
        ));
    }

    #[test]
    fn workspace_paths_decodes_uris() {
        let uris = r#"["file:///Users/hackme/projects","file:///tmp/a%20b"]"#;
        let paths = workspace_paths(uris);
        assert_eq!(paths, vec!["/Users/hackme/projects", "/tmp/a b"]);
    }

    #[test]
    fn parse_db_time_handles_agy_format() {
        assert!(parse_db_time("2026-08-15 17:55:12.345+00:00") > 1_700_000_000);
        assert_eq!(parse_db_time("garbage"), 0);
    }

    #[test]
    fn discover_missing_store_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(discover_agy_in(tmp.path(), 10).is_empty());
    }

    #[test]
    fn discover_reads_summaries_db() {
        let tmp = tempfile::tempdir().unwrap();
        let conn =
            rusqlite::Connection::open(tmp.path().join("conversation_summaries.db")).unwrap();
        conn.execute(
            "CREATE TABLE conversation_summaries (
                conversation_id text PRIMARY KEY, title text NOT NULL DEFAULT '',
                preview text NOT NULL DEFAULT '', step_count integer NOT NULL DEFAULT 0,
                last_modified_time datetime NOT NULL, workspace_uris text NOT NULL DEFAULT '',
                status text NOT NULL DEFAULT '', source text NOT NULL DEFAULT '',
                project_id text NOT NULL DEFAULT '', agent_name text NOT NULL DEFAULT '',
                parent_conversation_id text NOT NULL DEFAULT '', nesting_depth integer NOT NULL DEFAULT 0,
                battle_id text NOT NULL DEFAULT '', winning_conversation_id text NOT NULL DEFAULT '',
                not_fully_idle numeric NOT NULL DEFAULT false, killed numeric NOT NULL DEFAULT false,
                last_user_input_time datetime NOT NULL, last_user_input_step_index integer NOT NULL DEFAULT -1,
                app_data_dir text NOT NULL DEFAULT '')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO conversation_summaries (conversation_id, title, preview, last_modified_time, workspace_uris, last_user_input_time)
             VALUES ('b32ad831-5bf0-4fb4-b0ce-c2277d520b74', 'analyze similarity', '', '2026-08-15 17:55:12.345+00:00', '[\"file:///tmp/proj\"]', '2026-08-15 17:55:00.000+00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO conversation_summaries (conversation_id, title, preview, last_modified_time, workspace_uris, last_user_input_time)
             VALUES ('1746a1dd-8ff4-4d8a-b3cc-45c0c682f328', '', 'fix the build error', '2026-08-15 18:28:00.000+00:00', '[\"file:///other\"]', '2026-08-15 18:27:00.000+00:00')",
            [],
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("history.jsonl"),
            "{\"display\":\"fix the build error please\",\"timestamp\":1785428096076,\"workspace\":\"/tmp\",\"conversationId\":\"1746a1dd-8ff4-4d8a-b3cc-45c0c682f328\"}\n",
        )
        .unwrap();

        let found = discover_agy_in(tmp.path(), 10);
        assert_eq!(found.len(), 2);
        // newest first: 18:28 row
        assert_eq!(found[0].id, "1746a1dd-8ff4-4d8a-b3cc-45c0c682f328");
        assert_eq!(found[0].cwd.as_deref(), Some("/other"));
        // empty title falls back to preview (itself user-prompt-derived)
        assert_eq!(found[0].title.as_deref(), Some("fix the build error"));
        assert_eq!(found[1].title.as_deref(), Some("analyze similarity"));
    }

    #[test]
    fn discover_real_store_smoke() {
        let Some(dir) = agy_home() else { return };
        if !dir.join("conversation_summaries.db").exists() {
            return; // agy not installed
        }
        let found = discover_agy(20);
        assert!(found.iter().all(|s| s.source == HandoffSource::Agy));
        if let Some(first) = found.first() {
            let events = extract_agy_events(&dir, &first.id).unwrap_or_default();
            // Tier-2 or Tier-1 must produce at least one event on real data.
            assert!(!events.is_empty(), "no events extracted for {}", first.id);
        }
    }
}
