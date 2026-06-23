use std::io::{IsTerminal, Write};
use std::time::Duration;

use crate::config::registry::open_registry;
use crate::domain::launch_blueprint::LaunchTarget;
use crate::launcher::args;

pub struct SessionOptions {
    pub suppress_banner: bool,
}

/// Launch Claude with the given target and environment.
pub fn run_session(
    paths: &crate::config::layout::AppPaths,
    target: &LaunchTarget,
    args: &[String],
    env: &[String],
    policy: SessionOptions,
    mode: Option<&str>,
) -> anyhow::Result<i32> {
    let args = args::normalize_claude_args(args);

    let config = open_registry(&paths.config_file)?;

    let (mut env, cleanup) = super::overlay::prepare_provider_env(target, &args, env, &config)?;

    if let Some(mode_name) = mode {
        let mode_dir = std::path::Path::new(&paths.modes_dir).join(mode_name);
        env.push(format!("CLAUDE_CONFIG_DIR={}", mode_dir.display()));
    }

    // Seed bundled skills (global + mode-specific)
    if let Some(home) = dirs::home_dir() {
        let global_skills = home.join(".claude").join("skills");
        crate::adapters::skill::seeder::install_skills(&global_skills);
        if let Some(mode_name) = mode {
            let mode_skills = std::path::Path::new(&paths.modes_dir)
                .join(mode_name)
                .join("skills");
            crate::adapters::skill::seeder::install_skills(&mode_skills);
        }
    }

    // Update check
    if let Some(msg) =
        crate::adapters::update::check::maybe_message(paths, crate::adapters::version::VALUE)
            .ok()
            .flatten()
            .filter(|_| is_tty_stderr())
    {
        let _ = writeln!(std::io::stderr(), "{}", msg);
    }

    if !policy.suppress_banner && is_tty_stdout() {
        print!("{}", crate::adapters::ui::output::banner(target, mode));
    }

    // Sanitize the session file before launch so cross-provider artifacts
    // (invalid thinking-block signatures and non-conforming server_tool_use IDs
    // written by ZAI/GLM or other providers) don't cause a 400 from the
    // Anthropic API on resume.
    sanitize_resume_session(&args);

    let claude_path = find_claude_cli()?;
    let code = exec_claude_session(&claude_path, &args, &env)?;

    cleanup();
    Ok(code)
}

pub fn launch_session(
    paths: &crate::config::layout::AppPaths,
    target: &LaunchTarget,
    args: &[String],
    env: &[String],
    options: SessionOptions,
    mode: Option<&str>,
) -> anyhow::Result<i32> {
    run_session(paths, target, args, env, options, mode)
}

