use super::service::{ServiceConfig, ServiceManager};

pub struct WindowsServiceManager {
    _config: ServiceConfig,
}

impl WindowsServiceManager {
    pub fn new(config: ServiceConfig) -> Self {
        Self { _config: config }
    }
}

impl ServiceManager for WindowsServiceManager {
    fn install(&self) -> anyhow::Result<()> {
        anyhow::bail!("Windows service management is not yet implemented")
    }

    fn start(&self) -> anyhow::Result<()> {
        anyhow::bail!("Windows service management is not yet implemented")
    }

    fn stop(&self) -> anyhow::Result<()> {
        anyhow::bail!("Windows service management is not yet implemented")
    }

    fn is_running(&self) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn enable(&self) -> anyhow::Result<()> {
        anyhow::bail!("Windows service management is not yet implemented")
    }

    fn disable(&self) -> anyhow::Result<()> {
        anyhow::bail!("Windows service management is not yet implemented")
    }
}
