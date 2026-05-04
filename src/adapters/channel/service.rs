use std::path::PathBuf;

pub struct ServiceConfig {
    pub listen_addr: String,
    pub profile: Option<String>,
    pub claudy_bin_path: PathBuf,
    pub log_dir: PathBuf,
    pub pid_file: PathBuf,
}

pub trait ServiceManager {
    fn install(&self) -> anyhow::Result<()>;
    fn start(&self) -> anyhow::Result<()>;
    fn stop(&self) -> anyhow::Result<()>;
    fn is_running(&self) -> anyhow::Result<bool>;
    fn enable(&self) -> anyhow::Result<()>;
    fn disable(&self) -> anyhow::Result<()>;
}

pub fn platform_service(config: ServiceConfig) -> anyhow::Result<Box<dyn ServiceManager>> {
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(
            super::service_launchd::LaunchdServiceManager::new(config),
        ))
    }
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(
            super::service_systemd::SystemdServiceManager::new(config),
        ))
    }
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(super::service_win::WindowsServiceManager::new(
            config,
        )))
    }
}
