use std::path::PathBuf;
use std::process::Command;

use super::service::{ServiceConfig, ServiceManager};

pub struct SystemdServiceManager {
    config: ServiceConfig,
    unit_path: PathBuf,
}

impl SystemdServiceManager {
    pub fn new(config: ServiceConfig) -> Self {
        let home = dirs::home_dir().expect("home directory");
        let unit_dir = home.join(".config").join("systemd").join("user");
        let unit_path = unit_dir.join("claudy-channel.service");
        Self { config, unit_path }
    }

    fn generate_unit(&self, auto_start: bool) -> String {
        let mut exec_args = format!(
            "{} channel serve --listen {}",
            self.config.claudy_bin_path.display(),
            self.config.listen_addr
        );
        if let Some(ref profile) = self.config.profile {
            exec_args.push_str(&format!(" --profile {}", profile));
        }

        let wanted_by = if auto_start { "default.target" } else { "" };

        format!(
            r#"[Unit]
Description=Claudy Channel Server
After=network.target

[Service]
Type=simple
ExecStart={exec_args}
Restart=on-failure
RestartSec=5

[Install]
WantedBy={wanted_by}
"#
        )
    }
}

impl ServiceManager for SystemdServiceManager {
    fn install(&self) -> anyhow::Result<()> {
        let unit = self.generate_unit(false);
        if let Some(parent) = self.unit_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.unit_path, unit)?;
        Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output()?;
        Ok(())
    }

    fn start(&self) -> anyhow::Result<()> {
        self.install()?;
        let output = Command::new("systemctl")
            .args(["--user", "start", "claudy-channel"])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "systemctl start failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn stop(&self) -> anyhow::Result<()> {
        let output = Command::new("systemctl")
            .args(["--user", "stop", "claudy-channel"])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "systemctl stop failed: {}",
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
        self.install()?;
        let unit = self.generate_unit(true);
        std::fs::write(&self.unit_path, unit)?;
        Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output()?;
        let output = Command::new("systemctl")
            .args(["--user", "enable", "--now", "claudy-channel"])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "systemctl enable failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn disable(&self) -> anyhow::Result<()> {
        let _ = self.stop();
        let output = Command::new("systemctl")
            .args(["--user", "disable", "claudy-channel"])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "systemctl disable failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
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
    fn test_generate_unit_contains_exec_start() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path());
        let mgr = SystemdServiceManager {
            config,
            unit_path: tmp.path().join("test.service"),
        };

        let unit = mgr.generate_unit(true);

        assert!(unit.contains("ExecStart="));
        assert!(unit.contains("channel serve"));
        assert!(unit.contains("127.0.0.1:3456"));
        assert!(unit.contains("--profile zai"));
        assert!(unit.contains("WantedBy=default.target"));
    }
}
