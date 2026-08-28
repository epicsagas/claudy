//! Session bridge: exchange messages with existing codex/agy sessions from a
//! running Claude session (MCP tools `list_sessions` / `read_session` /
//! `send_message`).
//!
//! Mechanics (verified 2026-08-28):
//! - codex idle session: `codex queue --thread <id> --message <msg>` parks the
//!   message; `codex exec resume --skip-git-repo-check <id> <nudge>` consumes
//!   it (the queued text appears in the rollout turn).
//! - agy any session: `agy --conversation <id> -p <msg>` continues the same
//!   conversation thread with prior context intact.
//! - reading: the handoff readers already parse both stores live (rollout
//!   JSONL / conversation SQLite WAL).

use std::path::Path;
use std::time::Duration;

use crate::adapters::handoff::{agy, agy_home, codex};
use crate::adapters::mcp::runner;
use crate::domain::handoff::{DigestEvent, ForeignSessionSummary, HandoffSource};

/// Sessions listed per `list_sessions` call.
const LIST_LIMIT: usize = 50;
/// Default / max event count for `read_session`.
pub const TAIL_DEFAULT: usize = 20;
const TAIL_MAX: usize = 200;
/// Fallback send timeouts (seconds) when the agent isn't in the discovery set.
pub const CODEX_SEND_TIMEOUT_SECS: u64 = 3600;
pub const AGY_SEND_TIMEOUT_SECS: u64 = 300;
/// Max message length, matching ask_agent's prompt cap.
const MESSAGE_MAX_CHARS: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSource {
    Codex,
    Agy,
}

impl BridgeSource {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s {
            "codex" => Ok(BridgeSource::Codex),
            "agy" => Ok(BridgeSource::Agy),
            other => anyhow::bail!("Unknown source '{other}' (expected 'codex' or 'agy')"),
        }
    }

    fn handoff_source(self) -> HandoffSource {
        match self {
            BridgeSource::Codex => HandoffSource::Codex,
            BridgeSource::Agy => HandoffSource::Agy,
        }
    }
}

/// List recent foreign sessions, newest first.
pub fn list_sessions(source: BridgeSource) -> Vec<ForeignSessionSummary> {
    match source {
        BridgeSource::Codex => codex::discover_codex(LIST_LIMIT),
        BridgeSource::Agy => agy::discover_agy(LIST_LIMIT),
    }
}

/// Render a session summary as one text line for the MCP response.
pub fn render_session(s: &ForeignSessionSummary, now_secs: u64) -> String {
    let age_secs = now_secs.saturating_sub(s.last_modified);
    let age = if age_secs < 3600 {
        format!("{}m", age_secs / 60)
    } else if age_secs < 86400 {
        format!("{}h", age_secs / 3600)
    } else {
        format!("{}d", age_secs / 86400)
    };
    let title = s
        .title
        .as_deref()
        .unwrap_or("(untitled)")
        .replace('\n', " ");
    let title: String = title.chars().take(80).collect();
    format!(
        "{}  {:<9}  {:<4}  {}  {}",
        &s.id[..s.id.len().min(8)],
        s.source.as_str(),
        age,
        s.cwd.as_deref().unwrap_or("-"),
        title
    )
}

/// Read the last `tail` events of a foreign session, rendered as text.
pub fn read_session(source: BridgeSource, id: &str, tail: usize) -> anyhow::Result<String> {
    let summary = find_session(source, id)?;
    let events = extract_events(source, &summary)?;
    let tail = tail.clamp(1, TAIL_MAX);
    let start = events.len().saturating_sub(tail);
    let slice = &events[start..];
    Ok(render_events(
        source,
        &summary,
        slice,
        events.len() - slice.len(),
    ))
}

/// The nudge that makes the resume turn consume queued bridge messages.
const CODEX_RESUME_NUDGE: &str =
    "A bridged message was just queued for you. Process it now and reply with its answer.";

/// `codex queue` argv: park the message on the session's queue.
fn codex_queue_args(id: &str, message: &str) -> Vec<String> {
    vec![
        "queue".to_string(),
        "--thread".to_string(),
        id.to_string(),
        "--message".to_string(),
        message.to_string(),
    ]
}

/// `codex exec resume` argv: resume headlessly; the turn consumes the queue.
fn codex_resume_args(id: &str) -> Vec<String> {
    vec![
        "exec".to_string(),
        "resume".to_string(),
        "--skip-git-repo-check".to_string(),
        id.to_string(),
        CODEX_RESUME_NUDGE.to_string(),
    ]
}

