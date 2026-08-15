//! Handoff: extract a conversation digest from a foreign CLI session
//! (Codex / Antigravity) so a new Claude session can continue the task.
//!
//! Pure module — no I/O. Adapters under `adapters::handoff` produce
//! [`ForeignSessionSummary`] + [`DigestEvent`] lists; this module renders them
//! into a single seeded prompt under a byte budget.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffSource {
    Codex,
    Agy,
}

impl HandoffSource {
    pub fn as_str(self) -> &'static str {
        match self {
            HandoffSource::Codex => "codex",
            HandoffSource::Agy => "agy",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            HandoffSource::Codex => "Codex CLI",
            HandoffSource::Agy => "Antigravity",
        }
    }
}

/// Metadata for one foreign session, cheap to gather for listings.
#[derive(Debug, Clone)]
pub struct ForeignSessionSummary {
    pub source: HandoffSource,
    pub id: String,
    /// codex: first user message head; agy: title or preview.
    pub title: Option<String>,
    pub cwd: Option<String>,
    /// Unix seconds of last activity (file mtime / db timestamp).
    pub last_modified: u64,
    /// Store-specific locator: codex rollout file, agy conversations dir.
    pub path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone)]
pub enum DigestEvent {
    User(String),
    Assistant(String),
    ToolCall { name: String, args: String },
    ToolOutput(String),
}

/// Soft target for the rendered digest.
pub const DIGEST_BUDGET_BYTES: usize = 16 * 1024;
/// Absolute cap (still far below OS argv limits).
pub const DIGEST_HARD_CAP_BYTES: usize = 24 * 1024;

const USER_CAP: usize = 2048;
const ASSISTANT_CAP: usize = 600;
const TOOL_ARGS_CAP: usize = 120;
const TOOL_OUTPUT_CAP: usize = 120;
const LAST_ASSISTANT_CAP: usize = 3072;

/// Render the handoff prompt for a foreign session.
///
/// Truncation ladder when over budget: drop older tool lines, shrink
/// assistant turns, then drop the oldest events — always keeping the first
/// user event and the last few events, with an elision marker.
pub fn build_digest(
    summary: &ForeignSessionSummary,
    events: &[DigestEvent],
    now_secs: u64,
    budget: usize,
) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "You are continuing a task that was previously run in {}. Below is a digest of that session. Read it, then continue where it left off.\n\n",
        summary.source.label()
    ));

    out.push_str("## Session\n");
    out.push_str(&format!(
        "- Source: {} (session {})\n",
        summary.source.as_str(),
        short_id(&summary.id)
    ));
    if let Some(cwd) = &summary.cwd {
        out.push_str(&format!("- Workspace: {cwd}\n"));
    }
    out.push_str(&format!(
        "- Last activity: {} ago\n",
        format_age(now_secs.saturating_sub(summary.last_modified))
    ));
    if let Some(title) = &summary.title {
        out.push_str(&format!("- Task: {}\n", single_line(title, 160)));
    }
    out.push_str(
        "- Note: assistant replies and tool output are truncated. The workspace files reflect the latest state — inspect before assuming.\n\n",
    );

    out.push_str("## Conversation\n");
    if events.is_empty() {
        out.push_str("(no conversation events could be extracted — rely on the task line above and the workspace state)\n");
    } else {
        for level in 0..4 {
            let (kept, elided) = select_events(events, level);
            let body = render_events(&kept, events, level);
            let marked = if elided > 0 {
                format!("({elided} earlier events elided)\n\n")
            } else {
                String::new()
            };
            let candidate = format!("{out}{marked}{body}");
            if candidate.len() <= budget || level == 3 {
                out = candidate;
                break;
            }
        }
    }

    out.push_str("\n## Continue\n");
    out.push_str(
        "Continue the task above in this workspace. Review recent changes (git status / git diff) first if relevant. Do not restart from scratch; ask only if the digest is ambiguous about the current goal.\n",
    );

    if out.len() > DIGEST_HARD_CAP_BYTES {
        let cut = out
            .char_indices()
            .take_while(|(i, _)| *i <= DIGEST_HARD_CAP_BYTES.saturating_sub(24))
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        out.truncate(cut);
        out.push_str("\n(digest truncated)\n");
    }
    out
}

