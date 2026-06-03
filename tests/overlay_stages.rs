use std::collections::HashMap;

use claudy::config::registry::AppRegistry;
use claudy::domain::launch_blueprint::LaunchTarget;
use claudy::launcher::env_schema::EnvMap;
use claudy::launcher::overlay::pipeline::{
    materialize_overlay, prepare_provider_env, resolve_model,
};
use claudy::launcher::overlay::stages::ModelSource;

fn make_target(family: &str, model: &str) -> LaunchTarget {
    LaunchTarget {
        profile: "test".to_string(),
        display_name: String::new(),
        description: String::new(),
        category: String::new(),
        family: family.to_string(),
        base_url: String::new(),
        model: model.to_string(),
        model_tiers: HashMap::new(),
        auth_mode: "secret".to_string(),
        secret_key: String::new(),
        literal_auth_token: String::new(),
        test_url: String::new(),
    }
}

#[test]
fn test_stage1_model_resolution_from_target_default() {
    let target = make_target("anthropic_compatible_non_claude", "glm-5");
    let env_map = EnvMap::from_env_slice_lenient(&[]);
    let result = resolve_model(&target, &[], &env_map);
    assert_eq!(result.session_model, "glm-5");
    assert!(matches!(result.source, ModelSource::TargetDefault(ref s) if s == "glm-5"));
}

#[test]
fn test_stage1_model_resolution_from_tier_fallback() {
    let mut target = make_target("anthropic_compatible_non_claude", "");
    target.model_tiers = HashMap::from([("opus".to_string(), "glm-5".to_string())]);
    let env_map = EnvMap::from_env_slice_lenient(&[]);
    let result = resolve_model(&target, &[], &env_map);
    assert_eq!(result.session_model, "glm-5");
    assert!(matches!(result.source, ModelSource::TierFallback { ref tier, .. } if tier == "opus"));
}

#[test]
fn test_stage1_model_resolution_from_env_var() {
    let target = make_target("anthropic_compatible_non_claude", "");
    let env_map = EnvMap::from_env_slice_lenient(&["ANTHROPIC_MODEL=env-model".to_string()]);
    let result = resolve_model(&target, &[], &env_map);
    assert_eq!(result.session_model, "env-model");
    assert!(matches!(result.source, ModelSource::EnvVar(ref s) if s == "env-model"));
}

#[test]
fn test_stage1_model_resolution_empty_when_no_source() {
    let target = make_target("anthropic_compatible_non_claude", "");
    let env_map = EnvMap::from_env_slice_lenient(&[]);
    let result = resolve_model(&target, &[], &env_map);
    assert!(result.session_model.is_empty());
}

#[test]
fn test_stage2_overlay_materialization_with_no_settings() {
    let config = AppRegistry::default();
    let result = materialize_overlay("unknown-model", &config);
    // Default compaction is enabled, so env_overrides should contain the pct var
    assert!(!result.env_overrides.is_empty());
    let keys: Vec<&str> = result.env_overrides.iter().map(|(k, _)| k.as_str()).collect();
    assert!(keys.contains(&"CLAUDE_AUTOCOMPACT_PCT_OVERRIDE"));
}

#[test]
fn test_end_to_end_pipeline_claude_strict() {
    let target = make_target("claude_strict", "");
    let env = vec!["PATH=/usr/bin".to_string()];
    let cfg = AppRegistry::default();
    let (result_env, cleanup) = prepare_provider_env(&target, &[], &env, &cfg).unwrap();
    cleanup();
    assert_eq!(result_env, env);
}

#[test]
fn test_end_to_end_pipeline_sets_model() {
    let target = make_target("anthropic_compatible_non_claude", "glm-5");
    let env = vec![
        "PATH=/usr/bin".to_string(),
        "ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic".to_string(),
        "ANTHROPIC_AUTH_TOKEN=test-key".to_string(),
    ];
    let cfg = AppRegistry::default();
    let (result_env, cleanup) = prepare_provider_env(&target, &[], &env, &cfg).unwrap();
    cleanup();

    let map: HashMap<_, _> = result_env
        .iter()
        .filter_map(|s| s.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert_eq!(
        map.get("ANTHROPIC_MODEL").map(|s| s.as_str()),
        Some("glm-5")
    );
}
