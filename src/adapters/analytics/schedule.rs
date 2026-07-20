//! Self-scheduling ingestion (R1).
//!
//! Installs an OS-level periodic job that runs `claudy analytics ingest` once
//! per hour, so the analytics DB never silently freezes when the operator stops
//! invoking ingest by hand.
//!
//! This reuses the same proven mechanisms as the channel `ServiceManager`
//! (atomic file writes via `config::atomic`, generated plist/unit files,
//! `launchctl`/`systemctl` invocation, an enriched `PATH`), but is kept as a
//! focused, separate module rather than overloading the channel trait: a
//! periodic one-shot job (macOS `StartInterval`+`RunAtLoad`, Linux `.timer`)
//! has different lifecycle semantics from a long-running daemon
//! (`KeepAlive`/`Restart`). See SPEC R1 design decision.

use crate::domain::commands::ScheduleAction;
use crate::domain::context::Context;
use std::path::PathBuf;
use std::process::Command;

/// Hourly cadence, in seconds.
const INTERVAL_SECS: u64 = 3600;
#[cfg(target_os = "macos")]
const LAUNCH_LABEL: &str = "com.claudy.analytics.ingest";
#[cfg(target_os = "linux")]
const SYSTEMD_UNIT: &str = "claudy-analytics-ingest";

pub fn run_schedule(ctx: &mut Context, action: ScheduleAction) -> anyhow::Result<i32> {
    let claudy_bin = std::env::current_exe()?;
    let log_dir = analytics_log_dir(ctx);

    match action {
        ScheduleAction::Install => install(&claudy_bin, &log_dir),
        ScheduleAction::Uninstall => uninstall(),
        ScheduleAction::Status => status(),
    }
}

fn analytics_log_dir(ctx: &Context) -> PathBuf {
    PathBuf::from(&ctx.paths.claudy_home).join("logs")
}

#[cfg(target_os = "macos")]
fn install(claudy_bin: &std::path::Path, log_dir: &std::path::Path) -> anyhow::Result<i32> {
    // Ensure the log dir exists so launchd can open StandardOutPath/ErrorPath.
    std::fs::create_dir_all(log_dir)?;
    let plist_path = launch_plist_path()?;
    let plist = generate_plist(claudy_bin, log_dir);
    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::config::atomic::write_atomic(&plist_path.to_string_lossy(), plist.as_bytes(), 0o644)?;

    // Best-effort: unload any previous copy, then load the new one.
    let _ = Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .output();
    let out = Command::new("launchctl")
        .args(["load", "-w", &plist_path.to_string_lossy()])
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "launchctl load failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    println!("Installed hourly ingestion scheduler ({LAUNCH_LABEL}).");
    println!(
        "Logs: {}/analytics-ingest.stdout.log (stderr: analytics-ingest.stderr.log)",
        log_dir.display(),
    );
    Ok(0)
}

#[cfg(target_os = "macos")]
fn uninstall() -> anyhow::Result<i32> {
    let plist_path = launch_plist_path()?;
    if plist_path.exists() {
        let _ = Command::new("launchctl")
            .args(["unload", &plist_path.to_string_lossy()])
            .output();
        std::fs::remove_file(&plist_path).ok();
        println!("Removed ingestion scheduler ({LAUNCH_LABEL}).");
    } else {
        println!("Ingestion scheduler is not installed.");
    }
    Ok(0)
}

#[cfg(target_os = "macos")]
fn status() -> anyhow::Result<i32> {
    let plist_path = launch_plist_path()?;
    let installed = plist_path.exists();
    let loaded = if installed {
        Command::new("launchctl")
            .args(["list", LAUNCH_LABEL])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    };
    println!(
        "ingestion scheduler: {} ({})",
        if loaded {
            "loaded & scheduled"
        } else if installed {
            "installed (not loaded)"
        } else {
            "not installed"
        },
        LAUNCH_LABEL,
    );
    Ok(0)
}

#[cfg(target_os = "macos")]
fn launch_plist_path() -> anyhow::Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{LAUNCH_LABEL}.plist")))
}

#[cfg(target_os = "macos")]
fn generate_plist(claudy_bin: &std::path::Path, log_dir: &std::path::Path) -> String {
    let bin = claudy_bin.display();
    let stdout_log = log_dir
        .join("analytics-ingest.stdout.log")
        .to_string_lossy()
        .to_string();
    let stderr_log = log_dir
        .join("analytics-ingest.stderr.log")
        .to_string_lossy()
        .to_string();
    let env_path = build_service_path();
    let home_dir = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LAUNCH_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{bin}</string>
        <string>analytics</string>
        <string>ingest</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>StartInterval</key>
    <integer>{INTERVAL_SECS}</integer>
    <key>StandardOutPath</key>
    <string>{stdout_log}</string>
    <key>StandardErrorPath</key>
    <string>{stderr_log}</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>{env_path}</string>
        <key>HOME</key>
        <string>{home_dir}</string>
    </dict>
</dict>
</plist>
"#
    )
}

#[cfg(target_os = "linux")]
fn install(claudy_bin: &std::path::Path, log_dir: &std::path::Path) -> anyhow::Result<i32> {
    // Ensure the log dir exists so the oneshot service can append its log.
    std::fs::create_dir_all(log_dir)?;
    let (service_path, timer_path) = systemd_unit_paths()?;
    std::fs::create_dir_all(service_path.parent().unwrap_or(std::path::Path::new(".")))?;
    let service = generate_service(claudy_bin, log_dir);
    let timer = generate_timer();
    crate::config::atomic::write_atomic(
        &service_path.to_string_lossy(),
        service.as_bytes(),
        0o644,
    )?;
    crate::config::atomic::write_atomic(&timer_path.to_string_lossy(), timer.as_bytes(), 0o644)?;

    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();
    let out = Command::new("systemctl")
        .args([
            "--user",
            "enable",
            "--now",
            &format!("{SYSTEMD_UNIT}.timer"),
        ])
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "systemctl enable failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    println!("Installed hourly ingestion scheduler ({SYSTEMD_UNIT}.timer).");
    println!("Logs: {}/analytics-ingest.log", log_dir.display());
    Ok(0)
}

