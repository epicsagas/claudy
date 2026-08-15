use std::path::PathBuf;

use crate::adapters::commands::session_cmd::{format_age, sanitize_str, truncate_display};
use crate::adapters::handoff::{agy, codex, cwd_matches};
use crate::domain::context::Context;
use crate::domain::handoff::{
    DIGEST_BUDGET_BYTES, DigestEvent, ForeignSessionSummary, HandoffSource, build_digest,
};

/// Parsed `claudy [profile] handoff [codex|agy] [options]` arguments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HandoffOptions {
    pub source: Option<HandoffSource>,
    /// `-c` — hand off the single most recent session, no picker.
    pub continue_latest: bool,
    /// `-r` — pick among the 5 most recent sessions.
    pub resume_pick: bool,
    pub id: Option<String>,
    pub cwd: Option<String>,
    pub profile: Option<String>,
    pub print: bool,
    pub yolo: bool,
    /// Unknown flags, forwarded verbatim to the Claude session.
    pub forward: Vec<String>,
}

/// Parse handoff arguments. Known flags are consumed; the source is a
/// positional (`codex` / `agy`); anything else is forwarded to Claude.
pub fn parse_handoff_args(args: &[String]) -> anyhow::Result<HandoffOptions> {
    let mut opts = HandoffOptions::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        let (flag, inline_value) = match arg.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f, Some(v.to_string())),
            _ => (arg, None),
        };
        let take_value = |i: &mut usize| -> anyhow::Result<String> {
            if let Some(v) = inline_value.clone() {
                return Ok(v);
            }
            *i += 1;
            args.get(*i)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("flag {flag} requires a value"))
        };
        match flag {
            "codex" | "agy" => {
                if opts.source.is_some() {
                    anyhow::bail!("source specified twice: {arg}");
                }
                opts.source = Some(if flag == "codex" {
                    HandoffSource::Codex
                } else {
                    HandoffSource::Agy
                });
            }
            "-c" | "--continue" => opts.continue_latest = true,
            "-r" | "--resume" => opts.resume_pick = true,
            "--print" => opts.print = true,
            "--yolo" => opts.yolo = true,
            "--id" => opts.id = Some(take_value(&mut i)?),
            "--cwd" => opts.cwd = Some(take_value(&mut i)?),
            "--profile" => opts.profile = Some(take_value(&mut i)?),
            _ => opts.forward.push(arg.to_string()),
        }
        i += 1;
    }

    if opts.continue_latest && opts.resume_pick {
        anyhow::bail!("-c/--continue and -r/--resume are mutually exclusive");
    }
    if (opts.continue_latest || opts.resume_pick) && opts.id.is_some() {
        anyhow::bail!("-c/-r cannot be combined with --id");
    }
    Ok(opts)
}

