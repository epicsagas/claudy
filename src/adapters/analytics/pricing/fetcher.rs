use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ureq::config::Config;

const MODELS_DEV_URL: &str = "https://models.dev/api.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelsDevCost {
    pub input: Option<f64>,
    pub output: Option<f64>,
    pub cache_read: Option<f64>,
    pub cache_write: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelsDevEntry {
    pub id: String,
    pub name: Option<String>,
    #[serde(default)]
    pub cost: Option<ModelsDevCost>,
}

// Raw deserialization types matching the actual API shape:
// { "<provider_id>": { "models": { "<model_id>": { ... } } } }
#[derive(Debug, Deserialize)]
struct RawModel {
    id: Option<String>,
    name: Option<String>,
    #[serde(default)]
    cost: Option<ModelsDevCost>,
}

#[derive(Debug, Deserialize)]
struct RawProvider {
    #[serde(default)]
    models: HashMap<String, RawModel>,
}

fn parse_raw_response(raw: HashMap<String, RawProvider>) -> Vec<ModelsDevEntry> {
    // Collect all models across all providers, de-duplicating by model id.
    // When the same model id appears in multiple providers, prefer the entry
    // that has the most complete cost data (cache_read present wins).
    let mut by_id: HashMap<String, ModelsDevEntry> = HashMap::new();

    for (_provider_id, provider) in raw {
        for (model_key, raw_model) in provider.models {
            let id = raw_model.id.unwrap_or(model_key);
            let entry = ModelsDevEntry {
                id: id.clone(),
                name: raw_model.name,
                cost: raw_model.cost,
            };
            by_id
                .entry(id)
                .and_modify(|existing| {
                    // Prefer the richer cost entry (one that has cache_read)
                    let incoming_has_cache =
                        entry.cost.as_ref().and_then(|c| c.cache_read).is_some();
                    let existing_has_cache =
                        existing.cost.as_ref().and_then(|c| c.cache_read).is_some();
                    if incoming_has_cache && !existing_has_cache {
                        *existing = entry.clone();
                    }
                })
                .or_insert(entry);
        }
    }

    by_id.into_values().collect()
}

pub struct ModelsDev {
    cache_path: PathBuf,
}

impl ModelsDev {
    pub fn new(cache_path: PathBuf) -> Self {
        Self { cache_path }
    }

    pub fn fetch_and_cache(&self) -> anyhow::Result<Vec<ModelsDevEntry>> {
        let config = Config::builder()
            .timeout_global(Some(std::time::Duration::from_secs(10)))
            .build();
        let agent = ureq::Agent::new_with_config(config);
        let mut resp = agent.get(MODELS_DEV_URL).call()?;

        let raw: HashMap<String, RawProvider> = resp.body_mut().read_json()?;
        let entries = parse_raw_response(raw);

        self.write_cache_atomic(&entries)?;

        Ok(entries)
    }

    /// Serialize `entries` to JSON and write atomically (temp-file + rename).
    pub fn write_cache_atomic(&self, entries: &[ModelsDevEntry]) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(entries)?;
        crate::config::atomic::write_atomic(
            self.cache_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid cache path"))?,
            json.as_bytes(),
            0o644,
        )
    }

    pub fn load_cache(&self) -> anyhow::Result<Vec<ModelsDevEntry>> {
        let data = fs::read_to_string(&self.cache_path)?;
        let entries: Vec<ModelsDevEntry> = serde_json::from_str(&data)?;
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_raw_response_extracts_models() {
        let json = r#"{
            "anthropic": {
                "models": {
                    "claude-sonnet-4-6": {
                        "id": "claude-sonnet-4-6",
                        "name": "Claude Sonnet 4.6",
                        "cost": { "input": 3.0, "output": 15.0 }
                    }
                }
            }
        }"#;
        let raw: HashMap<String, RawProvider> = serde_json::from_str(json).unwrap();
        let entries = parse_raw_response(raw);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "claude-sonnet-4-6");
        let cost = entries[0].cost.as_ref().unwrap();
        assert_eq!(cost.input, Some(3.0));
    }

    #[test]
    fn test_dedup_prefers_entry_with_cache_read() {
        let json = r#"{
            "provider_a": {
                "models": {
                    "claude-opus-4-7": {
                        "id": "claude-opus-4-7",
                        "cost": { "input": 5.0, "output": 25.0 }
                    }
                }
            },
            "provider_b": {
                "models": {
                    "claude-opus-4-7": {
                        "id": "claude-opus-4-7",
                        "cost": { "input": 5.0, "output": 25.0, "cache_read": 0.5, "cache_write": 6.25 }
                    }
                }
            }
        }"#;
        let raw: HashMap<String, RawProvider> = serde_json::from_str(json).unwrap();
        let entries = parse_raw_response(raw);
        assert_eq!(entries.len(), 1);
        let cost = entries[0].cost.as_ref().unwrap();
        assert_eq!(cost.cache_read, Some(0.5));
    }

    #[test]
    fn test_load_cache_returns_entries() {
        let json = r#"[
            {
                "id": "claude-sonnet-4-6",
                "name": "Claude Sonnet 4.6",
                "cost": { "input": 3.0, "output": 15.0, "cache_read": 0.3, "cache_write": 3.75 }
            }
        ]"#;
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(json.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let fetcher = ModelsDev::new(tmp.path().to_path_buf());
        let entries = fetcher.load_cache().unwrap();
        assert_eq!(entries.len(), 1);
        let cost = entries[0].cost.as_ref().unwrap();
        assert_eq!(cost.input, Some(3.0));
    }

    #[test]
    fn test_missing_cost_fields_ok() {
        let json = r#"[{ "id": "some-model-without-cost" }]"#;
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(json.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let fetcher = ModelsDev::new(tmp.path().to_path_buf());
        let entries = fetcher.load_cache().unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].cost.is_none());
    }

    /// Verify that write_cache_atomic writes valid JSON that load_cache can read back.
    /// This exercises the atomic write path end-to-end without a network call.
    #[test]
    fn test_write_cache_atomic_round_trips() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let fetcher = ModelsDev::new(cache_path.clone());

        let entries = vec![ModelsDevEntry {
            id: "claude-atomic-test".to_string(),
            name: Some("Atomic Test".to_string()),
            cost: Some(ModelsDevCost {
                input: Some(1.0),
                output: Some(5.0),
                cache_read: Some(0.1),
                cache_write: Some(1.25),
            }),
        }];

        fetcher.write_cache_atomic(&entries).unwrap();
        assert!(
            cache_path.exists(),
            "cache file must exist after atomic write"
        );

        let loaded = fetcher.load_cache().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "claude-atomic-test");
        let cost = loaded[0].cost.as_ref().unwrap();
        assert_eq!(cost.input, Some(1.0));
    }
}
