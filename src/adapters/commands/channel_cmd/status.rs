use crate::domain::context::Context;

use super::lifecycle::resolve_listen_addr;

#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    // On Windows, try opening the process; failure means it's not running
    let result = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
        .output();
    match result {
        Ok(out) => String::from_utf8_lossy(&out.stdout).contains(&pid.to_string()),
        Err(_) => false,
    }
}

pub(super) fn run_status(ctx: &mut Context) -> anyhow::Result<i32> {
    let pid_path = &ctx.paths.channel_pid_file;
    let pid_str = match std::fs::read_to_string(pid_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            ctx.output.info("Channel server is not running.");
            show_channel_config(ctx);
            return Ok(1);
        }
        Err(e) => return Err(e.into()),
    };

    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid PID file content: {}", pid_str.trim()))?;

    let running = is_process_running(pid);
    if running {
        let listen_addr = resolve_listen_addr(ctx, None);
        let health = ureq::Agent::new_with_config(
            ureq::Agent::config_builder()
                .timeout_global(Some(std::time::Duration::from_secs(2)))
                .build(),
        )
        .get(&format!("http://{}/health", listen_addr))
        .call();

        match health {
            Ok(_) => {
                ctx.output.success(&format!(
                    "Channel server is running (PID {}, listening on {})",
                    pid, listen_addr
                ));
            }
            Err(_) => {
                ctx.output.warn(&format!(
                    "Channel process exists (PID {}) but health check failed on {}",
                    pid, listen_addr
                ));
            }
        }
    } else {
        ctx.output.warn(&format!(
            "Channel PID file exists ({}) but process is not running",
            pid
        ));
        std::fs::remove_file(pid_path).ok();
    }

    show_channel_config(ctx);
    Ok(if running { 0 } else { 1 })
}

fn show_channel_config(ctx: &mut Context) {
    let cfg = &ctx.config.channel;
    let platforms: &[(&str, &str, Option<&str>, Option<&str>)] = &[
        ("telegram", "TELEGRAM_BOT_TOKEN", None, None),
        ("slack", "SLACK_BOT_TOKEN", None, Some("SLACK_APP_TOKEN")),
        ("discord", "DISCORD_BOT_TOKEN", None, None),
    ];

    ctx.output.header("Channels");
    for (platform, token_key, extra_key, socket_key) in platforms {
        let has_token = ctx.secrets.get(*token_key).is_some_and(|v| !v.is_empty());
        let has_extra = extra_key
            .map(|k| ctx.secrets.get(k).is_some_and(|v| !v.is_empty()))
            .unwrap_or(true);
        let has_socket = socket_key
            .map(|k| ctx.secrets.get(k).is_some_and(|v| !v.is_empty()))
            .unwrap_or(true);
        let configured = has_token && has_extra;
        let enabled = cfg.enabled_platforms.iter().any(|p| p == platform);

        let status = if configured && enabled {
            if !has_socket && socket_key.is_some() {
                "ready (no socket mode)"
            } else {
                "ready"
            }
        } else if configured {
            "configured (not enabled)"
        } else if has_token && !has_extra {
            "incomplete"
        } else {
            "not configured"
        };

        let profile = cfg.profile_for(platform);
        let mode = cfg
            .mode_for(platform)
            .unwrap_or_else(|| "default".to_string());
        let users = cfg.allowed_users_for(platform);

        let mut details = vec![status.to_string()];
        if !profile.is_empty() {
            details.push(format!("profile={}", profile));
        }
        if mode != "default" {
            details.push(format!("mode={}", mode));
        }
        if !users.is_empty() {
            details.push(format!("users={}", users.join(",")));
        }

        ctx.output
            .info(&format!("  {}: {}", platform, details.join(", ")));
    }
}