/// Progressive event selection. Level semantics:
/// 0 — everything; 1 — tool events beyond the last 10 dropped;
/// 2 — same as 1; 3 — only the first user event plus the last 8 events.
/// Returns (kept, elided_count).
fn select_events(events: &[DigestEvent], level: usize) -> (Vec<DigestEvent>, usize) {
    if level < 1 {
        return (events.to_vec(), 0);
    }

    let keep_tail_tools = 10;
    let tool_positions: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, DigestEvent::ToolCall { .. } | DigestEvent::ToolOutput(_)))
        .map(|(i, _)| i)
        .collect();
    let recent_tools: std::collections::HashSet<usize> = tool_positions
        .iter()
        .rev()
        .take(keep_tail_tools)
        .copied()
        .collect();

    let kept_idx: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(i, e)| {
            let is_tool = matches!(e, DigestEvent::ToolCall { .. } | DigestEvent::ToolOutput(_));
            !is_tool || recent_tools.contains(i)
        })
        .map(|(i, _)| i)
        .collect();

    if level < 3 {
        let elided = events.len() - kept_idx.len();
        let kept = kept_idx.iter().map(|&i| events[i].clone()).collect();
        return (kept, elided);
    }

    // Level 3: first user event + last 8 kept indices.
    let mut final_idx: Vec<usize> = kept_idx
        .iter()
        .copied()
        .filter(|&i| matches!(events[i], DigestEvent::User(_)))
        .take(1)
        .collect();
    let mut tail: Vec<usize> = kept_idx.iter().rev().take(8).copied().collect();
    tail.reverse();
    for i in tail {
        if !final_idx.contains(&i) {
            final_idx.push(i);
            final_idx.sort_unstable();
        }
    }
    let elided = events.len() - final_idx.len();
    let kept = final_idx.iter().map(|&i| events[i].clone()).collect();
    (kept, elided)
}

/// Render selected events. `all` is the unfiltered event list, used to spot
/// the final assistant turn (which gets a larger cap and a LAST marker).
fn render_events(kept: &[DigestEvent], all: &[DigestEvent], level: usize) -> String {
    let last_assistant = all
        .iter()
        .rposition(|e| matches!(e, DigestEvent::Assistant(_)));
    let last_assistant_text = all.iter().rev().find_map(|e| match e {
        DigestEvent::Assistant(t) => Some(t.as_str()),
        _ => None,
    });

    let user_cap = if level >= 2 { 1024 } else { USER_CAP };
    let asst_cap = if level >= 2 { 200 } else { ASSISTANT_CAP };

    let mut out = String::new();
    let mut n = 0usize;
    for (idx, event) in kept.iter().enumerate() {
        n += 1;
        match event {
            DigestEvent::User(text) => {
                out.push_str(&format!(
                    "[{n}] USER:\n{}\n\n",
                    cap_head_tail(text, user_cap)
                ));
            }
            DigestEvent::Assistant(text) => {
                if Some(idx) == last_assistant_kept(kept, all) {
                    out.push_str(&format!(
                        "[{n}] ASSISTANT (final state):\n{}\n\n",
                        cap_head_tail(text, LAST_ASSISTANT_CAP)
                    ));
                } else {
                    out.push_str(&format!(
                        "[{n}] ASSISTANT (truncated):\n{}\n\n",
                        cap_head_tail(text, asst_cap)
                    ));
                }
            }
            DigestEvent::ToolCall { name, args } => {
                out.push_str(&format!(
                    "[{n}] TOOL: {} — {}\n",
                    name,
                    single_line(args, TOOL_ARGS_CAP)
                ));
            }
            DigestEvent::ToolOutput(text) => {
                out.push_str(&format!("[{n}] → {}\n", single_line(text, TOOL_OUTPUT_CAP)));
            }
        }
    }

    // If the final assistant turn got dropped by selection, still surface it:
    // it carries the current state of the task.
    if level >= 1
        && last_assistant.is_some()
        && last_assistant_kept(kept, all).is_none()
        && let Some(text) = last_assistant_text
    {
        n += 1;
        out.push_str(&format!(
            "[{n}] ASSISTANT (final state):\n{}\n\n",
            cap_head_tail(text, LAST_ASSISTANT_CAP)
        ));
    }
    out
}

