use std::collections::HashMap;

use claudy::config::registry::AppRegistry;
use claudy::domain::launch_blueprint::LaunchTarget;
use claudy::launcher::env_schema::EnvMap;
use claudy::launcher::overlay::pipeline::prepare_provider_env;

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

/// Invariant: overlay pipeline produces identical output for claude_strict regardless of input.
#[test]
fn test_overlay_differential_claude_strict_early_exit() {
    let target = make_target("claude_strict", "some-model");
    let env = vec!["PATH=/usr/bin".to_string()];
    let cfg = AppRegistry::default();
    let (result, cleanup) = prepare_provider_env(&target, &[], &env, &cfg).unwrap();
    cleanup();
    assert_eq!(result, env);
}

/// Invariant: model resolution from CLI always takes priority over env/target.
#[test]
fn test_model_resolution_priority_cli_over_env() {
    let target = make_target("anthropic_compatible_non_claude", "target-model");
    let env_map = EnvMap::from_env_slice_lenient(&["ANTHROPIC_MODEL=env-model".to_string()]);
    let args = vec!["--model".to_string(), "cli-model".to_string()];
    let result = claudy::launcher::overlay::pipeline::resolve_model(&target, &args, &env_map);
    assert_eq!(result.session_model, "cli-model");
    assert!(
        matches!(result.source, claudy::launcher::overlay::stages::ModelSource::CliOverride(ref s) if s == "cli-model")
    );
}

/// Invariant: EnvMap rejects what the old env_slice_to_map silently dropped.
#[test]
fn test_env_map_rejects_malformed_keys() {
    let pairs = vec!["VALID=value".to_string(), "=novaluekey".to_string()];
    let result = EnvMap::from_env_slice(&pairs);
    assert!(result.is_err(), "EnvMap should reject empty key");
}

/// Invariant: EnvMap accepts underscore-prefixed keys (common in CI env vars).
#[test]
fn test_env_map_accepts_underscore_prefix() {
    let pairs = vec!["_CI_TOKEN=secret".to_string()];
    let map = EnvMap::from_env_slice(&pairs).expect("parse");
    assert_eq!(map.get("_CI_TOKEN"), Some("secret"));
}
