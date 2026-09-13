use std::collections::HashMap;

use crate::config::registry::AppRegistry;
use crate::domain::launch_blueprint::LaunchTarget;
use crate::providers::index::ProviderIndex;

/// Resolves a profile name to a launch [`LaunchTarget`].
///
/// Lookup priority: catalog provider → `or-` alias → custom provider.
pub struct ProfileRouter<'a> {
    catalog: &'a ProviderIndex,
    cfg: &'a AppRegistry,
}

impl<'a> ProfileRouter<'a> {
    pub fn new(catalog: &'a ProviderIndex, cfg: &'a AppRegistry) -> Self {
        Self { catalog, cfg }
    }

    pub fn resolve(&self, name: &str) -> anyhow::Result<LaunchTarget> {
        self.lookup_catalog(name)
            .or_else(|| self.lookup_openrouter(name))
            .or_else(|| self.lookup_custom(name))
            .ok_or_else(|| anyhow::anyhow!("unknown profile {:?}", name))
    }

    pub fn all_targets(&self) -> Vec<LaunchTarget> {
        let catalog_ids = self.catalog.ids();
        let or_names = self.cfg.openrouter_names();
        let custom_names = self.cfg.custom_provider_names();

        let all_names: Vec<String> = catalog_ids
            .into_iter()
            .chain(or_names.into_iter().map(|n| format!("or-{}", n)))
            .chain(custom_names)
            .collect();

        all_names
            .iter()
            .filter_map(|name| self.resolve(name).ok())
            .collect()
    }

    /// Merge a provider's base model/tiers with any user `provider_overrides`
    /// preset stored under `name`. Override values win when non-empty.
    fn apply_preset(
        &self,
        name: &str,
        base_model: &str,
        base_tiers: &HashMap<String, String>,
    ) -> (String, HashMap<String, String>) {
        match self.cfg.provider_overrides.get(name) {
            Some(ov) => {
                let m = ov.model.trim();
                let model = if m.is_empty() {
                    base_model.to_owned()
                } else {
                    m.to_owned()
                };
                let mut tiers = base_tiers.clone();
                tiers.extend(ov.model_tiers.iter().map(|(k, v)| (k.clone(), v.clone())));
                (model, tiers)
            }
            None => (base_model.to_owned(), base_tiers.clone()),
        }
    }

    fn lookup_catalog(&self, name: &str) -> Option<LaunchTarget> {
        let p = self.catalog.get(name)?;
        let (model, tiers) = self.apply_preset(name, &p.default_model, &p.model_tiers);

        Some(LaunchTarget {
            profile: name.to_owned(),
            display_name: p.display_name.clone(),
            description: p.description.clone(),
            category: p.category.clone(),
            family: p.family.clone(),
            base_url: p.base_url.clone(),
            model,
            model_tiers: tiers,
            auth_mode: p.auth_mode.clone(),
            secret_key: p.key_var.clone(),
            literal_auth_token: p.literal_auth_token.clone(),
            test_url: p.test_url.clone(),
        })
    }

    fn lookup_openrouter(&self, name: &str) -> Option<LaunchTarget> {
        let alias = name.strip_prefix("or-")?;
        let alias_model = self.cfg.openrouter_aliases.get(alias)?;
        let (model, tiers) = self.apply_preset(name, alias_model, &HashMap::new());
        Some(LaunchTarget {
            profile: name.to_owned(),
            display_name: format!("OpenRouter: {}", alias),
            description: "OpenRouter alias".to_owned(),
            category: "openrouter".to_owned(),
            family: "openrouter".to_owned(),
            base_url: "https://openrouter.ai/api".to_owned(),
            model,
            model_tiers: tiers,
            auth_mode: "secret".to_owned(),
            secret_key: "OPENROUTER_API_KEY".to_owned(),
            literal_auth_token: String::new(),
            test_url: "https://openrouter.ai/api".to_owned(),
        })
    }

    fn lookup_custom(&self, name: &str) -> Option<LaunchTarget> {
        let cp = self.cfg.custom_providers.get(name)?;
        let (model, tiers) = self.apply_preset(name, &cp.default_model, &HashMap::new());
        Some(LaunchTarget {
            profile: name.to_owned(),
            display_name: cp.display_name.clone(),
            description: format!("Custom provider: {}", name),
            category: "custom".to_owned(),
            family: "anthropic_compatible_non_claude".to_owned(),
            base_url: cp.base_url.clone(),
            model,
            model_tiers: tiers,
            auth_mode: "secret".to_owned(),
            secret_key: cp.api_key_env.clone(),
            literal_auth_token: String::new(),
            test_url: cp.base_url.clone(),
        })
    }
}

