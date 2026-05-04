use std::path::PathBuf;
use std::process::Command;

use super::service::{ServiceConfig, ServiceManager};

pub struct LaunchdServiceManager {
    config: ServiceConfig,
    plist_path: PathBuf,
}

impl LaunchdServiceManager {
    pub fn new(config: ServiceConfig) -> Self {
        let home = dirs::home_dir().expect("home directory");
        let plist_path = home
            .join("Library")
            .join("LaunchAgents")
            .join("com.claudy.channel.plist");
        Self { config, plist_path }
    }

    fn generate_plist(&self, run_at_load: bool) -> String {
        let mut args = vec![
            xml_arg(self.config.claudy_bin_path.display()),
            xml_arg("channel"),
            xml_arg("serve"),
            xml_arg("--listen"),
            xml_arg(&self.config.listen_addr),
        ];
        if let Some(ref profile) = self.config.profile {
            args.push(xml_arg("--profile"));
            args.push(xml_arg(profile));
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.claudy.channel</string>
    <key>ProgramArguments</key>
    <array>
{args}
    </array>
    <key>RunAtLoad</key>
    <{run_at_load}/>
    <key>KeepAlive</key>
    <true/>
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
"#,
            args = args.join("\n"),
            run_at_load = if run_at_load { "true" } else { "false" },
            stdout_log = self.config.log_dir.join("stdout.log").display(),
            stderr_log = self.config.log_dir.join("stderr.log").display(),
            env_path = build_service_path(),
            home_dir = dirs::home_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
        )
    }
}

/// Build a PATH string that includes common binary locations where Claude CLI might live.
fn build_service_path() -> String {
    let home = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let current_path = std::env::var("PATH").unwrap_or_default();
    let mut paths: Vec<String> = current_path.split(':').map(|s| s.to_string()).collect();

    // Ensure Claude CLI locations are included
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

fn xml_arg(value: impl std::fmt::Display) -> String {
    format!("        <string>{}</string>", value)
}

impl ServiceManager for LaunchdServiceManager {
    fn install(&self) -> anyhow::Result<()> {
        let plist = self.generate_plist(false);
        if let Some(parent) = self.plist_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.plist_path, plist)?;
        Ok(())
    }

    fn start(&self) -> anyhow::Result<()> {
        self.install()?;

        let output = Command::new("launchctl")
            .args(["load", "-w", &self.plist_path.to_string_lossy()])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "launchctl load failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn stop(&self) -> anyhow::Result<()> {
        let output = Command::new("launchctl")
            .args(["unload", &self.plist_path.to_string_lossy()])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "launchctl unload failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn is_running(&self) -> anyhow::Result<bool> {
        let pid_str = std::fs::read_to_string(&self.config.pid_file).ok();
        match pid_str {
            Some(s) => {
                let pid: i32 = s.trim().parse().unwrap_or(0);
                Ok(pid > 0 && unsafe { libc::kill(pid, 0) } == 0)
            }
            None => Ok(false),
        }
    }

    fn enable(&self) -> anyhow::Result<()> {
        let running = self.is_running().unwrap_or(false);
        if running {
            let _ = self.stop();
        }

        let plist = self.generate_plist(true);
        if let Some(parent) = self.plist_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.plist_path, &plist)?;

        let output = Command::new("launchctl")
            .args(["load", "-w", &self.plist_path.to_string_lossy()])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "launchctl load failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn disable(&self) -> anyhow::Result<()> {
        let _ = self.stop();

        let plist = self.generate_plist(false);
        std::fs::write(&self.plist_path, plist)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_config(tmp: &Path) -> ServiceConfig {
        ServiceConfig {
            listen_addr: "127.0.0.1:3456".to_string(),
            profile: Some("zai".to_string()),
            claudy_bin_path: tmp.join("bin").join("claudy"),
            log_dir: tmp.join("logs"),
            pid_file: tmp.join("pid"),
        }
    }

    #[test]
    fn test_generate_plist_contains_required_keys() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path());
        let mgr = LaunchdServiceManager {
            config,
            plist_path: tmp.path().join("test.plist"),
        };

        let plist = mgr.generate_plist(false);

        assert!(plist.contains("<key>Label</key>"));
        assert!(plist.contains("com.claudy.channel"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<false/>"));
        assert!(plist.contains("<key>KeepAlive</key>"));
        assert!(plist.contains("127.0.0.1:3456"));
        assert!(plist.contains("--profile"));
        assert!(
            plist.contains("EnvironmentVariables"),
            "plist must set PATH env var"
        );
        assert!(plist.contains("zai"));
        assert!(plist.contains("channel"));
        assert!(plist.contains("serve"));
    }

    #[test]
    fn test_generate_plist_run_at_load_true() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path());
        let mgr = LaunchdServiceManager {
            config,
            plist_path: tmp.path().join("test.plist"),
        };

        let plist = mgr.generate_plist(true);

        let idx = plist
            .find("<key>RunAtLoad</key>")
            .expect("RunAtLoad key present");
        let after = &plist[idx + "<key>RunAtLoad</key>".len()..];
        assert!(
            after.starts_with("\n    <true/>"),
            "RunAtLoad should be true, got: {}",
            &after[..30.min(after.len())]
        );
    }

    #[test]
    fn test_generate_plist_no_profile() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut config = test_config(tmp.path());
        config.profile = None;
        let mgr = LaunchdServiceManager {
            config,
            plist_path: tmp.path().join("test.plist"),
        };

        let plist = mgr.generate_plist(false);
        assert!(!plist.contains("--profile"));
    }
}