/// `agy` argv: continue the conversation headlessly with one prompt.
fn agy_send_args(id: &str, message: &str) -> Vec<String> {
    vec![
        "--conversation".to_string(),
        id.to_string(),
        "-p".to_string(),
        message.to_string(),
    ]
}

/// Deliver a message into an existing foreign session, returning the reply.
///
/// codex: if the resume step fails (e.g. quota exhaustion), the queued
/// message stays parked and is consumed by the next successful resume.
pub async fn send_message(
    source: BridgeSource,
    id: &str,
    message: &str,
    timeout_secs: u64,
) -> anyhow::Result<String> {
    if message.is_empty() {
        anyhow::bail!("Message must not be empty");
    }
    if message.len() > MESSAGE_MAX_CHARS {
        anyhow::bail!("Message exceeds maximum length of {MESSAGE_MAX_CHARS} characters");
    }
    let summary = find_session(source, id)?;
    let cwd = summary.cwd.as_deref().map(Path::new);

    match source {
        BridgeSource::Codex => {
            // Park the message, then resume headlessly — the resume turn
            // consumes the queued message.
            let queue_args = codex_queue_args(&summary.id, message);
            runner::run_command(
                "codex queue",
                "codex",
                &queue_args,
                cwd,
                Duration::from_secs(timeout_secs),
            )
            .await?;

            let resume_args = codex_resume_args(&summary.id);
            runner::run_command(
                "codex exec resume",
                "codex",
                &resume_args,
                cwd,
                Duration::from_secs(timeout_secs),
            )
            .await
        }
        BridgeSource::Agy => {
            let args = agy_send_args(&summary.id, message);
            runner::run_command("agy", "agy", &args, cwd, Duration::from_secs(timeout_secs)).await
        }
    }
}

/// Find a session by exact id or unique prefix (the listing shows 8 chars).
fn find_session(source: BridgeSource, id: &str) -> anyhow::Result<ForeignSessionSummary> {
    find_in(&list_sessions(source), id, source.handoff_source().as_str())
}

fn find_in(
    sessions: &[ForeignSessionSummary],
    id: &str,
    source_name: &str,
) -> anyhow::Result<ForeignSessionSummary> {
    if id.is_empty() {
        anyhow::bail!("Id must not be empty");
    }
    let matches: Vec<&ForeignSessionSummary> = sessions
        .iter()
        .filter(|s| s.id == id || s.id.starts_with(id))
        .collect();
    match matches.as_slice() {
        [] => anyhow::bail!(
            "No {source_name} session matching id '{id}' (use list_sessions to see ids)"
        ),
        [one] => Ok((*one).clone()),
        _ => anyhow::bail!(
            "Id '{id}' matches {} {source_name} sessions — use more characters",
            matches.len()
        ),
    }
}

fn extract_events(
    source: BridgeSource,
    summary: &ForeignSessionSummary,
) -> anyhow::Result<Vec<DigestEvent>> {
    match source {
        BridgeSource::Codex => {
            let path = summary.path.as_ref().ok_or_else(|| {
                anyhow::anyhow!("No rollout file recorded for session {}", summary.id)
            })?;
            codex::extract_codex_events(path)
        }
        BridgeSource::Agy => {
            let dir = agy_home().ok_or_else(|| anyhow::anyhow!("Cannot locate the agy store"))?;
            agy::extract_agy_events(&dir, &summary.id)
        }
    }
}

fn render_events(
    source: BridgeSource,
    summary: &ForeignSessionSummary,
    events: &[DigestEvent],
    elided: usize,
) -> String {
    let mut out = format!(
        "Session {} ({}) — {} events, {} earlier elided\n\n",
        &summary.id[..summary.id.len().min(8)],
        source.handoff_source().as_str(),
        events.len(),
        elided
    );
    if events.is_empty() {
        out.push_str("(no events could be extracted)\n");
        return out;
    }
    for event in events {
        match event {
            DigestEvent::User(t) => out.push_str(&format!("USER: {}\n", single_line(t, 400))),
            DigestEvent::Assistant(t) => {
                out.push_str(&format!("ASSISTANT: {}\n", single_line(t, 400)))
            }
            DigestEvent::ToolCall { name, args } => {
                out.push_str(&format!("TOOL {name}: {}\n", single_line(args, 160)))
            }
            DigestEvent::ToolOutput(t) => out.push_str(&format!("→ {}\n", single_line(t, 160))),
        }
    }
    out
}