/// `claudy [profile] handoff ...` — extract a digest from a foreign CLI
/// session (codex or agy) and seed a new Claude session with it.
pub fn run_handoff(
    ctx: &mut Context,
    profile: Option<String>,
    args: Vec<String>,
) -> anyhow::Result<i32> {
    let opts = parse_handoff_args(&args)?;
    let profile = opts
        .profile
        .or(profile)
        .unwrap_or_else(|| "anthropic".to_string());

    // ── gather ──────────────────────────────────────────────────────────────
    let mut sessions: Vec<ForeignSessionSummary> = Vec::new();
    match opts.source {
        Some(HandoffSource::Codex) => sessions.extend(discover_codex_titled()),
        Some(HandoffSource::Agy) => sessions.extend(discover_agy_titled()),
        None => {
            sessions.extend(discover_codex_titled());
            sessions.extend(discover_agy_titled());
        }
    }

    if sessions.is_empty() {
        let store = match opts.source {
            Some(HandoffSource::Codex) => "~/.codex/sessions",
            Some(HandoffSource::Agy) => "~/.gemini/antigravity-cli",
            None => "~/.codex/sessions or ~/.gemini/antigravity-cli",
        };
        ctx.output.warn(&format!(
            "No foreign sessions found (looked in {store}). Is the CLI installed?"
        ));
        return Ok(1);
    }
    sessions.sort_by_key(|s| std::cmp::Reverse(s.last_modified));

    // ── filter by cwd ───────────────────────────────────────────────────────
    let target_cwd: Option<PathBuf> = match opts.cwd.as_deref() {
        Some(c) => Some(PathBuf::from(c)),
        None => std::env::current_dir().ok(),
    };
    let cwd_explicit = opts.cwd.is_some();
    let mut filtered: Vec<_> = match &target_cwd {
        Some(target) => sessions
            .iter()
            .filter(|s| cwd_matches(s, target))
            .cloned()
            .collect(),
        None => sessions.clone(),
    };
    if filtered.is_empty() {
        if cwd_explicit {
            anyhow::bail!(
                "No sessions found for workspace '{}'",
                opts.cwd.as_deref().unwrap_or_default()
            );
        }
        filtered = sessions; // implicit filter matched nothing — show all
    }

    // ── filter by id ────────────────────────────────────────────────────────
    if let Some(ref want) = opts.id {
        filtered.retain(|s| {
            s.id == *want
                || s.path
                    .as_ref()
                    .is_some_and(|p| p.to_string_lossy().contains(want.as_str()))
        });
        if filtered.is_empty() {
            anyhow::bail!("No session matching id '{want}'");
        }
    }

    // ── choose ──────────────────────────────────────────────────────────────
    let chosen = if opts.continue_latest {
        filtered.into_iter().next().expect("non-empty checked")
    } else if opts.resume_pick {
        filtered.truncate(5);
        match pick_session(ctx, filtered)? {
            Some(s) => s,
            None => return Ok(0),
        }
    } else if filtered.len() == 1 {
        filtered.into_iter().next().expect("len checked")
    } else {
        match pick_session(ctx, filtered)? {
            Some(s) => s,
            None => return Ok(0),
        }
    };

    // ── extract ─────────────────────────────────────────────────────────────
    let events = extract_events(&chosen)?;
    if events.is_empty() {
        anyhow::bail!(
            "Could not extract any conversation events from session {} ({})",
            &chosen.id[..8.min(chosen.id.len())],
            chosen.source.as_str()
        );
    }

    let digest = build_digest(&chosen, &events, unix_now(), DIGEST_BUDGET_BYTES);

    // ── deliver ─────────────────────────────────────────────────────────────
    if opts.print {
        println!("{digest}");
        return Ok(0);
    }

    let ok = ctx.prompt.confirm(
        &format!(
            "Start a new Claude session seeded with this {} session ({} events)?",
            chosen.source.as_str(),
            events.len()
        ),
        true,
    )?;
    if !ok {
        ctx.output.info("Cancelled.");
        return Ok(0);
    }

    let mut launch_args = vec![digest];
    if opts.yolo {
        launch_args.push("--dangerously-skip-permissions".into());
    }
    launch_args.extend(opts.forward);
    launch_seeded_session(ctx, &profile, launch_args)
}

fn pick_session(
    ctx: &mut Context,
    filtered: Vec<ForeignSessionSummary>,
) -> anyhow::Result<Option<ForeignSessionSummary>> {
    if filtered.len() == 1 {
        return Ok(Some(filtered.into_iter().next().expect("len checked")));
    }
    let now = unix_now();
    let items: Vec<String> = filtered
        .iter()
        .map(|s| {
            let age = format_age(now.saturating_sub(s.last_modified));
            let title = display_title(s);
            let label = sanitize_str(&truncate_display(&title, 40));
            format!(
                "[{}] {} / {} {}",
                s.source.as_str(),
                label,
                &s.id[..8.min(s.id.len())],
                age
            )
        })
        .collect();
    match ctx
        .prompt
        .select_opt("Select session to hand off", &items, 0)?
    {
        None => {
            ctx.output.info("Cancelled.");
            Ok(None)
        }
        Some(i) => Ok(Some(
            filtered.into_iter().nth(i).expect("picker index in range"),
        )),
    }
}