/// Position in `kept` of the event that is the final assistant event of `all`.
fn last_assistant_kept(kept: &[DigestEvent], all: &[DigestEvent]) -> Option<usize> {
    let last_text: &str = all.iter().rev().find_map(|e| match e {
        DigestEvent::Assistant(t) => Some(t.as_str()),
        _ => None,
    })?;
    kept.iter()
        .rposition(|e| matches!(e, DigestEvent::Assistant(t) if t.as_str() == last_text))
}

fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
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

/// Keep the head and tail of long text with an elision marker in the middle.
fn cap_head_tail(s: &str, cap: usize) -> String {
    let total = s.chars().count();
    if total <= cap {
        return s.trim_end().to_string();
    }
    let head = cap * 2 / 3;
    let tail = cap - head;
    let head_s: String = s.chars().take(head).collect();
    let tail_s: String = s.chars().skip(total - tail).collect();
    format!(
        "{head_s}\n[…{}/{} chars elided…]\n{tail_s}",
        total - cap,
        total
    )
}

fn format_age(secs: u64) -> String {
    if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary() -> ForeignSessionSummary {
        ForeignSessionSummary {
            source: HandoffSource::Codex,
            id: "019fb39e-32f4-7a60-ad90-824ec3273092".into(),
            title: Some("Fix parser module".into()),
            cwd: Some("/tmp/project".into()),
            last_modified: 1_800_000_000,
            path: None,
        }
    }

    #[test]
    fn user_text_is_verbatim_under_cap() {
        let events = vec![DigestEvent::User("fix the parser bug".into())];
        let d = build_digest(&summary(), &events, 1_800_000_300, DIGEST_BUDGET_BYTES);
        assert!(d.contains("fix the parser bug"));
        assert!(d.contains("[1] USER"));
    }

    #[test]
    fn header_contains_session_fields() {
        let events = vec![DigestEvent::User("x".into())];
        let d = build_digest(&summary(), &events, 1_800_000_300, DIGEST_BUDGET_BYTES);
        assert!(d.contains("Source: codex"));
        assert!(d.contains("session 019fb39e"));
        assert!(d.contains("Workspace: /tmp/project"));
        assert!(d.contains("Task: Fix parser module"));
        assert!(d.contains("5m ago"));
    }

    #[test]
    fn assistant_turns_truncated() {
        let long = "a".repeat(4000);
        let events = vec![
            DigestEvent::User("q".into()),
            DigestEvent::Assistant(long.clone()),
        ];
        let d = build_digest(&summary(), &events, 1_800_000_300, DIGEST_BUDGET_BYTES);
        assert!(!d.contains(&long));
        assert!(d.contains("chars elided"));
        // The only assistant turn is the final one — gets the larger cap.
        assert!(d.contains("final state"));
    }

    #[test]
    fn respects_budget_with_many_events() {
        let mut events = Vec::new();
        for i in 0..200 {
            events.push(DigestEvent::User(format!(
                "user message number {i} with some padding text to add volume"
            )));
            events.push(DigestEvent::Assistant("assistant reply ".repeat(80)));
            events.push(DigestEvent::ToolCall {
                name: "shell".into(),
                args: format!("cargo test -- {i}"),
            });
        }
        let d = build_digest(&summary(), &events, 1_800_000_300, DIGEST_BUDGET_BYTES);
        assert!(d.len() <= DIGEST_HARD_CAP_BYTES, "len={}", d.len());
        assert!(d.contains("elided"));
    }

    #[test]
    fn empty_events_produce_minimal_digest() {
        let d = build_digest(&summary(), &[], 1_800_000_300, DIGEST_BUDGET_BYTES);
        assert!(d.contains("no conversation events"));
        assert!(d.contains("## Continue"));
    }

    #[test]
    fn final_assistant_survives_heavy_truncation() {
        let mut events = Vec::new();
        for i in 0..100 {
            events.push(DigestEvent::User(format!("turn {i}")));
            events.push(DigestEvent::Assistant(
                format!("reply for turn {i} with enough text to occupy space ").repeat(40),
            ));
        }
        events.push(DigestEvent::Assistant(
            "FINAL: the parser fix is half-applied, tests failing".into(),
        ));
        let d = build_digest(&summary(), &events, 1_800_000_300, DIGEST_BUDGET_BYTES);
        assert!(d.contains("FINAL: the parser fix is half-applied"));
    }
}
