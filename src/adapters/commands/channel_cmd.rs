use crate::adapters::channel::service::{self, ServiceConfig};
use crate::config::registry::{open_registry, write_registry};
use crate::config::vault::{load_vault, persist_vault, redact_credential};
use crate::domain::commands::ChannelAction;
use crate::domain::context::Context;

const VALID_PLATFORMS: &[&str] = &["telegram", "slack", "discord"];

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

const TELEGRAM_BOT_TOKEN_KEY: &str = "TELEGRAM_BOT_TOKEN";
const SLACK_BOT_TOKEN_KEY: &str = "SLACK_BOT_TOKEN";
const DISCORD_BOT_TOKEN_KEY: &str = "DISCORD_BOT_TOKEN";

fn bot_token_key(platform: &str) -> &'static str {
    match platform {
        "telegram" => TELEGRAM_BOT_TOKEN_KEY,
        "slack" => SLACK_BOT_TOKEN_KEY,
        "discord" => DISCORD_BOT_TOKEN_KEY,
        _ => "",
    }
}

fn validate_platform(platform: &str) -> anyhow::Result<()> {
    if VALID_PLATFORMS.contains(&platform) {
        Ok(())
    } else {
        anyhow::bail!(
            "Unknown platform '{}'. Valid platforms: {}",
            platform,
            VALID_PLATFORMS.join(", ")
        )
    }
}

