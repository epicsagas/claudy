use std::env;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub claudy_home: String,
    pub config_dir: String,
    pub data_dir: String,
    pub cache_dir: String,
    pub bin_dir: String,
    pub config_file: String,
    pub secrets_file: String,
    pub manifest_file: String,
    pub session_patch_dir: String,
    pub update_cache_file: String,
    pub modes_dir: String,
    pub channel_dir: String,
    pub channel_pid_file: String,
    pub channel_sessions_file: String,
    pub channel_audit_file: String,
    pub channel_logs_dir: String,
    pub analytics_dir: String,
    pub analytics_db: String,
}

pub fn discover() -> anyhow::Result<AppPaths> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    // Centralize everything under ~/.claudy
    let claudy_home = getenv("CLAUDY_HOME", &home.join(".claudy").to_string_lossy());
    let claudy_home_path = Path::new(&claudy_home);

    let config_dir = claudy_home.clone();
    let data_dir = claudy_home.clone();
    let cache_dir = claudy_home_path.join("cache").to_string_lossy().to_string();

    let bin_dir = default_bin_dir(&home.to_string_lossy());

    let channel_dir = claudy_home_path
        .join("channel")
        .to_string_lossy()
        .to_string();

    let analytics_dir = claudy_home_path
        .join("analytics")
        .to_string_lossy()
        .to_string();

    let analytics_db = Path::new(&analytics_dir)
        .join("analytics.db")
        .to_string_lossy()
        .to_string();

    Ok(AppPaths {
        claudy_home: claudy_home.clone(),
        config_dir: config_dir.clone(),
        data_dir: data_dir.clone(),
        cache_dir: cache_dir.clone(),
        bin_dir: bin_dir.clone(),
        config_file: Path::new(&config_dir)
            .join("config.yaml")
            .to_string_lossy()
            .to_string(),
        secrets_file: Path::new(&data_dir)
            .join("secrets.env")
            .to_string_lossy()
            .to_string(),
        manifest_file: Path::new(&data_dir)
            .join("launchers.json")
            .to_string_lossy()
            .to_string(),
        session_patch_dir: Path::new(&data_dir)
            .join("session-patches")
            .to_string_lossy()
            .to_string(),
        update_cache_file: Path::new(&cache_dir)
            .join("update.json")
            .to_string_lossy()
            .to_string(),
        modes_dir: claudy_home_path.join("modes").to_string_lossy().to_string(),
        channel_pid_file: Path::new(&channel_dir)
            .join("pid")
            .to_string_lossy()
            .to_string(),
        channel_sessions_file: Path::new(&channel_dir)
            .join("sessions.json")
            .to_string_lossy()
            .to_string(),
        channel_audit_file: Path::new(&channel_dir)
            .join("audit.jsonl")
            .to_string_lossy()
            .to_string(),
        channel_logs_dir: Path::new(&channel_dir)
            .join("logs")
            .to_string_lossy()
            .to_string(),
        channel_dir,
        analytics_dir,
        analytics_db,
    })
}

impl AppPaths {
    pub fn ensure_base_dirs(&self) -> anyhow::Result<()> {
        for dir in &[
            &self.config_dir,
            &self.data_dir,
            &self.cache_dir,
            &self.session_patch_dir,
            &self.bin_dir,
            &self.modes_dir,
            &self.channel_dir,
            &self.analytics_dir,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

fn default_bin_dir(home: &str) -> String {
    // Prefer the directory where claudy is already installed (via which)
    if let Ok(found) = which::which("claudy") {
        let abs = std::fs::canonicalize(&found).unwrap_or(found);
        if let Some(parent) = abs.parent() {
            return parent.to_string_lossy().to_string();
        }
    }
    // Fallback to ~/.cargo/bin
    Path::new(home)
        .join(".cargo/bin")
        .to_string_lossy()
        .to_string()
}

fn getenv(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn setup_env(root: &std::path::Path) {
        let home = root.join("home");
        unsafe {
            env::set_var("HOME", &home);
            env::remove_var("XDG_CONFIG_HOME");
            env::remove_var("XDG_DATA_HOME");
            env::remove_var("XDG_CACHE_HOME");
            env::remove_var("CLAUDY_HOME");
            env::remove_var("CLAUDY_CONFIG_DIR");
            env::remove_var("CLAUDY_DATA_DIR");
            env::remove_var("CLAUDY_CACHE_DIR");
        }
    }

    #[test]
    #[serial]
    fn test_discover_consolidated_under_claudy_home() {
        let root = tempfile::tempdir().expect("tempdir");
        setup_env(root.path());

        let paths = discover().expect("discover");
        let home = root.path().join("home");
        let claudy_home = home.join(".claudy");

        assert_eq!(paths.config_dir, claudy_home.to_string_lossy().to_string());
        assert_eq!(paths.data_dir, claudy_home.to_string_lossy().to_string());
    }
}
