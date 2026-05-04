use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::providers::index::ProviderIndex;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerModelOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_context_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelPreset {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub model: String,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub model_tiers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserEndpoint {
    pub name: String,
    #[serde(rename = "display_name")]
    pub display_name: String,
    pub base_url: String,
    #[serde(rename = "api_key_env")]
    pub api_key_env: String,
    #[serde(
        rename = "default_model",
        skip_serializing_if = "String::is_empty",
        default
    )]
    pub default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindowPolicy {
    #[serde(default = "default_auto_compact")]
    pub auto_compact: bool,
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

fn default_auto_compact() -> bool {
    true
}
fn default_threshold() -> f64 {
    0.8
}

impl Default for ContextWindowPolicy {
    fn default() -> Self {
        Self {
            auto_compact: true,
            threshold: 0.8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BridgeSettings {
    #[serde(default)]
    pub enabled_platforms: Vec<String>,
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    /// Provider profile used when a platform has no explicit mapping.
    #[serde(default)]
    pub default_profile: String,
    /// Per-platform provider profile overrides. Key = platform name (telegram/slack/discord).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub platform_profiles: HashMap<String, String>,
    /// Default mode applied to all platforms unless overridden.
    #[serde(default)]
    pub default_mode: String,
    /// Per-platform mode overrides. Key = platform, Value = mode name (from ~/.claudy/modes/).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub platform_modes: HashMap<String, String>,
    #[serde(default)]
    pub allowed_users: Vec<String>,
    #[serde(default)]
    pub max_concurrent_sessions: usize,
    /// Maximum seconds to wait for a Claude stream response (default: 1800 = 30 min).
    #[serde(default = "default_stream_timeout_secs")]
    pub stream_timeout_secs: u64,
}

fn default_stream_timeout_secs() -> u64 {
    1800
}

impl BridgeSettings {
    /// Resolve the provider profile for a given platform.
    /// Falls back to `default_profile` if no per-platform mapping exists.
    pub fn profile_for(&self, platform: &str) -> String {
        self.platform_profiles
            .get(platform)
            .cloned()
            .unwrap_or_else(|| self.default_profile.clone())
    }

    /// Resolve the mode for a given platform.
    /// Falls back to `default_mode` if no per-platform mapping exists.
    pub fn mode_for(&self, platform: &str) -> Option<String> {
        let mode = self
            .platform_modes
            .get(platform)
            .cloned()
            .unwrap_or_else(|| self.default_mode.clone());
        if mode.is_empty() { None } else { Some(mode) }
    }
}

pub fn default_listen_addr() -> String {
    "127.0.0.1:3456".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegistry {
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(rename = "provider_overrides", default)]
    pub provider_overrides: HashMap<String, ModelPreset>,
    #[serde(rename = "openrouter_aliases", default)]
    pub openrouter_aliases: HashMap<String, String>,
    #[serde(rename = "custom_providers", default)]
    pub custom_providers: HashMap<String, UserEndpoint>,
    #[serde(default)]
    pub compaction: ContextWindowPolicy,
    #[serde(rename = "model_settings", default)]
    pub model_settings: HashMap<String, PerModelOverrides>,
    #[serde(default)]
    pub channel: BridgeSettings,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub agents: HashMap<String, crate::domain::agent::AgentConfig>,
}

fn default_version() -> i32 {
    1
}

impl Default for AppRegistry {
    fn default() -> Self {
        AppRegistry {
            version: 1,
            provider_overrides: HashMap::new(),
            openrouter_aliases: HashMap::new(),
            custom_providers: HashMap::new(),
            compaction: ContextWindowPolicy::default(),
            model_settings: HashMap::new(),
            channel: BridgeSettings::default(),
            agents: HashMap::new(),
        }
    }
}

impl AppRegistry {
    /// Read from disk, returning default if the file doesn't exist.
    /// Auto-migrates legacy config.yaml to config.yaml on first run.
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        // Migrate config.yaml → config.yaml if yaml doesn't exist yet
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") && !path.exists() {
            let json_path = path.with_extension("json");
            if json_path.exists() {
                let raw = std::fs::read(&json_path)?;
                let cfg: Self = serde_json::from_slice(&raw)?;
                cfg.write_to(path)?;
                let _ = std::fs::remove_file(&json_path);
                return Ok(cfg);
            }
        }

        let raw = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };
        serde_yaml::from_slice(&raw).map_err(Into::into)
    }

    /// Serialize and write atomically.
    pub fn write_to(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let yaml = serde_yaml::to_string(self)?;
        super::atomic::write_atomic(&p.to_string_lossy(), yaml.as_bytes(), 0o644)?;
        Ok(())
    }

    /// Migrate pre-v1 secrets into provider overrides / custom providers.
    pub fn ingest_legacy_secrets(
        &mut self,
        secrets: &crate::config::vault::SecretVault,
        catalog: &ProviderIndex,
    ) {
        let builtin_keys = catalog.builtin_secret_keys();

        secrets
            .iter()
            .filter(|(k, _)| k.ends_with("_API_KEY") || k.starts_with("OPENROUTER_MODEL_"))
            .for_each(|(key, value)| {
                if let Some(suffix) = key.strip_prefix("OPENROUTER_MODEL_") {
                    let alias = normalize_openrouter_name(&suffix.to_lowercase().replace('_', "-"));
                    let val = value.trim();
                    if !alias.is_empty() && !is_launcher_placeholder(val) && !val.is_empty() {
                        self.openrouter_aliases
                            .entry(alias)
                            .or_insert_with(|| val.to_owned());
                    }
                    return;
                }

                if !key.ends_with("_API_KEY") || builtin_keys.contains(key) {
                    return;
                }

                let base_key = format!("CLAUDY_{}_BASE_URL", key);
                let base_url = match secrets.get(&base_key) {
                    Some(v) if !v.is_empty() => v.clone(),
                    _ => return,
                };
                let name = key[..key.len() - "_API_KEY".len()]
                    .to_lowercase()
                    .replace('_', "-");
                if self.custom_providers.contains_key(&name) {
                    return;
                }
                self.custom_providers.insert(
                    name.clone(),
                    UserEndpoint {
                        name: name.clone(),
                        display_name: name,
                        base_url,
                        api_key_env: key.clone(),
                        default_model: String::new(),
                    },
                );
            });
    }

    pub fn is_provider_configured(
        &self,
        id: &str,
        catalog: &ProviderIndex,
        secrets: &crate::config::vault::SecretVault,
    ) -> bool {
        let provider = match catalog.get(id) {
            Some(p) => p,
            None => return false,
        };

        if provider.auth_mode == "secret" {
            return secrets
                .get(&provider.key_var)
                .cloned()
                .or_else(|| std::env::var(&provider.key_var).ok())
                .is_some_and(|v| !v.trim().is_empty());
        }

        self.provider_overrides
            .get(id)
            .is_some_and(|ov| !ov.model.is_empty() || !ov.model_tiers.is_empty())
    }

    pub fn is_openrouter_configured(&self, secrets: &crate::config::vault::SecretVault) -> bool {
        let has_key = secrets
            .get("OPENROUTER_API_KEY")
            .cloned()
            .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
            .is_some_and(|v| !v.trim().is_empty());
        has_key || !self.openrouter_aliases.is_empty()
    }

    /// Remove no-op overrides and canonicalize aliases.
    pub fn compact(&mut self, catalog: &ProviderIndex) {
        let old = std::mem::take(&mut self.provider_overrides);
        self.provider_overrides = old
            .into_iter()
            .filter_map(|(id, ov)| {
                if ov.model.trim().is_empty() && ov.model_tiers.is_empty() {
                    return None;
                }
                let provider = catalog.get(&id)?;
                let model = resolve_model_choice(provider, &ov.model);
                let tiers = prune_empty_tiers(&ov.model_tiers);
                if (model.is_empty() || model == provider.default_model) && tiers.is_empty() {
                    None
                } else {
                    Some((
                        id,
                        ModelPreset {
                            model,
                            model_tiers: tiers,
                        },
                    ))
                }
            })
            .collect();

        let old_aliases = std::mem::take(&mut self.openrouter_aliases);
        self.openrouter_aliases = old_aliases
            .into_iter()
            .filter_map(|(name, model)| {
                let name = normalize_openrouter_name(&name);
                let model = model.trim();
                if name.is_empty() || model.is_empty() || is_launcher_placeholder(model) {
                    None
                } else {
                    Some((name, model.to_owned()))
                }
            })
            .collect();
    }

    pub fn openrouter_names(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.openrouter_aliases.keys().cloned().collect();
        keys.sort();
        keys
    }

    pub fn custom_provider_names(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.custom_providers.keys().cloned().collect();
        keys.sort();
        keys
    }
}

/// Resolve a model string that might be a 1-based index into `provider.model_choices`.
fn resolve_model_choice(
    provider: &crate::providers::index::ServiceDescriptor,
    raw: &str,
) -> String {
    let val = raw.trim();
    if val.is_empty() {
        return String::new();
    }
    val.parse::<usize>()
        .ok()
        .and_then(|idx| provider.model_choices.get(idx.wrapping_sub(1)))
        .map(|c| c.id.clone())
        .unwrap_or_else(|| val.to_owned())
}

fn prune_empty_tiers(tiers: &HashMap<String, String>) -> HashMap<String, String> {
    const RECOGNIZED: &[&str] = &["opus", "sonnet", "haiku", "small"];
    tiers
        .iter()
        .filter(|(k, v)| RECOGNIZED.contains(&k.as_str()) && !v.trim().is_empty())
        .map(|(k, v)| (k.clone(), v.trim().to_owned()))
        .collect()
}

pub fn normalize_openrouter_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .trim_start_matches("claudy-or-")
        .trim_matches('-')
        .to_owned()
}

