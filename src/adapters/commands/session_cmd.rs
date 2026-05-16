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
    let effective_filter: Option<String> = project
        .map(|s| s.to_string())
        .or_else(|| {
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
        // If the cwd-derived filter matches nothing, fall back to all sessions.
        if filtered.is_empty() && project.is_none() {
            discover_sessions(&projects_dir, 200)
        } else {
            filtered
        }
    } else {
        sessions
    };

    // Keep only sessions that actually have invalid thinking blocks
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

    // ── display table ────────────────────────────────────────────────────────
    let sep = "─".repeat(82);
    ctx.output.header("Sessions with invalid thinking blocks");
    ctx.output.write_line(&sep)?;
    ctx.output.write_line(&format!(
        " {:<3}  {:<16}  {:<8}  {:<7}  {:<36}  {}",
        "#", "Project", "Session", "Age", "Last message", "Blocks"
    ))?;
    ctx.output.write_line(&sep)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for (idx, (s, n)) in flagged.iter().enumerate() {
        let age = format_age(now.saturating_sub(s.last_modified));
        let last = s
            .last_message
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(36)
            .collect::<String>();
        ctx.output.write_line(&format!(
            " {:<3}  {:<16}  {:<8}  {:<7}  {:<36}  {}",
            idx + 1,
            truncate_str(&s.project_name, 16),
            &s.session_id[..8],
            age,
            last,
            n,
        ))?;
    }
    ctx.output.write_line(&sep)?;

    // ── selection ────────────────────────────────────────────────────────────
    let targets: Vec<(SessionInfo, usize)> = if all {
        flagged
    } else {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut items: Vec<String> = flagged
            .iter()
            .map(|(s, n)| {
                let age = format_age(now.saturating_sub(s.last_modified));
                let preview = s
                    .last_message
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(40)
                    .collect::<String>();
                format!(
                    "{} / {}  {}  \"{}\"  ({} blocks)",
                    truncate_str(&s.project_name, 20),
                    &s.session_id[..8],
                    age,
                    preview,
                    n
                )
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
            Some(i) if i == flagged.len() => flagged, // "all"
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

fn format_age(secs: u64) -> String {
    if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{}…", t)
    }
}