#[cfg(target_os = "linux")]
fn uninstall() -> anyhow::Result<i32> {
    let (service_path, timer_path) = systemd_unit_paths()?;
    let _ = Command::new("systemctl")
        .args([
            "--user",
            "disable",
            "--now",
            &format!("{SYSTEMD_UNIT}.timer"),
        ])
        .output();
    let mut removed = false;
    if timer_path.exists() {
        std::fs::remove_file(&timer_path).ok();
        removed = true;
    }
    if service_path.exists() {
        std::fs::remove_file(&service_path).ok();
        removed = true;
    }
    if removed {
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output();
        println!("Removed ingestion scheduler ({SYSTEMD_UNIT}.timer).");
    } else {
        println!("Ingestion scheduler is not installed.");
    }
    Ok(0)
}

#[cfg(target_os = "linux")]
fn status() -> anyhow::Result<i32> {
    let (_, timer_path) = systemd_unit_paths()?;
    let enabled = Command::new("systemctl")
        .args(["--user", "is-enabled", &format!("{SYSTEMD_UNIT}.timer")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "ingestion scheduler: {} ({SYSTEMD_UNIT}.timer, installed={})",
        if enabled { "enabled" } else { "not enabled" },
        timer_path.exists(),
    );
    Ok(0)
}

#[cfg(target_os = "linux")]
fn systemd_unit_paths() -> anyhow::Result<(PathBuf, PathBuf)> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    let dir = home.join(".config").join("systemd").join("user");
    Ok((
        dir.join(format!("{SYSTEMD_UNIT}.service")),
        dir.join(format!("{SYSTEMD_UNIT}.timer")),
    ))
}

#[cfg(target_os = "linux")]
fn generate_service(claudy_bin: &std::path::Path, log_dir: &std::path::Path) -> String {
    let exec = claudy_bin.display();
    let log_file = log_dir
        .join("analytics-ingest.log")
        .to_string_lossy()
        .to_string();
    format!(
        r#"[Unit]
Description=Claudy analytics ingestion (hourly)
After=network.target

[Service]
Type=oneshot
ExecStart={exec} analytics ingest
StandardOutput=append:{log_file}
StandardError=append:{log_file}
"#
    )
}

#[cfg(target_os = "linux")]
fn generate_timer() -> String {
    let interval_min = INTERVAL_SECS / 60;
    format!(
        r#"[Unit]
Description=Run Claudi analytics ingestion hourly

[Timer]
OnBootSec=2min
OnUnitActiveSec={interval_min}min
Persistent=true

[Install]
WantedBy=timers.target
"#
    )
}

// ── Unsupported platform fallback ──

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn install(_claudy_bin: &std::path::Path, _log_dir: &std::path::Path) -> anyhow::Result<i32> {
    unsupported()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn uninstall() -> anyhow::Result<i32> {
    unsupported()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn status() -> anyhow::Result<i32> {
    unsupported()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn unsupported() -> anyhow::Result<i32> {
    println!("Scheduled ingestion is not yet supported on this platform.");
    println!("Run 'claudy analytics ingest' manually, or via your OS task scheduler:");
    println!("  claudy analytics ingest");
    Ok(1)
}

/// Build a PATH string including common binary locations (mirrors the channel
/// service path builder so the scheduler can locate the Claude CLI if needed).
/// macOS-only: the LaunchAgent plist needs an explicit PATH; systemd units
/// inherit the user environment.
#[cfg(target_os = "macos")]
fn build_service_path() -> String {
    let home = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let current_path = std::env::var("PATH").unwrap_or_default();
    let mut paths: Vec<String> = current_path.split(':').map(|s| s.to_string()).collect();
    let extras = [
        format!("{home}/.local/bin"),
        format!("{home}/bin"),
        "/usr/local/bin".to_string(),
        "/opt/homebrew/bin".to_string(),
    ];
    for extra in extras {
        if !paths.contains(&extra) {
            paths.push(extra);
        }
    }
    paths.join(":")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn test_plist_is_periodic_not_daemon() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bin = tmp.path().join("claudy");
        let log_dir = tmp.path().join("logs");
        let plist = generate_plist(&bin, &log_dir);

        // Periodic one-shot markers (R1 intent), not a KeepAlive daemon.
        assert!(plist.contains("<key>Label</key>"));
        assert!(plist.contains(LAUNCH_LABEL));
        assert!(plist.contains("<key>StartInterval</key>"));
        assert!(plist.contains(&format!("<integer>{INTERVAL_SECS}</integer>")));
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(!plist.contains("KeepAlive"), "timer must not set KeepAlive");
        assert!(plist.contains("analytics"));
        assert!(plist.contains("ingest"));
        assert!(plist.contains("EnvironmentVariables"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_systemd_units_are_periodic() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bin = tmp.path().join("claudy");
        let log_dir = tmp.path().join("logs");
        let service = generate_service(&bin, &log_dir);
        let timer = generate_timer();

        assert!(service.contains("Type=oneshot"));
        assert!(service.contains("ExecStart="));
        assert!(service.contains("analytics ingest"));
        assert!(timer.contains("OnUnitActiveSec="));
        assert!(timer.contains("Persistent=true"));
        assert!(timer.contains("WantedBy=timers.target"));
    }
}