fn list_modes(modes_dir: &str) -> Vec<String> {
    std::fs::read_dir(modes_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_listen_addr(ctx: &Context, listen: Option<&str>) -> String {
    let config_addr = ctx.config.channel.listen_addr.clone();
    let default_addr = crate::config::registry::default_listen_addr();
    listen.map(|s| s.to_string()).unwrap_or_else(|| {
        if config_addr.is_empty() {
            default_addr
        } else {
            config_addr
        }
    })
}

fn build_service_config(
    ctx: &Context,
    listen: Option<&str>,
    profile: Option<&str>,
) -> anyhow::Result<ServiceConfig> {
    let listen_addr = resolve_listen_addr(ctx, listen);
    let claudy_bin = std::env::current_exe()?;
    Ok(ServiceConfig {
        listen_addr,
        profile: profile.filter(|p| !p.is_empty()).map(|p| p.to_string()),
        claudy_bin_path: claudy_bin,
        log_dir: ctx.paths.channel_logs_dir.clone().into(),
        pid_file: ctx.paths.channel_pid_file.clone().into(),
    })
}

pub fn run_channel(
    ctx: &mut Context,
    action: ChannelAction,
    profile: Option<&str>,
    listen: Option<&str>,
) -> anyhow::Result<i32> {
    match action {
        ChannelAction::Serve => run_serve(ctx, profile, listen),
        ChannelAction::Start => run_start(ctx, profile, listen),
        ChannelAction::Stop => run_stop(ctx),
        ChannelAction::Restart => run_restart(ctx, profile, listen),
        ChannelAction::Status => run_status(ctx),
        ChannelAction::Add { platform } => run_add(ctx, &platform),
        ChannelAction::Remove { platform } => run_remove(ctx, &platform),
        ChannelAction::Enable => run_enable(ctx, profile, listen),
        ChannelAction::Disable => run_disable(ctx),
    }
}

fn run_serve(
    ctx: &mut Context,
    profile: Option<&str>,
    listen: Option<&str>,
) -> anyhow::Result<i32> {
    let listen_addr = resolve_listen_addr(ctx, listen);

    if let Some(p) = profile.filter(|p| !p.is_empty()) {
        ctx.config.channel.default_profile = p.to_string();
    }

    let has_any_profile = !ctx.config.channel.default_profile.is_empty()
        || !ctx.config.channel.platform_profiles.is_empty();
    if !has_any_profile {
        ctx.output.error(
            "No profile configured. Use --profile or set channel.default_profile / channel.platform_profiles in config.",
        );
        return Ok(1);
    }

    ctx.output
        .info(&format!("Serving channel on {}...", listen_addr,));

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { crate::adapters::channel::server::run(ctx, &listen_addr).await })
}

fn run_start(
    ctx: &mut Context,
    profile: Option<&str>,
    listen: Option<&str>,
) -> anyhow::Result<i32> {
    let svc_config = build_service_config(ctx, listen, profile)?;
    let mgr = service::platform_service(svc_config)?;

    if let Ok(true) = mgr.is_running() {
        ctx.output.info("Channel server is already running.");
        return Ok(0);
    }

    ctx.output.info("Starting channel server...");
    mgr.start()?;
    ctx.output.success("Channel server started.");
    Ok(0)
}

fn run_stop(ctx: &mut Context) -> anyhow::Result<i32> {
    let svc_config = build_service_config(ctx, None, None)?;
    let mgr = service::platform_service(svc_config)?;

    match mgr.is_running() {
        Ok(true) => {}
        _ => {
            ctx.output.warn("Channel server is not running.");
            return Ok(1);
        }
    }

    ctx.output.info("Stopping channel server...");
    mgr.stop()?;
    ctx.output.success("Channel server stopped.");
    Ok(0)
}

fn run_restart(
    ctx: &mut Context,
    profile: Option<&str>,
    listen: Option<&str>,
) -> anyhow::Result<i32> {
    let svc_config = build_service_config(ctx, listen, profile)?;
    let mgr = service::platform_service(svc_config)?;

    let running = mgr.is_running().unwrap_or(false);
    if running {
        ctx.output.info("Stopping channel server...");
        mgr.stop()?;
    }

    ctx.output.info("Starting channel server...");
    mgr.start()?;
    ctx.output.success("Channel server restarted.");
    Ok(0)
}

fn run_enable(
    ctx: &mut Context,
    profile: Option<&str>,
    listen: Option<&str>,
) -> anyhow::Result<i32> {
    let svc_config = build_service_config(ctx, listen, profile)?;
    let mgr = service::platform_service(svc_config)?;

    ctx.output
        .info("Enabling channel service (auto-start on login)...");
    mgr.enable()?;
    ctx.output
        .success("Channel service enabled. It will start automatically on login.");
    Ok(0)
}

fn run_disable(ctx: &mut Context) -> anyhow::Result<i32> {
    let svc_config = build_service_config(ctx, None, None)?;
    let mgr = service::platform_service(svc_config)?;

    ctx.output.info("Disabling channel service...");
    mgr.disable()?;
    ctx.output
        .success("Channel service disabled. It will no longer start automatically.");
    Ok(0)
}

fn run_status(ctx: &mut Context) -> anyhow::Result<i32> {
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
    let platforms = [
        ("telegram", "TELEGRAM_BOT_TOKEN", None, None),
        ("slack", "SLACK_BOT_TOKEN", Some("SLACK_SIGNING_SECRET"), Some("SLACK_APP_TOKEN")),
        (
            "discord",
            "DISCORD_BOT_TOKEN",
            Some("DISCORD_APPLICATION_ID"),
            None,
        ),
    ];

    ctx.output.header("Channels");
    for (platform, token_key, extra_key, socket_key) in &platforms {
        let has_token = ctx
            .secrets
            .get(*token_key)
            .is_some_and(|v| !v.is_empty());
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

fn run_add(ctx: &mut Context, platform: &str) -> anyhow::Result<i32> {
    validate_platform(platform)?;

    let key = bot_token_key(platform);
    let existing = ctx.secrets.get(key).cloned().unwrap_or_default();
    let display_label = format!(
        "{} bot token{}",
        platform,
        if existing.is_empty() {
            String::new()
        } else {
            format!(" (current: {})", redact_credential(&existing))
        }
    );

    let token = if existing.is_empty() {
        let t = ctx.prompt.prompt_secret(&display_label)?;
        if t.trim().is_empty() {
            ctx.output.error("Bot token is required.");
            return Ok(1);
        }
        t.trim().to_string()
    } else {
        let t = ctx.prompt.prompt_secret_opt(&display_label)?;
        match t {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => existing.clone(),
        }
    };

    let current_users = ctx
        .config
        .channel
        .platform_allowed_users
        .get(platform)
        .cloned()
        .unwrap_or_default()
        .join(", ");
    let users_label = format!(
        "Allowed user IDs/usernames (comma-separated){}",
        if current_users.is_empty() {
            String::new()
        } else {
            format!(" [{}]", current_users)
        }
    );
    let users_input = ctx
        .prompt
        .prompt_opt(&users_label, &current_users)?
        .unwrap_or_default();
    let allowed_users: Vec<String> = users_input
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if allowed_users.is_empty() {
        ctx.output
            .error("At least one allowed user ID or username is required.");
        return Ok(1);
    }

    let mut cfg = open_registry(&ctx.paths.config_file)?;

    let current_profile = cfg
        .channel
        .platform_profiles
        .get(platform)
        .cloned()
        .unwrap_or_default();
    let default_profile = if current_profile.is_empty() {
        cfg.channel.default_profile.clone()
    } else {
        current_profile.clone()
    };
    let profile_label = format!("Provider profile for {} [{}]", platform, default_profile);
    let profile_input = ctx
        .prompt
        .prompt_opt(&profile_label, &default_profile)?
        .unwrap_or_default();

    let available_modes = list_modes(&ctx.paths.modes_dir);
    let current_mode = cfg
        .channel
        .platform_modes
        .get(platform)
        .cloned()
        .unwrap_or_else(|| cfg.channel.default_mode.clone());
    let mode_default = if current_mode.is_empty() {
        "default".to_string()
    } else {
        current_mode.clone()
    };
    let mode_hint = if available_modes.is_empty() {
        " (no modes found in ~/.claudy/modes/)"
    } else {
        &format!(" (available: {})", available_modes.join(", "))
    };
    let mode_label = format!("Mode for {} [{}]{}", platform, mode_default, mode_hint);
    let mode_input = ctx
        .prompt
        .prompt_opt(&mode_label, &mode_default)?
        .unwrap_or_default();

    let mut secrets = load_vault(&ctx.paths.secrets_file)?;
    secrets.insert(key.to_string(), token);
    prompt_extra_secrets(ctx, platform, &mut secrets)?;
    persist_vault(&ctx.paths.secrets_file, &secrets)?;
    ctx.output
        .success(&format!("Saved {} bot token.", platform));

    cfg.channel
        .platform_allowed_users
        .insert(platform.to_string(), allowed_users);
    if !profile_input.is_empty() {
        cfg.channel
            .platform_profiles
            .insert(platform.to_string(), profile_input);
    }
    if !mode_input.is_empty() && mode_input != "default" {
        cfg.channel
            .platform_modes
            .insert(platform.to_string(), mode_input);
    } else {
        cfg.channel.platform_modes.remove(platform);
    }
    if !cfg.channel.enabled_platforms.iter().any(|p| p == platform) {
        cfg.channel.enabled_platforms.push(platform.to_string());
    }
    write_registry(&ctx.paths.config_file, &cfg)?;

    ctx.output
        .success(&format!("Channel '{}' added and configured.", platform));
    Ok(0)
}

/// Prompt for platform-specific extra secrets (e.g. Discord APPLICATION_ID).
fn prompt_extra_secrets(
    ctx: &mut Context,
    platform: &str,
    secrets: &mut crate::config::vault::SecretVault,
) -> anyhow::Result<()> {
    let extras: &[(&str, &str)] = match platform {
        "discord" => &[("DISCORD_APPLICATION_ID", "Application ID"), ("DISCORD_PUBLIC_KEY", "Public Key")],
        "slack" => &[
            ("SLACK_SIGNING_SECRET", "Signing Secret"),
            ("SLACK_APP_TOKEN", "App Token (xapp-)"),
        ],
        _ => &[],
    };
    for &(key, label) in extras {
        let existing = secrets.get(key).cloned().unwrap_or_default();
        let display = format!(
            "{} {}{}",
            platform,
            label,
            if existing.is_empty() {
                String::new()
            } else {
                format!(" (current: {})", redact_credential(&existing))
            }
        );
        let value = if existing.is_empty() {
            let v = ctx.prompt.prompt_secret(&display)?;
            if v.trim().is_empty() {
                ctx.output.warn(&format!("{} not provided — Discord features may be limited.", label));
                continue;
            }
            v.trim().to_string()
        } else {
            let v = ctx.prompt.prompt_secret_opt(&display)?;
            match v {
                Some(s) if !s.trim().is_empty() => s.trim().to_string(),
                _ => existing,
            }
        };
        secrets.insert(key.to_string(), value);
    }
    Ok(())
}

fn run_remove(ctx: &mut Context, platform: &str) -> anyhow::Result<i32> {
    validate_platform(platform)?;

    let key = bot_token_key(platform);
    let mut secrets = load_vault(&ctx.paths.secrets_file)?;
    let had_secret = secrets.remove(key).is_some();
    if had_secret {
        persist_vault(&ctx.paths.secrets_file, &secrets)?;
        ctx.output
            .success(&format!("Removed {} bot token.", platform));
    }

    let mut cfg = open_registry(&ctx.paths.config_file)?;
    let before = cfg.channel.enabled_platforms.len();
    cfg.channel.enabled_platforms.retain(|p| p != platform);
    let was_enabled = cfg.channel.enabled_platforms.len() < before;
    if was_enabled {
        write_registry(&ctx.paths.config_file, &cfg)?;
    }

    if !had_secret && !was_enabled {
        ctx.output
            .info(&format!("Platform '{}' was not configured.", platform));
        return Ok(0);
    }

    ctx.output
        .success(&format!("Channel '{}' removed.", platform));
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::layout::AppPaths;
    use crate::config::registry::AppRegistry;
    use crate::config::vault::SecretVault;
    use crate::domain::commands::Options;
    use crate::providers::index as catalog;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct CaptureOutput {
        messages: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl CaptureOutput {
        fn new() -> Self {
            Self::default()
        }
    }

    impl crate::ports::ui_ports::OutputPort for CaptureOutput {
        fn header(&mut self, title: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(("header".into(), title.into()));
        }
        fn info(&mut self, msg: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(("info".into(), msg.into()));
        }
        fn success(&mut self, msg: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(("success".into(), msg.into()));
        }
        fn warn(&mut self, msg: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(("warn".into(), msg.into()));
        }
        fn error(&mut self, msg: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(("error".into(), msg.into()));
        }
        fn write_line(&mut self, msg: &str) -> std::io::Result<()> {
            self.messages
                .lock()
                .unwrap()
                .push(("line".into(), msg.into()));
            Ok(())
        }
    }

    struct StubPrompt;

    impl crate::ports::ui_ports::PrompterPort for StubPrompt {
        fn prompt(&mut self, _label: &str, _default: &str) -> anyhow::Result<String> {
            Ok(String::new())
        }
        fn prompt_opt(&mut self, _label: &str, _default: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }
        fn prompt_secret(&mut self, _label: &str) -> anyhow::Result<String> {
            Ok(String::new())
        }
        fn prompt_secret_opt(&mut self, _label: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }
        fn confirm(&mut self, _label: &str, _default_yes: bool) -> anyhow::Result<bool> {
            Ok(_default_yes)
        }
        fn confirm_opt(
            &mut self,
            _label: &str,
            _default_yes: bool,
        ) -> anyhow::Result<Option<bool>> {
            Ok(Some(_default_yes))
        }
        fn select(
            &mut self,
            _label: &str,
            _items: &[String],
            _default: usize,
        ) -> anyhow::Result<usize> {
            Ok(_default)
        }
        fn select_opt(
            &mut self,
            _label: &str,
            _items: &[String],
            _default: usize,
        ) -> anyhow::Result<Option<usize>> {
            Ok(Some(_default))
        }
    }

    fn make_test_context(pid_file_path: &str) -> Context {
        let catalog = catalog::load_index().expect("catalog should load");
        Context {
            paths: AppPaths {
                claudy_home: String::new(),
                config_dir: String::new(),
                data_dir: String::new(),
                cache_dir: String::new(),
                bin_dir: String::new(),
                config_file: String::new(),
                secrets_file: String::new(),
                manifest_file: String::new(),
                session_patch_dir: String::new(),
                update_cache_file: String::new(),
                modes_dir: String::new(),
                channel_dir: String::new(),
                channel_pid_file: pid_file_path.to_string(),
                channel_sessions_file: String::new(),
                channel_audit_file: String::new(),
                channel_logs_dir: String::new(),
                analytics_dir: "/tmp/test-analytics".to_string(),
                analytics_db: "/tmp/test-analytics/analytics.db".to_string(),
            },
            config: AppRegistry::default(),
            secrets: SecretVault::empty(),
            catalog,
            output: Box::new(CaptureOutput::new()),
            prompt: Box::new(StubPrompt),
            options: Options::default(),
        }
    }

    #[test]
    fn test_validate_platform_accepts_known_values() {
        assert!(validate_platform("telegram").is_ok());
        assert!(validate_platform("slack").is_ok());
        assert!(validate_platform("discord").is_ok());
    }

    #[test]
    fn test_validate_platform_rejects_unknown_value() {
        let err = validate_platform("teams").expect_err("teams should be rejected");
        assert!(err.to_string().contains("Unknown platform 'teams'"));
    }

    #[test]
    fn test_bot_token_key_mapping() {
        assert_eq!(bot_token_key("telegram"), "TELEGRAM_BOT_TOKEN");
        assert_eq!(bot_token_key("slack"), "SLACK_BOT_TOKEN");
        assert_eq!(bot_token_key("discord"), "DISCORD_BOT_TOKEN");
        assert_eq!(bot_token_key("unknown"), "");
    }

    #[test]
    fn test_list_modes_only_returns_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mode1 = dir.path().join("work");
        let mode2 = dir.path().join("personal");
        let file = dir.path().join("README.txt");
        std::fs::create_dir_all(&mode1).expect("create mode1");
        std::fs::create_dir_all(&mode2).expect("create mode2");
        std::fs::write(&file, "not-a-mode").expect("write non-directory file");

        let mut all_modes = list_modes(&dir.path().to_string_lossy());
        all_modes.sort();

        assert_eq!(all_modes, vec!["personal".to_string(), "work".to_string()]);
    }

    #[test]
    fn test_list_modes_returns_empty_for_missing_path() {
        let missing = "/tmp/claudy-modes-path-that-should-not-exist";
        let modes = list_modes(missing);
        assert!(modes.is_empty());
    }

    #[test]
    fn test_resolve_listen_addr_uses_default_when_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pid_path = dir.path().join("pid");
        let ctx = make_test_context(&pid_path.to_string_lossy());

        let addr = resolve_listen_addr(&ctx, None);
        assert_eq!(addr, "127.0.0.1:3456");
    }

    #[test]
    fn test_resolve_listen_addr_prefers_cli_over_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pid_path = dir.path().join("pid");
        let ctx = make_test_context(&pid_path.to_string_lossy());

        let addr = resolve_listen_addr(&ctx, Some("0.0.0.0:9999"));
        assert_eq!(addr, "0.0.0.0:9999");
    }

    #[test]
    fn test_run_serve_returns_one_when_no_profile_configured() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pid_path = dir.path().join("pid");
        let mut ctx = make_test_context(&pid_path.to_string_lossy());
        ctx.config.channel.default_profile.clear();
        ctx.config.channel.platform_profiles.clear();

        let code = run_serve(&mut ctx, None, Some("127.0.0.1:18080"))
            .expect("run_serve should return code");
        assert_eq!(code, 1);
    }
}