pub fn is_launcher_placeholder(value: &str) -> bool {
    let lower = value.trim().to_lowercase();
    lower.starts_with("claudy-") || lower.starts_with("claudy ")
}

// Legacy free-function API preserved for backward compat.
pub fn open_registry(path: &str) -> anyhow::Result<AppRegistry> {
    AppRegistry::open(path)
}

pub fn write_registry(path: &str, cfg: &AppRegistry) -> anyhow::Result<()> {
    cfg.write_to(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::index as providers;

    fn load_catalog() -> ProviderIndex {
        providers::load_index().expect("catalog should load")
    }

    #[test]
    fn test_compact_repairs_legacy_overrides_and_aliases() {
        let catalog = load_catalog();
        let mut cfg = AppRegistry {
            version: 1,
            provider_overrides: HashMap::from([(
                "zai".to_string(),
                ModelPreset {
                    model: "1".to_string(),
                    model_tiers: HashMap::new(),
                },
            )]),
            openrouter_aliases: HashMap::from([
                (
                    "claudy-or-kimi-k25".to_string(),
                    "claudy-or-kimi-k25".to_string(),
                ),
                ("kimi-k25".to_string(), "moonshotai/kimi-k2.5".to_string()),
            ]),
            ..Default::default()
        };

        cfg.compact(&catalog);

        assert!(!cfg.provider_overrides.contains_key("zai"));
        assert_eq!(cfg.openrouter_aliases.len(), 1);
        assert_eq!(
            cfg.openrouter_aliases.get("kimi-k25").map(|s| s.as_str()),
            Some("moonshotai/kimi-k2.5")
        );
    }
}
