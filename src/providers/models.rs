use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use ureq::config::Config;

const MODELS_DEV_URL: &str = "https://models.dev/api.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimits {
    pub context: Option<u64>,
    pub input: Option<u64>,
    pub output: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub limits: Option<ModelLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevPayload {
    pub models: Vec<ModelEntry>,
}

pub fn fetch_and_cache(cache_path: &str) -> anyhow::Result<ModelsDevPayload> {
    let config = Config::builder()
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let mut resp = agent.get(MODELS_DEV_URL).call()?;

    let payload: ModelsDevPayload = resp.body_mut().read_json()?;

    if let Some(parent) = Path::new(cache_path).parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&payload)?;
    crate::config::atomic::write_atomic(cache_path, json.as_bytes(), 0o644)?;
    Ok(payload)
}

pub fn load_cache(cache_path: &str) -> anyhow::Result<Option<ModelsDevPayload>> {
    if !Path::new(cache_path).exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(cache_path)?;
    let payload: ModelsDevPayload = serde_json::from_str(&data)?;
    Ok(Some(payload))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaTag {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaResponse {
    models: Vec<OllamaTag>,
}

pub fn fetch_ollama_models(base_url: &str) -> anyhow::Result<Vec<String>> {
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let config = Config::builder()
        .timeout_global(Some(std::time::Duration::from_secs(2)))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let mut resp = agent.get(&url).call()?;

    let payload: OllamaResponse = resp.body_mut().read_json()?;
    Ok(payload.models.into_iter().map(|m| m.name).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIModelEntry {
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponse {
    data: Vec<OpenAIModelEntry>,
}

pub fn fetch_openai_compatible_models(base_url: &str) -> anyhow::Result<Vec<String>> {
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    let config = Config::builder()
        .timeout_global(Some(std::time::Duration::from_secs(2)))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let mut resp = agent.get(&url).call()?;

    let payload: OpenAIResponse = resp.body_mut().read_json()?;
    Ok(payload.data.into_iter().map(|m| m.id).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mock_payload() {
        let raw = r#"{
            "models": [
                {
                    "id": "anthropic/claude-3-5-sonnet",
                    "name": "Claude 3.5 Sonnet",
                    "provider_id": "anthropic",
                    "limits": {
                        "context": 200000,
                        "input": 200000,
                        "output": 8192
                    }
                }
            ]
        }"#;
        let payload: ModelsDevPayload = serde_json::from_str(raw).unwrap();
        assert_eq!(payload.models.len(), 1);
    }
}
