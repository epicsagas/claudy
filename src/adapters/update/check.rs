use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use ureq::config::Config;

use super::states::UpdateState;
use crate::config::layout::AppPaths;

const GITHUB_API_URL: &str = "https://api.github.com/repos/epicsagas/claudy/releases/latest";
const CHECK_INTERVAL: Duration = Duration::from_secs(86_400);
const NOTIFY_INTERVAL: Duration = Duration::from_secs(86_400);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheFile {
    pub last_checked_unix: u64,
    pub latest_version: Option<String>,
    pub last_notified_unix: Option<u64>,
    pub last_notified_for: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

pub struct VersionMetadata {
    pub version: String,
}

pub trait VersionProvider {
    fn fetch_latest(&self) -> anyhow::Result<VersionMetadata>;
}

pub struct GitHubProvider {
    pub url: String,
}

impl GitHubProvider {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl VersionProvider for GitHubProvider {
    fn fetch_latest(&self) -> anyhow::Result<VersionMetadata> {
        let config = Config::builder()
            .timeout_global(Some(Duration::from_secs(3)))
            .build();
        let agent = ureq::Agent::new_with_config(config);

        let mut resp = agent
            .get(&self.url)
            .header("User-Agent", "claudy-cli")
            .call()?;

        if self.url.ends_with(".json") {
            #[derive(Deserialize)]
            struct VersionJson {
                version: String,
            }
            let meta: VersionJson = resp.body_mut().read_json()?;
            Ok(VersionMetadata {
                version: strip_v_prefix(&meta.version),
            })
        } else {
            let release: GitHubRelease = resp.body_mut().read_json()?;
            Ok(VersionMetadata {
                version: strip_v_prefix(&release.tag_name),
            })
        }
    }
}

pub fn maybe_message(paths: &AppPaths, current_version: &str) -> anyhow::Result<Option<String>> {
    let provider = GitHubProvider::new(GITHUB_API_URL.to_string());
    let now = epoch_secs();
    UpdateCheckScheduler::new(paths, current_version, now, &provider).run()
}

struct UpdateCheckScheduler<'a, P: VersionProvider> {
    paths: &'a AppPaths,
    current_version: &'a str,
    now: u64,
    provider: &'a P,
    cache: CacheFile,
    dirty: bool,
    state: UpdateState,
}

impl<'a, P: VersionProvider> UpdateCheckScheduler<'a, P> {
    fn new(paths: &'a AppPaths, current_version: &'a str, now: u64, provider: &'a P) -> Self {
        Self {
            paths,
            current_version,
            now,
            provider,
            cache: CacheFile::default(),
            dirty: false,
            state: UpdateState::Idle,
        }
    }

    fn run(mut self) -> anyhow::Result<Option<String>> {
        if self.current_version == "dev" || self.current_version.is_empty() {
            self.state = UpdateState::Complete;
            return Ok(None);
        }

        self.cache = CacheFile::open(&self.paths.update_cache_file)?;

        if self.cache_last_checked_expired() {
            self.state = UpdateState::RefreshNeeded;
            self.refresh();
        }

        self.state = UpdateState::Compared;
        let latest = self.cache.latest_version.clone();
        let message = latest
            .as_ref()
            .filter(|v| semver_gt(self.current_version, v))
            .and_then(|v| {
                let due = self.notification_due(v);
                due.then(|| {
                    let msg = self.build_notify_message(v);
                    self.cache.last_notified_unix = Some(self.now);
                    self.cache.last_notified_for = Some(v.clone());
                    self.dirty = true;
                    self.state = UpdateState::Notified;
                    msg
                })
            });

        if self.dirty {
            self.cache.persist(&self.paths.update_cache_file)?;
        }
        self.state = UpdateState::Complete;
        Ok(message)
    }

    fn cache_last_checked_expired(&self) -> bool {
        self.cache.last_checked_unix == 0
            || self.now > self.cache.last_checked_unix + CHECK_INTERVAL.as_secs()
    }

    fn notification_due(&self, latest: &str) -> bool {
        match (self.cache.last_notified_unix, &self.cache.last_notified_for) {
            (Some(last_ts), Some(last_ver)) if last_ver == latest => {
                self.now > last_ts + NOTIFY_INTERVAL.as_secs()
            }
            _ => true,
        }
    }

    fn refresh(&mut self) {
        self.cache.last_checked_unix = self.now;
        self.dirty = true;
        if let Ok(meta) = self.provider.fetch_latest() {
            self.cache.latest_version = Some(meta.version);
        }
        self.state = UpdateState::Refreshed;
    }

    fn build_notify_message(&self, latest: &str) -> String {
        format!(
            "\x1b[32m✨ A new version of claudy is available: {} → {}\x1b[0m\n\x1b[32m   Run 'claudy update' to install.\x1b[0m",
            self.current_version, latest
        )
    }
}

impl CacheFile {
    fn open(path: &str) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&raw).or_else(|_| Ok(Self::default()))
    }

    fn persist(&self, path: &str) -> anyhow::Result<()> {
        let p = Path::new(path);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let tmp = p
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!(".update-cache-{}.tmp", epoch_secs()));
        fs::write(&tmp, json)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }
}

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_secs()
}

