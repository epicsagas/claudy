use crate::adapters::channel::sessions::{
    SessionInfo, claude_projects_dir, count_invalid_thinking_blocks, discover_sessions,
    sanitize_session_thinking_blocks,
};
use crate::domain::context::Context;

pub fn run_session_sanitize(
    ctx: &mut Context,
    project: Option<&str>,
    all: bool,
    yes: bool,
) -> anyhow::Result<i32> {
    let Some(projects_dir) = claude_projects_dir() else {
        ctx.output.warn("~/.claude/projects not found");
        return Ok(1);
    };

    let sessions = discover_sessions(&projects_dir, 200);

    // If no project filter given, default to the current directory's name.
    let effective_filter: Option<String> = project.map(|s| s.to_string()).or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
    });

    let sessions: Vec<SessionInfo> = if let Some(ref f) = effective_filter {
        let f = f.to_lowercase();
        let filtered: Vec<_> = sessions
            .into_iter()
            .filter(|s| s.project_name.to_lowercase().contains(&f))
            .collect();
        if filtered.is_empty() && project.is_none() {
            discover_sessions(&projects_dir, 200)
        } else {
            filtered
        }
    } else {
        sessions
    };

    let flagged: Vec<(SessionInfo, usize)> = sessions
        .into_iter()
        .filter_map(|s| {
            let n = count_invalid_thinking_blocks(&projects_dir, &s.session_id);
            if n > 0 { Some((s, n)) } else { None }
        })
        .collect();

    if flagged.is_empty() {
        ctx.output
            .success("No sessions with invalid thinking blocks found.");
        return Ok(0);
    }

    // ── selection ────────────────────────────────────────────────────────────
    let targets: Vec<(SessionInfo, usize)> = if all {
        flagged
    } else {
        // Conservative item budget: dialoguer adds "~> " (3 cols) + scrollbar
        let term_cols = terminal_cols();
        let item_budget = term_cols.saturating_sub(6).min(76);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut items: Vec<String> = flagged
            .iter()
            .map(|(s, n)| {
                let age = format_age(now.saturating_sub(s.last_modified));
                let proj = sanitize_str(&truncate_display(&s.project_name, 14));
                let raw_msg = sanitize_str(
                    &s.last_message
                        .as_deref()
                        .unwrap_or("")
                        .lines()
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                // prefix: "proj / sid  age  (Nblk)  "
                let prefix = format!("{} / {}  {}  ({}blk)  ", proj, &s.session_id[..8], age, n);
                let prefix_w = display_width(&prefix);
                let preview_budget = item_budget.saturating_sub(prefix_w);
                let preview = truncate_display(&raw_msg, preview_budget);
                format!("{}{}", prefix, preview)
            })
            .collect();
        items.push("Sanitize ALL".to_string());

        let choice = ctx
            .prompt
            .select_opt("Select session to sanitize", &items, 0)?;

        match choice {
            None => {
                ctx.output.info("Cancelled.");
                return Ok(0);
            }
            Some(i) if i == flagged.len() => flagged,
            Some(i) => vec![flagged.into_iter().nth(i).unwrap()],
        }
    };

    // ── confirm & run ────────────────────────────────────────────────────────
    if !yes && !all {
        let ok = ctx.prompt.confirm(
            "Convert thinking blocks to text and overwrite session file?",
            true,
        )?;
        if !ok {
            ctx.output.info("Cancelled.");
            return Ok(0);
        }
    }

    let mut total = 0usize;
    for (s, _) in &targets {
        match sanitize_session_thinking_blocks(&projects_dir, &s.session_id) {
            Ok(n) => {
                total += n;
                ctx.output.success(&format!(
                    "{} / {} — converted {} block(s)",
                    s.project_name,
                    &s.session_id[..8],
                    n
                ));
            }
            Err(e) => {
                ctx.output.warn(&format!(
                    "{} / {} — failed: {}",
                    s.project_name,
                    &s.session_id[..8],
                    e
                ));
            }
        }
    }

    ctx.output
        .info(&format!("Done. {} block(s) converted in total.", total));
    Ok(0)
}

/// Strip control characters, tabs, and zero-width chars to prevent line-wrap in dialoguer.
fn sanitize_str(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() && *c != '\u{200B}' && *c != '\u{FEFF}')
        .collect()
}

/// Returns terminal column width via TIOCGWINSZ, falling back to 80.
fn terminal_cols() -> usize {
    #[cfg(unix)]
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            return ws.ws_col as usize;
        }
    }
    80
}

/// Display column width of a string (CJK/fullwidth chars count as 2).
fn display_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// Truncate string to at most `max_cols` display columns.
fn truncate_display(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    let mut cols = 0usize;
    let mut result = String::new();
    for ch in s.chars() {
        let w = char_width(ch);
        if cols + w > max_cols {
            break;
        }
        result.push(ch);
        cols += w;
    }
    result
}

/// Approximate display width for a single char (1 for ASCII/narrow, 2 for CJK/wide).
fn char_width(c: char) -> usize {
    let cp = c as u32;
    if matches!(cp,
        0x1100..=0x115F   // Hangul Jamo
        | 0x2E80..=0x303F // CJK Radicals / Kangxi / Symbols
        | 0x3040..=0x33FF // Japanese kana + CJK compatibility
        | 0x3400..=0x4DBF // CJK Extension A
        | 0x4E00..=0x9FFF // CJK Unified Ideographs
        | 0xA000..=0xA4CF // Yi
        | 0xA960..=0xA97F // Hangul Jamo Extended-A
        | 0xAC00..=0xD7AF // Hangul Syllables
        | 0xD7B0..=0xD7FF // Hangul Jamo Extended-B
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
        | 0xFE10..=0xFE6F // CJK Compatibility Forms / Small Forms
        | 0xFF00..=0xFFEF // Halfwidth/Fullwidth Forms
        | 0x1F300..=0x1FAFF // Emoji / Symbols
        | 0x20000..=0x2FA1F // CJK Extensions B-F
    ) {
        2
    } else {
        1
    }
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
