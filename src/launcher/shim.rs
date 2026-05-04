use crate::config::layout::AppPaths;
use std::env;
use std::process::Command;

pub fn run_claude_shim(paths: &AppPaths, args: &[String]) -> anyhow::Result<i32> {
    let real_claude = locate_real_claude(paths)?;

    let mut cmd = Command::new(real_claude);
    cmd.args(args);

    let status = cmd.status()?;
    Ok(status.code().unwrap_or(1))
}

/// Find the real Claude binary, excluding ourselves in case we are the `claude` symlink.
pub fn locate_real_claude(paths: &AppPaths) -> anyhow::Result<String> {
    let current_exe = env::current_exe()
        .ok()
        .and_then(|p| std::fs::canonicalize(&p).ok());

    // Search PATH, skipping our own binary
    if let Ok(candidates) = which::which_all("claude") {
        for candidate in candidates {
            let resolved = std::fs::canonicalize(&candidate).unwrap_or_else(|_| candidate.clone());
            if let Some(ref exe) = current_exe
                && resolved == *exe
            {
                continue;
            }
            return Ok(candidate
                .to_str()
                .expect("candidate path should be valid UTF-8")
                .to_string());
        }
    }

    // Fallback to claude-real in bin dir
    let bin_claude = std::path::Path::new(&paths.bin_dir).join("claude-real");
    if bin_claude.exists() {
        return Ok(bin_claude
            .to_str()
            .expect("bin_claude path should be valid UTF-8")
            .to_string());
    }

    Err(anyhow::anyhow!(
        "The real Claude CLI binary could not be located. Please ensure it is installed correctly."
    ))
}