/// Detect whether the binary was invoked as a launcher symlink.
pub fn detect_symlink_invocation(argv0: &str) -> (String, bool) {
    let fname = std::path::Path::new(argv0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let is_launcher = fname.strip_prefix("claudy-").is_some_and(|suffix| {
        !suffix.is_empty() && suffix != "dev" && !suffix.starts_with("2") && suffix != "cli" // cargo run
    });

    if is_launcher {
        (fname.strip_prefix("claudy-").unwrap().to_owned(), true)
    } else {
        (String::new(), false)
    }
}

// Legacy free-function API.
pub fn route_profile(
    name: &str,
    catalog: &ProviderIndex,
    cfg: &AppRegistry,
) -> anyhow::Result<LaunchTarget> {
    ProfileRouter::new(catalog, cfg).resolve(name)
}

pub fn all_launch_targets(catalog: &ProviderIndex, cfg: &AppRegistry) -> Vec<LaunchTarget> {
    ProfileRouter::new(catalog, cfg).all_targets()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::registry::ModelPreset;
    use crate::providers::index as providers;

    fn load_catalog() -> ProviderIndex {
        providers::load_index().expect("catalog should load")
    }

    #[test]
    fn test_invocation_claudy_bare() {
        let (p, is) = detect_symlink_invocation("claudy");
        assert!(!is);
        assert_eq!(p, "");
    }

    #[test]
    fn test_invocation_launcher() {
        let (p, is) = detect_symlink_invocation("claudy-native");
        assert!(is);
        assert_eq!(p, "native");
    }

    #[test]
    fn test_resolve_with_tier_overrides() {
        let catalog = load_catalog();
        let cfg = AppRegistry {
            provider_overrides: HashMap::from([(
                "zai".to_string(),
                ModelPreset {
                    model: String::new(),
                    model_tiers: HashMap::from([
                        ("haiku".to_string(), "glm-4-flash".to_string()),
                        ("opus".to_string(), "glm-6".to_string()),
                    ]),
                },
            )]),
            ..AppRegistry::default()
        };

        let target = route_profile("zai", &catalog, &cfg).expect("resolve");
        assert_eq!(target.model_tiers.get("haiku").unwrap(), "glm-4-flash");
        assert_eq!(target.model_tiers.get("opus").unwrap(), "glm-6");
        assert_eq!(target.model_tiers.get("sonnet").unwrap(), "glm-5");
    }

    #[test]
    fn test_resolve_with_model_and_explicit_tiers() {
        let catalog = load_catalog();
        let cfg = AppRegistry {
            provider_overrides: HashMap::from([(
                "zai".to_string(),
                ModelPreset {
                    model: "glm-4-plus".to_string(),
                    model_tiers: HashMap::from([("haiku".to_string(), "glm-4-flash".to_string())]),
                },
            )]),
            ..AppRegistry::default()
        };

        let target = route_profile("zai", &catalog, &cfg).expect("resolve");
        assert_eq!(target.model, "glm-4-plus");
        assert_eq!(target.model_tiers.get("haiku").unwrap(), "glm-4-flash");
    }

    #[test]
    fn test_resolve_with_partial_tier_overrides() {
        let catalog = load_catalog();
        let cfg = AppRegistry {
            provider_overrides: HashMap::from([(
                "zai".to_string(),
                ModelPreset {
                    model: "glm-5-new".to_string(),
                    model_tiers: HashMap::from([("haiku".to_string(), "glm-4-flash".to_string())]),
                },
            )]),
            ..AppRegistry::default()
        };

        let target = route_profile("zai", &catalog, &cfg).expect("resolve");
        assert_eq!(target.model, "glm-5-new");
        assert_eq!(target.model_tiers.get("haiku").unwrap(), "glm-4-flash");
        assert_eq!(target.model_tiers.get("sonnet").unwrap(), "glm-5");
    }

    #[test]
    fn test_resolve_legacy_model_only_overrides_all_tiers() {
        let catalog = load_catalog();
        let cfg = AppRegistry {
            provider_overrides: HashMap::from([(
                "zai".to_string(),
                ModelPreset {
                    model: "glm-custom-1".to_string(),
                    model_tiers: HashMap::new(),
                },
            )]),
            ..AppRegistry::default()
        };

        let target = route_profile("zai", &catalog, &cfg).expect("resolve");
        assert_eq!(target.model, "glm-custom-1");
        assert_eq!(target.model_tiers.get("haiku").unwrap(), "glm-5");
    }

    #[test]
    fn test_resolve_open_router_alias() {
        let catalog = load_catalog();
        let cfg = AppRegistry {
            openrouter_aliases: HashMap::from([(
                "kimi".to_string(),
                "moonshotai/kimi-k2.5".to_string(),
            )]),
            ..AppRegistry::default()
        };

        let target = route_profile("or-kimi", &catalog, &cfg).expect("resolve");
        assert_eq!(target.family, "openrouter");
        assert_eq!(target.model, "moonshotai/kimi-k2.5");
        assert_eq!(target.secret_key, "OPENROUTER_API_KEY");
    }

    #[test]
    fn test_resolve_open_router_alias_with_tier_overrides() {
        let catalog = load_catalog();
        let cfg = AppRegistry {
            openrouter_aliases: HashMap::from([(
                "kimi".to_string(),
                "moonshotai/kimi-k2.5".to_string(),
            )]),
            provider_overrides: HashMap::from([(
                "or-kimi".to_string(),
                ModelPreset {
                    model: String::new(),
                    model_tiers: HashMap::from([
                        ("haiku".to_string(), "moonshotai/kimi-vl-a3b".to_string()),
                        ("opus".to_string(), "moonshotai/kimi-k2.5".to_string()),
                    ]),
                },
            )]),
            ..AppRegistry::default()
        };

        let target = route_profile("or-kimi", &catalog, &cfg).expect("resolve");
        assert_eq!(target.model, "moonshotai/kimi-k2.5");
        assert_eq!(
            target.model_tiers.get("haiku").unwrap(),
            "moonshotai/kimi-vl-a3b"
        );
        assert_eq!(
            target.model_tiers.get("opus").unwrap(),
            "moonshotai/kimi-k2.5"
        );
    }

    #[test]
    fn test_resolve_custom_provider_with_tier_overrides() {
        let catalog = load_catalog();
        let cfg = AppRegistry {
            custom_providers: HashMap::from([(
                "my-llm".to_string(),
                crate::config::registry::UserEndpoint {
                    name: "my-llm".to_string(),
                    display_name: "My LLM".to_string(),
                    base_url: "https://my-llm.com/api".to_string(),
                    api_key_env: "MY_LLM_API_KEY".to_string(),
                    default_model: "test-model".to_string(),
                },
            )]),
            provider_overrides: HashMap::from([(
                "my-llm".to_string(),
                ModelPreset {
                    model: String::new(),
                    model_tiers: HashMap::from([
                        ("haiku".to_string(), "test-mini".to_string()),
                        ("fable".to_string(), "test-max".to_string()),
                    ]),
                },
            )]),
            ..AppRegistry::default()
        };

        let target = route_profile("my-llm", &catalog, &cfg).expect("resolve");
        assert_eq!(target.model, "test-model");
        assert_eq!(target.model_tiers.get("haiku").unwrap(), "test-mini");
        assert_eq!(target.model_tiers.get("fable").unwrap(), "test-max");
    }

    #[test]
    fn test_resolve_custom_provider() {
        let catalog = load_catalog();
        let cfg = AppRegistry {
            custom_providers: HashMap::from([(
                "my-llm".to_string(),
                crate::config::registry::UserEndpoint {
                    name: "my-llm".to_string(),
                    display_name: "My LLM".to_string(),
                    base_url: "https://my-llm.com/api".to_string(),
                    api_key_env: "MY_LLM_API_KEY".to_string(),
                    default_model: "test-model".to_string(),
                },
            )]),
            ..AppRegistry::default()
        };

        let target = route_profile("my-llm", &catalog, &cfg).expect("resolve");
        assert_eq!(target.display_name, "My LLM");
        assert_eq!(target.base_url, "https://my-llm.com/api");
        assert_eq!(target.secret_key, "MY_LLM_API_KEY");
        assert_eq!(target.model, "test-model");
    }
}