pub fn find_claude_cli() -> anyhow::Result<std::path::PathBuf> {
    let current_exe = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::canonicalize(&p).ok());

    // 1. Try PATH-based lookup first
    for candidate in which::which_all("claude").into_iter().flatten() {
        let resolved = std::fs::canonicalize(&candidate).unwrap_or_else(|_| candidate.clone());
        if current_exe.as_ref().is_some_and(|exe| resolved == *exe) {
            continue;
        }
        return Ok(candidate);
    }

    // 2. Fallback: check well-known Claude CLI locations
    if let Some(home) = dirs::home_dir() {
        let fallbacks = [
            home.join(".local").join("bin").join("claude"),
            home.join("bin").join("claude"),
        ];
        for candidate in &fallbacks {
            if candidate.exists() {
                let resolved =
                    std::fs::canonicalize(candidate).unwrap_or_else(|_| candidate.clone());
                if current_exe.as_ref().is_none_or(|exe| resolved != *exe) {
                    return Ok(candidate.clone());
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "The Claude CLI binary could not be found. Please ensure it is installed and available in your PATH."
    ))
}

pub fn exec_claude_session(
    claude_path: &std::path::Path,
    args: &[String],
    env: &[String],
) -> anyhow::Result<i32> {
    let start = std::time::Instant::now();
    let status = spawn_claude(claude_path, args, env)?;
    let elapsed = start.elapsed();

    // Claude Code has a REPL bug where --continue/--resume crashes on startup
    // but exits with code 0. Detect via wall-clock: a crash exits in < 2s,
    // whereas a successful session restore keeps the process alive.
    if has_resume_flag(args) && elapsed < Duration::from_secs(2) {
        let fallback = strip_resume_flags(args);
        if fallback.len() != args.len() {
            let _ = writeln!(
                std::io::stderr(),
                "\n  WARNING: Session restore failed (known Claude Code bug). Starting fresh...\n"
            );
            let retry = spawn_claude(claude_path, &fallback, env)?;
            return Ok(retry.code().unwrap_or(1));
        }
    }

    Ok(status.code().unwrap_or(1))
}

fn spawn_claude(
    claude_path: &std::path::Path,
    args: &[String],
    env: &[String],
) -> anyhow::Result<std::process::ExitStatus> {
    let mut cmd = std::process::Command::new(claude_path);
    cmd.args(args);
    cmd.stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    for env_str in env {
        if let Some((key, value)) = env_str.split_once('=') {
            cmd.env(key, value);
        }
    }

    Ok(cmd.status()?)
}

/// Sanitize the session that is about to be resumed.
///
/// Inspects the forwarded args for `--resume <id>`/`-r <id>` (explicit
/// session) or `--continue`/`-c` (most-recent session for the current
/// directory) and runs both sanitizers so the Anthropic API does not reject
/// the conversation history with HTTP 400.
fn sanitize_resume_session(args: &[String]) {
    use crate::adapters::channel::sessions::{
        claude_projects_dir, find_most_recent_session_id_for_cwd,
        sanitize_session_server_tool_use_ids, sanitize_session_thinking_blocks,
    };

    let Some(projects_dir) = claude_projects_dir() else {
        return;
    };

    let session_id = extract_resume_session_id(args).or_else(|| {
        if has_continue_flag(args) {
            find_most_recent_session_id_for_cwd(&projects_dir)
        } else {
            None
        }
    });

    let Some(sid) = session_id else { return };

    let stderr_tty = is_tty_stderr();

    match sanitize_session_thinking_blocks(&projects_dir, &sid) {
        Ok(0) => {}
        Ok(n) if stderr_tty => {
            let _ = writeln!(
                std::io::stderr(),
                "  [claudy] patched {n} thinking block(s) before resume"
            );
        }
        Ok(_) => {}
        Err(e) => {
            let _ = writeln!(
                std::io::stderr(),
                "  [claudy] warning: could not patch thinking blocks: {e}"
            );
        }
    }

    match sanitize_session_server_tool_use_ids(&projects_dir, &sid) {
        Ok(0) => {}
        Ok(n) if stderr_tty => {
            let _ = writeln!(
                std::io::stderr(),
                "  [claudy] patched {n} server_tool_use ID(s) before resume"
            );
        }
        Ok(_) => {}
        Err(e) => {
            let _ = writeln!(
                std::io::stderr(),
                "  [claudy] warning: could not patch server_tool_use IDs: {e}"
            );
        }
    }
}

/// Extract the session UUID from `--resume <uuid>`, `-r <uuid>`, or `--resume=<uuid>`.
fn extract_resume_session_id(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--resume" | "-r" => {
                let next = args.get(i + 1)?;
                if !next.starts_with('-') {
                    return Some(next.clone());
                }
            }
            a if a.starts_with("--resume=") => {
                return Some(a["--resume=".len()..].to_string());
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn has_continue_flag(args: &[String]) -> bool {
    args.iter()
        .any(|a| matches!(a.as_str(), "--continue" | "-c"))
}

fn has_resume_flag(args: &[String]) -> bool {
    args.iter()
        .any(|a| matches!(a.as_str(), "--continue" | "--resume"))
}

fn strip_resume_flags(args: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--continue" => i += 1,
            "--resume" => {
                i += 1;
                if i < args.len() && !args[i].starts_with('-') {
                    i += 1;
                }
            }
            other => {
                out.push(other.to_owned());
                i += 1;
            }
        }
    }
    out
}

fn is_tty_stderr() -> bool {
    std::io::stderr().is_terminal()
}

fn is_tty_stdout() -> bool {
    std::io::stdout().is_terminal()
}
