use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceDescriptor {
    pub id: String,
    #[serde(rename = "display_name")]
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub family: String,
    #[serde(rename = "auth_mode")]
    pub auth_mode: String,
    #[serde(rename = "key_var", skip_serializing_if = "String::is_empty", default)]
    pub key_var: String,
    #[serde(
        rename = "literal_auth_token",
        skip_serializing_if = "String::is_empty",
        default
    )]
    pub literal_auth_token: String,
    #[serde(rename = "base_url")]
    pub base_url: String,
    #[serde(rename = "default_model")]
    pub default_model: String,
    #[serde(rename = "model_tiers", default)]
    pub model_tiers: HashMap<String, String>,
    #[serde(rename = "model_choices", default)]
    pub model_choices: Vec<ModelDescriptor>,
    #[serde(rename = "test_url")]
    pub test_url: String,
    #[serde(default)]
    pub setup: Vec<String>,
    #[serde(default)]
    pub usage: Vec<String>,
}

/// Static catalog compiled into the binary from `catalog.json`.
static EMBEDDED: LazyLock<ProviderIndex> = LazyLock::new(|| {
    let raw = include_str!("catalog.json");
    let payload: IndexPayload = serde_json::from_str(raw).expect("catalog.json is valid");
    ProviderIndex::from_payload(payload)
});

#[derive(Debug, Deserialize)]
struct IndexPayload {
    providers: Vec<ServiceDescriptor>,
}

/// Immutable provider catalog with O(1) lookup by id.
#[derive(Debug, Clone)]
pub struct ProviderIndex {
    entries: Vec<ServiceDescriptor>,
    index: HashMap<String, usize>,
}

impl ProviderIndex {
    fn from_payload(payload: IndexPayload) -> Self {
        let index: HashMap<String, usize> = payload
            .providers
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id.clone(), i))
            .collect();
        Self {
            entries: payload.providers,
            index,
        }
    }

    /// Access the static catalog embedded at compile time.
    pub fn embedded() -> &'static ProviderIndex {
        &EMBEDDED
    }

    pub fn all(&self) -> &[ServiceDescriptor] {
        &self.entries
    }

    pub fn ids(&self) -> Vec<String> {
        self.entries.iter().map(|p| p.id.clone()).collect()
    }

    pub fn get(&self, id: &str) -> Option<&ServiceDescriptor> {
        self.index.get(id).map(|&i| &self.entries[i])
    }

    /// Unique categories in catalog order.
    pub fn categories(&self) -> Vec<String> {
        self.entries
            .iter()
            .scan(HashSet::new(), |seen, p| {
                Some(if seen.insert(p.category.clone()) {
                    Some(p.category.clone())
                } else {
                    None
                })
            })
            .flatten()
            .collect()
    }

    pub fn providers_by_category(&self, category: &str) -> Vec<&ServiceDescriptor> {
        self.entries
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    pub fn builtin_secret_keys(&self) -> HashSet<String> {
        self.entries
            .iter()
            .filter(|p| !p.key_var.is_empty())
            .map(|p| p.key_var.clone())
            .collect()
    }
}

/// Legacy free function preserved for backward compat.
pub fn load_index() -> anyhow::Result<ProviderIndex> {
    Ok(EMBEDDED.clone())
}
