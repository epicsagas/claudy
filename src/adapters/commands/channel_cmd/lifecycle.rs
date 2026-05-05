use crate::adapters::channel::service::{self, ServiceConfig};
use crate::domain::context::Context;

pub(super) fn resolve_listen_addr(ctx: &Context, listen: Option<&str>) -> String {
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

pub(super) fn run_serve(
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

pub(super) fn run_start(
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

pub(super) fn run_stop(ctx: &mut Context) -> anyhow::Result<i32> {
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

pub(super) fn run_restart(
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

pub(super) fn run_enable(
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

pub(super) fn run_disable(ctx: &mut Context) -> anyhow::Result<i32> {
    let svc_config = build_service_config(ctx, None, None)?;
    let mgr = service::platform_service(svc_config)?;

    ctx.output.info("Disabling channel service...");
    mgr.disable()?;
    ctx.output
        .success("Channel service disabled. It will no longer start automatically.");
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
