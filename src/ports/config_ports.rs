use std::collections::HashMap;

use crate::config::layout::AppPaths;
use crate::config::registry::{
    AppRegistry, ContextWindowPolicy, ModelPreset, PerModelOverrides, UserEndpoint,
};
use crate::config::vault::SecretVault;

// Re-export concrete types so domain can depend on ports instead of adapters.
pub use crate::config::layout::AppPaths as ConcreteAppPaths;
pub use crate::config::registry::AppRegistry as ConcreteAppRegistry;
pub use crate::config::vault::SecretVault as ConcreteSecretVault;

pub trait PathsPort {
    fn claudy_home(&self) -> &str;
    fn config_dir(&self) -> &str;
    fn data_dir(&self) -> &str;
    fn cache_dir(&self) -> &str;
    fn bin_dir(&self) -> &str;
    fn config_file(&self) -> &str;
    fn secrets_file(&self) -> &str;
    fn manifest_file(&self) -> &str;
    fn session_patch_dir(&self) -> &str;
    fn update_cache_file(&self) -> &str;
    fn modes_dir(&self) -> &str;
    fn ensure_base_dirs(&self) -> anyhow::Result<()>;
}

impl PathsPort for AppPaths {
    fn claudy_home(&self) -> &str {
        &self.claudy_home
    }
    fn config_dir(&self) -> &str {
        &self.config_dir
    }
    fn data_dir(&self) -> &str {
        &self.data_dir
    }
    fn cache_dir(&self) -> &str {
        &self.cache_dir
    }
    fn bin_dir(&self) -> &str {
        &self.bin_dir
    }
    fn config_file(&self) -> &str {
        &self.config_file
    }
    fn secrets_file(&self) -> &str {
        &self.secrets_file
    }
    fn manifest_file(&self) -> &str {
        &self.manifest_file
    }
    fn session_patch_dir(&self) -> &str {
        &self.session_patch_dir
    }
    fn update_cache_file(&self) -> &str {
        &self.update_cache_file
    }
    fn modes_dir(&self) -> &str {
        &self.modes_dir
    }
    fn ensure_base_dirs(&self) -> anyhow::Result<()> {
        AppPaths::ensure_base_dirs(self)
    }
}

pub trait SecretsPort {
    fn get(&self, key: &str) -> Option<&String>;
    fn contains_key(&self, key: &str) -> bool;
    fn insert(&mut self, key: String, value: String) -> Option<String>;
    fn clone_all(&self) -> SecretVault;
}

impl SecretsPort for SecretVault {
    fn get(&self, key: &str) -> Option<&String> {
        std::collections::HashMap::get(self, key)
    }
    fn contains_key(&self, key: &str) -> bool {
        std::collections::HashMap::contains_key(self, key)
    }
    fn insert(&mut self, key: String, value: String) -> Option<String> {
        std::collections::HashMap::insert(self, key, value)
    }
    fn clone_all(&self) -> SecretVault {
        self.clone()
    }
}

pub trait ConfigPort {
    fn version(&self) -> i32;
    fn provider_overrides(&self) -> &HashMap<String, ModelPreset>;
    fn provider_overrides_mut(&mut self) -> &mut HashMap<String, ModelPreset>;
    fn openrouter_aliases(&self) -> &HashMap<String, String>;
    fn openrouter_aliases_mut(&mut self) -> &mut HashMap<String, String>;
    fn custom_providers(&self) -> &HashMap<String, UserEndpoint>;
    fn custom_providers_mut(&mut self) -> &mut HashMap<String, UserEndpoint>;
    fn compaction(&self) -> &ContextWindowPolicy;
    fn model_settings(&self) -> &HashMap<String, PerModelOverrides>;
    fn model_settings_mut(&mut self) -> &mut HashMap<String, PerModelOverrides>;
}

impl ConfigPort for AppRegistry {
    fn version(&self) -> i32 {
        self.version
    }
    fn provider_overrides(&self) -> &HashMap<String, ModelPreset> {
        &self.provider_overrides
    }
    fn provider_overrides_mut(&mut self) -> &mut HashMap<String, ModelPreset> {
        &mut self.provider_overrides
    }
    fn openrouter_aliases(&self) -> &HashMap<String, String> {
        &self.openrouter_aliases
    }
    fn openrouter_aliases_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.openrouter_aliases
    }
    fn custom_providers(&self) -> &HashMap<String, UserEndpoint> {
        &self.custom_providers
    }
    fn custom_providers_mut(&mut self) -> &mut HashMap<String, UserEndpoint> {
        &mut self.custom_providers
    }
    fn compaction(&self) -> &ContextWindowPolicy {
        &self.compaction
    }
    fn model_settings(&self) -> &HashMap<String, PerModelOverrides> {
        &self.model_settings
    }
    fn model_settings_mut(&mut self) -> &mut HashMap<String, PerModelOverrides> {
        &mut self.model_settings
    }
}