fn extract_events(summary: &ForeignSessionSummary) -> anyhow::Result<Vec<DigestEvent>> {
    match summary.source {
        HandoffSource::Codex => {
            let path = summary
                .path
                .as_ref()
                .expect("codex summaries always carry a path");
            codex::extract_codex_events(path)
        }
        HandoffSource::Agy => {
            let dir = crate::adapters::handoff::agy_home()
                .expect("agy summaries only exist when the store was found");
            agy::extract_agy_events(&dir, &summary.id)
        }
    }
}

/// Spawn Claude interactively under `profile` with the digest as the initial
/// prompt. Mirrors the channel-server launch recipe (routing + launcher
/// modules directly) rather than reaching into `application::entrypoint`.
fn launch_seeded_session(
    ctx: &mut Context,
    profile: &str,
    launch_args: Vec<String>,
) -> anyhow::Result<i32> {
    let target = crate::routing::resolver::route_profile(profile, &ctx.catalog, &ctx.config)
        .map_err(|e| anyhow::anyhow!("Failed to resolve profile '{profile}': {e}"))?;
    let env = crate::launcher::envkit::build_auth_environment(&target, &ctx.secrets)?;
    crate::launcher::binary::run_session(
        &ctx.paths,
        &target,
        &launch_args,
        &env,
        crate::launcher::binary::SessionOptions {
            suppress_banner: false,
        },
        None,
    )
}

fn discover_codex_titled() -> Vec<ForeignSessionSummary> {
    let mut out = codex::discover_codex(200);
    for s in &mut out {
        if s.title.is_none() {
            s.title = codex::preview_user_message(s.path.as_deref().unwrap_or(&PathBuf::new()));
        }
    }
    out
}

fn discover_agy_titled() -> Vec<ForeignSessionSummary> {
    agy::discover_agy(200)
}

fn display_title(s: &ForeignSessionSummary) -> String {
    if let Some(t) = &s.title
        && !t.trim().is_empty()
    {
        return t.clone();
    }
    s.cwd
        .as_deref()
        .and_then(|c| std::path::Path::new(c).file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "(no preview)".into())
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_source_positional_and_short_flags() {
        let o = parse_handoff_args(&args(&["codex", "-c", "--yolo"])).unwrap();
        assert_eq!(o.source, Some(HandoffSource::Codex));
        assert!(o.continue_latest);
        assert!(o.yolo);
        assert!(o.forward.is_empty());
    }

    #[test]
    fn parses_resume_flag_and_value_flags() {
        let o = parse_handoff_args(&args(&["-r", "--cwd=/tmp", "--profile", "zai"])).unwrap();
        assert!(o.resume_pick);
        assert_eq!(o.cwd.as_deref(), Some("/tmp"));
        assert_eq!(o.profile.as_deref(), Some("zai"));

        let o = parse_handoff_args(&args(&["--id", "abc"])).unwrap();
        assert_eq!(o.id.as_deref(), Some("abc"));
        assert!(!o.resume_pick && !o.continue_latest);
    }

    #[test]
    fn unknown_flags_forward() {
        let o = parse_handoff_args(&args(&["--model", "opus", "--verbose"])).unwrap();
        assert_eq!(o.forward, args(&["--model", "opus", "--verbose"]));
    }

    #[test]
    fn continue_and_resume_conflict() {
        assert!(parse_handoff_args(&args(&["-c", "-r"])).is_err());
    }

    #[test]
    fn continue_conflicts_with_id() {
        assert!(parse_handoff_args(&args(&["-c", "--id", "x"])).is_err());
    }

    #[test]
    fn source_twice_rejected() {
        assert!(parse_handoff_args(&args(&["codex", "agy"])).is_err());
    }

    #[test]
    fn missing_value_rejected() {
        assert!(parse_handoff_args(&args(&["--id"])).is_err());
    }
}