fn single_line(s: &str, cap: usize) -> String {
    let joined: String = s
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut out: String = joined.chars().take(cap).collect();
    if joined.chars().count() > cap {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_parse() {
        assert_eq!(BridgeSource::parse("codex").unwrap(), BridgeSource::Codex);
        assert_eq!(BridgeSource::parse("agy").unwrap(), BridgeSource::Agy);
        assert!(BridgeSource::parse("slack").is_err());
    }

    #[test]
    fn find_in_requires_unique_prefix() {
        let mk = |id: &str| ForeignSessionSummary {
            source: HandoffSource::Codex,
            id: id.to_string(),
            title: None,
            cwd: None,
            last_modified: 0,
            path: None,
        };
        let sessions = vec![mk("aaaa1111-xx"), mk("aaaa2222-xx")];
        assert_eq!(
            find_in(&sessions, "aaaa1", "codex").unwrap().id,
            "aaaa1111-xx"
        );
        assert_eq!(
            find_in(&sessions, "aaaa2222-xx", "codex").unwrap().id,
            "aaaa2222-xx"
        ); // exact wins ambiguity
        let ambiguous = find_in(&sessions, "aaaa", "codex").unwrap_err().to_string();
        assert!(ambiguous.contains("matches 2"));
        let missing = find_in(&sessions, "zzzz", "codex").unwrap_err().to_string();
        assert!(missing.contains("No codex session"));
        // Empty id would prefix-match every session — rejected outright.
        assert!(find_in(&sessions, "", "codex").is_err());
    }

    #[test]
    fn codex_queue_args_shape() {
        let args = codex_queue_args("01a04867-abcd", "hello --flag $HOME");
        assert_eq!(
            args,
            vec![
                "queue",
                "--thread",
                "01a04867-abcd",
                "--message",
                "hello --flag $HOME",
            ]
        );
    }

    #[test]
    fn codex_resume_args_shape() {
        let args = codex_resume_args("01a04867-abcd");
        assert_eq!(
            args[..4],
            ["exec", "resume", "--skip-git-repo-check", "01a04867-abcd"][..]
        );
        assert_eq!(args[4], CODEX_RESUME_NUDGE);
    }

    #[test]
    fn agy_send_args_shape() {
        let args = agy_send_args("c4eada91-1234", "what was the codeword?");
        assert_eq!(
            args,
            vec![
                "--conversation",
                "c4eada91-1234",
                "-p",
                "what was the codeword?"
            ]
        );
    }

    #[test]
    fn render_session_line() {
        let s = ForeignSessionSummary {
            source: HandoffSource::Agy,
            id: "f66b432b-35c1".to_string(),
            title: Some("multi\nline title".to_string()),
            cwd: Some("/tmp/proj".to_string()),
            last_modified: 1_800_000_000,
            path: None,
        };
        let line = render_session(&s, 1_800_000_300);
        assert!(line.contains("f66b432b"));
        assert!(line.contains("agy"));
        assert!(line.contains("5m"));
        assert!(line.contains("/tmp/proj"));
        assert!(line.contains("multi line title"));
    }

    #[test]
    fn render_events_caps_lines() {
        let s = ForeignSessionSummary {
            source: HandoffSource::Codex,
            id: "019fb39e-32f4".to_string(),
            title: None,
            cwd: None,
            last_modified: 0,
            path: None,
        };
        let events = vec![
            DigestEvent::User("fix\nthe\nparser".to_string()),
            DigestEvent::Assistant("ok".repeat(1000)),
        ];
        let text = render_events(BridgeSource::Codex, &s, &events, 3);
        assert!(text.contains("USER: fix the parser"));
        assert!(text.contains("3 earlier elided"));
        assert!(text.contains('…'));
    }

    #[test]
    fn real_store_read_tail() {
        // Smoke: reads whichever sessions exist on this machine.
        for source in [BridgeSource::Codex, BridgeSource::Agy] {
            let sessions = list_sessions(source);
            let Some(first) = sessions.first() else {
                continue;
            };
            let text = read_session(source, &first.id, 5).unwrap();
            assert!(text.contains("Session"));
            // Header reports the returned event count — tail must bound it.
            let header = text.lines().next().unwrap_or_default();
            let shown: usize = header
                .split("— ")
                .nth(1)
                .and_then(|s| s.split(" events").next())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            assert!(shown <= 5, "tail=5 but {shown} events returned");
        }
    }
}