pub fn display_version(v: &str) -> String {
    strip_v_prefix(v)
}

fn strip_v_prefix(v: &str) -> String {
    v.trim_start_matches('v').to_owned()
}

#[cfg(test)]
fn normalize_version(v: &str) -> String {
    strip_v_prefix(v)
}

/// Semver-style comparison: true when `latest` is strictly newer than `current`.
pub fn is_newer(current: &str, latest: &str) -> bool {
    semver_gt(current, latest)
}

fn semver_gt(current: &str, latest: &str) -> bool {
    parse_version_segments(latest) > parse_version_segments(current)
}

fn parse_version_segments(v: &str) -> Vec<u32> {
    v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect()
}

#[cfg(test)]
fn needs_check(cache: &CacheFile, now: u64) -> bool {
    cache.last_checked_unix == 0 || now > cache.last_checked_unix + CHECK_INTERVAL.as_secs()
}

#[cfg(test)]
fn should_alert(cache: &CacheFile, now: u64) -> bool {
    match (
        &cache.last_notified_unix,
        &cache.last_notified_for,
        &cache.latest_version,
    ) {
        (Some(last_unix), Some(last_ver), Some(latest)) => {
            if last_ver != latest {
                return true;
            }
            now > last_unix + NOTIFY_INTERVAL.as_secs()
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::layout::AppPaths;

    struct StaticProvider {
        version: &'static str,
    }

    impl VersionProvider for StaticProvider {
        fn fetch_latest(&self) -> anyhow::Result<VersionMetadata> {
            Ok(VersionMetadata {
                version: self.version.to_string(),
            })
        }
    }

    fn make_paths() -> AppPaths {
        let root = std::env::temp_dir().join(format!(
            "claudy-update-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("mkdir");
        AppPaths {
            claudy_home: String::new(),
            config_dir: String::new(),
            data_dir: String::new(),
            cache_dir: root.to_string_lossy().to_string(),
            bin_dir: String::new(),
            config_file: String::new(),
            secrets_file: String::new(),
            manifest_file: String::new(),
            session_patch_dir: String::new(),
            update_cache_file: root.join("update.json").to_string_lossy().to_string(),
            modes_dir: String::new(),
            channel_dir: String::new(),
            channel_pid_file: String::new(),
            channel_sessions_file: String::new(),
            channel_audit_file: String::new(),
            channel_logs_dir: String::new(),
            analytics_dir: "/tmp/test-analytics".to_string(),
            analytics_db: "/tmp/test-analytics/analytics.db".to_string(),
        }
    }

    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("v3.0.1"), "3.0.1");
        assert_eq!(normalize_version("3.0.1"), "3.0.1");
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("3.0.0", "3.0.1"));
        assert!(is_newer("3.0.9", "3.1.0"));
        assert!(!is_newer("3.0.1", "3.0.0"));
        assert!(!is_newer("3.0.1", "3.0.1"));
    }

    #[test]
    fn test_needs_check_with_no_cache() {
        let cache = CacheFile::default();
        assert!(needs_check(&cache, 100));
    }

    #[test]
    fn test_needs_check_within_ttl() {
        let cache = CacheFile {
            last_checked_unix: 1000,
            ..Default::default()
        };
        assert!(!needs_check(&cache, 1000 + 3600));
    }

    #[test]
    fn test_should_alert_with_new_version() {
        let cache = CacheFile {
            last_notified_for: Some("v3.0.0".to_string()),
            latest_version: Some("v3.0.1".to_string()),
            ..Default::default()
        };
        assert!(should_alert(&cache, 1000));
    }

    #[test]
    fn test_should_alert_same_version_within_ttl() {
        let cache = CacheFile {
            last_notified_for: Some("v3.0.1".to_string()),
            latest_version: Some("v3.0.1".to_string()),
            last_notified_unix: Some(1000),
            ..Default::default()
        };
        assert!(!should_alert(&cache, 1000 + 3600));
    }

    #[test]
    fn test_should_alert_same_version_past_ttl() {
        let cache = CacheFile {
            last_notified_for: Some("v3.0.1".to_string()),
            latest_version: Some("v3.0.1".to_string()),
            last_notified_unix: Some(1000),
            ..Default::default()
        };
        assert!(
            should_alert(&cache, 1000 + 90000),
            "same version past TTL should re-notify"
        );
    }

    #[test]
    fn test_maybe_message_skips_dev_version() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = crate::config::layout::AppPaths {
            claudy_home: String::new(),
            config_dir: String::new(),
            data_dir: String::new(),
            cache_dir: dir.path().to_string_lossy().to_string(),
            bin_dir: String::new(),
            config_file: String::new(),
            secrets_file: String::new(),
            manifest_file: String::new(),
            session_patch_dir: String::new(),
            update_cache_file: dir.path().join("update.json").to_string_lossy().to_string(),
            modes_dir: String::new(),
            channel_dir: String::new(),
            channel_pid_file: String::new(),
            channel_sessions_file: String::new(),
            channel_audit_file: String::new(),
            channel_logs_dir: String::new(),
            analytics_dir: "/tmp/test-analytics".to_string(),
            analytics_db: "/tmp/test-analytics/analytics.db".to_string(),
        };
        let result = maybe_message(&paths, "dev").expect("maybe_message");
        assert!(result.is_none(), "dev builds should not check for updates");
    }

    #[test]
    fn test_maybe_message_skips_empty_version() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = crate::config::layout::AppPaths {
            claudy_home: String::new(),
            config_dir: String::new(),
            data_dir: String::new(),
            cache_dir: dir.path().to_string_lossy().to_string(),
            bin_dir: String::new(),
            config_file: String::new(),
            secrets_file: String::new(),
            manifest_file: String::new(),
            session_patch_dir: String::new(),
            update_cache_file: dir.path().join("update.json").to_string_lossy().to_string(),
            modes_dir: String::new(),
            channel_dir: String::new(),
            channel_pid_file: String::new(),
            channel_sessions_file: String::new(),
            channel_audit_file: String::new(),
            channel_logs_dir: String::new(),
            analytics_dir: "/tmp/test-analytics".to_string(),
            analytics_db: "/tmp/test-analytics/analytics.db".to_string(),
        };
        let result = maybe_message(&paths, "").expect("maybe_message");
        assert!(
            result.is_none(),
            "empty version should not check for updates"
        );
    }

    #[test]
    fn test_cache_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("update.json").to_string_lossy().to_string();

        let cache = CacheFile {
            last_checked_unix: 12345,
            latest_version: Some("1.2.3".to_string()),
            ..Default::default()
        };

        cache.persist(&path).expect("persist");
        let loaded = CacheFile::open(&path).expect("open");

        assert_eq!(loaded.last_checked_unix, 12345);
        assert_eq!(loaded.latest_version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_display_version() {
        let v = normalize_version("v1.2.3");
        assert_eq!(v, "1.2.3");
    }

    #[test]
    fn test_cache_open_returns_default_on_corrupt_json() {
        let paths = make_paths();
        std::fs::write(&paths.update_cache_file, "{invalid json").expect("write corrupt");
        let loaded = CacheFile::open(&paths.update_cache_file).expect("open");
        assert_eq!(loaded.last_checked_unix, 0);
        assert!(loaded.latest_version.is_none());
    }

    #[test]
    fn test_cache_persist_replaces_old_content() {
        let paths = make_paths();
        std::fs::write(&paths.update_cache_file, "{\"last_checked_unix\":1}").expect("seed");
        let cache = CacheFile {
            last_checked_unix: 42,
            latest_version: Some("1.2.3".to_string()),
            ..Default::default()
        };
        cache.persist(&paths.update_cache_file).expect("persist");
        let loaded = CacheFile::open(&paths.update_cache_file).expect("open");
        assert_eq!(loaded.last_checked_unix, 42);
        assert_eq!(loaded.latest_version.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn test_scheduler_transitions_to_complete_without_update() {
        let paths = make_paths();
        let provider = StaticProvider { version: "1.0.0" };
        let scheduler = UpdateCheckScheduler::new(&paths, "1.0.0", 1000, &provider);
        let msg = scheduler.run().expect("run");
        assert!(msg.is_none());
    }

    #[test]
    fn test_scheduler_emits_message_when_newer_version_exists() {
        let paths = make_paths();
        let provider = StaticProvider { version: "1.0.1" };
        let scheduler = UpdateCheckScheduler::new(&paths, "1.0.0", 1000, &provider);
        let msg = scheduler.run().expect("run");
        assert!(msg.is_some());
    }
}
