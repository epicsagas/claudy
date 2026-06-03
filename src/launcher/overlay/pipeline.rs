use crate::config::registry::AppRegistry;
use crate::domain::launch_blueprint::LaunchTarget;
use crate::launcher::args;
use crate::launcher::env_schema::EnvMap;

use super::stages::{CleanupHandle, ModelResolution, ModelSource, OverlayMaterialization};

type EnvResult = (Vec<String>, Box<dyn FnOnce()>);

/// Prepare provider-specific environment overrides for Claude Code.
/// Sets ANTHROPIC_MODEL and compaction env vars, letting Claude Code use its
/// original ~/.claude/ directory (skills, plugins, etc.) without a config overlay.
pub fn prepare_provider_env(
    target: &LaunchTarget,
    args: &[String],
    env: &[String],
    config: &AppRegistry,
) -> anyhow::Result<EnvResult> {
    if target.family == "claude_strict" {
        return Ok((env.to_vec(), Box::new(|| {})));
    }

    let mut env_map = EnvMap::from_env_slice_lenient(env);

    // Stage 1: Model resolution
    let resolution = resolve_model(target, args, &env_map);

    // Apply CLI override to env if present
    if let Some(override_model) = args::model_override(args) {
        env_map.set("ANTHROPIC_MODEL", &override_model);
        for key in &[
            "ANTHROPIC_DEFAULT_HAIKU_MODEL",
            "ANTHROPIC_DEFAULT_SONNET_MODEL",
            "ANTHROPIC_DEFAULT_OPUS_MODEL",
            "CLAUDE_CODE_SUBAGENT_MODEL",
        ] {
            env_map.set(key, &override_model);
        }
    }

    let has_claude_vars = env_map.contains_prefix("ANTHROPIC_")
        || env_map.get("CLAUDE_CODE_SUBAGENT_MODEL").is_some();
    if !has_claude_vars {
        return Ok((env_map.to_env_slice(), Box::new(|| {})));
    }

    if resolution.session_model.is_empty() {
        return Ok((env_map.to_env_slice(), Box::new(|| {})));
    }

    // Ensure ANTHROPIC_MODEL is set to the resolved session model.
    // This overrides whatever model is in ~/.claude/settings.json.
    env_map.set("ANTHROPIC_MODEL", &resolution.session_model);

    // Stage 2: Overlay materialization
    let overlay = materialize_overlay(&resolution.session_model, config);

    // Stage 3: Apply compaction env vars
    for (key, value) in overlay.env_overrides {
        env_map.set(&key, &value);
    }

    // Stage 4: Cleanup (no-op for now)
    let _cleanup = CleanupHandle::noop();

    Ok((env_map.to_env_slice(), Box::new(|| {})))
}

/// Stage 1: Determine which model to use for this session.
pub fn resolve_model(target: &LaunchTarget, args: &[String], env_map: &EnvMap) -> ModelResolution {
    if let Some(override_model) = args::model_override(args) {
        return ModelResolution {
            session_model: override_model.clone(),
            source: ModelSource::CliOverride(override_model),
        };
    }

    for key in &[
        "ANTHROPIC_MODEL",
        "ANTHROPIC_DEFAULT_OPUS_MODEL",
        "ANTHROPIC_DEFAULT_SONNET_MODEL",
        "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        "CLAUDE_CODE_SUBAGENT_MODEL",
    ] {
        if let Some(model) = env_map.get(key) {
            let trimmed = model.trim();
            if !trimmed.is_empty() {
                return ModelResolution {
                    session_model: trimmed.to_string(),
                    source: ModelSource::EnvVar(trimmed.to_string()),
                };
            }
        }
    }

    let model = target.model.trim();
    if !model.is_empty() {
        return ModelResolution {
            session_model: model.to_string(),
            source: ModelSource::TargetDefault(model.to_string()),
        };
    }

    for key in &["opus", "sonnet", "haiku", "small"] {
        if let Some(model) = target.model_tiers.get(*key) {
            let trimmed = model.trim();
            if !trimmed.is_empty() {
                return ModelResolution {
                    session_model: trimmed.to_string(),
                    source: ModelSource::TierFallback {
                        tier: key.to_string(),
                        model: trimmed.to_string(),
                    },
                };
            }
        }
    }

    ModelResolution {
        session_model: String::new(),
        source: ModelSource::TargetDefault(String::new()),
    }
}

/// Stage 2: Build Claude Code compaction env vars.
pub fn materialize_overlay(model: &str, config: &AppRegistry) -> OverlayMaterialization {
    OverlayMaterialization {
        env_overrides: build_compaction_env_vars(model, config),
    }
}

/// Translate claudy config into Claude Code's recognized compaction env vars.
///
/// Mapping:
/// - `auto_compact: true` + threshold → `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` (1-100)
/// - `auto_compact: false` → `DISABLE_AUTO_COMPACT=1`
/// - `max_context_tokens` → `CLAUDE_CODE_AUTO_COMPACT_WINDOW` (token count)
fn build_compaction_env_vars(model: &str, config: &AppRegistry) -> Vec<(String, String)> {
    let settings = config.model_settings.get(model);
    let global_compaction = &config.compaction;
    let mut vars = Vec::new();

    if global_compaction.auto_compact {
        let threshold = settings
            .and_then(|s| s.compaction_threshold)
            .unwrap_or(global_compaction.threshold);
        let pct = (threshold * 100.0).round().clamp(1.0, 100.0) as u32;
        vars.push((
            "CLAUDE_AUTOCOMPACT_PCT_OVERRIDE".to_string(),
            pct.to_string(),
        ));
    } else {
        vars.push(("DISABLE_AUTO_COMPACT".to_string(), "1".to_string()));
    }

    if let Some(max_tokens) = settings.and_then(|s| s.max_context_tokens) {
        vars.push((
            "CLAUDE_CODE_AUTO_COMPACT_WINDOW".to_string(),
            max_tokens.to_string(),
        ));
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::launch_blueprint::LaunchTarget;
    use std::collections::HashMap;

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

    fn env_to_map(env: &[String]) -> HashMap<String, String> {
        env.iter()
            .filter_map(|s| s.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_prepare_env_skips_native_claude() {
        let target = make_target("claude_strict", "");
        let env = vec!["PATH=/usr/bin".to_string()];
        let cfg = AppRegistry::default();

        let (result_env, cleanup) = prepare_provider_env(&target, &[], &env, &cfg).expect("env");
        cleanup();

        let map = env_to_map(&result_env);
        assert_eq!(map.get("CLAUDE_CONFIG_DIR").map(|s| s.as_str()), None);
    }

    #[test]
    fn test_prepare_env_sets_anthropic_model() {
        let target = make_target("anthropic_compatible_non_claude", "glm-5");
        let env = vec![
            "PATH=/usr/bin".to_string(),
            "ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic".to_string(),
            "ANTHROPIC_AUTH_TOKEN=test-key".to_string(),
        ];
        let cfg = AppRegistry::default();

        let (result_env, cleanup) = prepare_provider_env(&target, &[], &env, &cfg).expect("env");
        cleanup();

        let map = env_to_map(&result_env);
        assert_eq!(
            map.get("ANTHROPIC_MODEL").map(|s| s.as_str()),
            Some("glm-5")
        );
        assert_eq!(map.get("CLAUDE_CONFIG_DIR").map(|s| s.as_str()), None);
    }

    #[test]
    fn test_prepare_env_resolves_model_from_tiers() {
        let mut target = make_target("anthropic_compatible_non_claude", "");
        target.model_tiers = HashMap::from([("opus".to_string(), "glm-5".to_string())]);
        let env = vec![
            "PATH=/usr/bin".to_string(),
            "ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic".to_string(),
            "ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5".to_string(),
            "ANTHROPIC_AUTH_TOKEN=test-key".to_string(),
        ];
        let cfg = AppRegistry::default();

        let (result_env, cleanup) = prepare_provider_env(&target, &[], &env, &cfg).expect("env");
        cleanup();

        let map = env_to_map(&result_env);
        assert_eq!(
            map.get("ANTHROPIC_MODEL").map(|s| s.as_str()),
            Some("glm-5")
        );
    }

    #[test]
    fn test_resolve_model_from_target_default() {
        let target = make_target("anthropic_compatible_non_claude", "glm-5");
        let env_map = EnvMap::from_env_slice_lenient(&[]);
        let result = resolve_model(&target, &[], &env_map);
        assert_eq!(result.session_model, "glm-5");
        assert!(matches!(result.source, ModelSource::TargetDefault(_)));
    }

    #[test]
    fn test_resolve_model_from_tier_fallback() {
        let mut target = make_target("anthropic_compatible_non_claude", "");
        target.model_tiers = HashMap::from([("opus".to_string(), "glm-5".to_string())]);
        let env_map = EnvMap::from_env_slice_lenient(&[]);
        let result = resolve_model(&target, &[], &env_map);
        assert_eq!(result.session_model, "glm-5");
        assert!(matches!(result.source, ModelSource::TierFallback { .. }));
    }

    #[test]
    fn test_overlay_auto_compact_off_sets_disable() {
        let cfg = AppRegistry {
            compaction: crate::config::registry::ContextWindowPolicy {
                auto_compact: false,
                threshold: 0.8,
            },
            ..AppRegistry::default()
        };
        let vars = build_compaction_env_vars("any-model", &cfg);
        let map: HashMap<_, _> = vars.into_iter().collect();
        assert_eq!(
            map.get("DISABLE_AUTO_COMPACT").map(|s| s.as_str()),
            Some("1")
        );
        assert_eq!(map.get("CLAUDE_AUTOCOMPACT_PCT_OVERRIDE"), None);
    }

    #[test]
    fn test_overlay_auto_compact_on_sets_pct() {
        let cfg = AppRegistry {
            compaction: crate::config::registry::ContextWindowPolicy {
                auto_compact: true,
                threshold: 0.7,
            },
            ..AppRegistry::default()
        };
        let vars = build_compaction_env_vars("any-model", &cfg);
        let map: HashMap<_, _> = vars.into_iter().collect();
        assert_eq!(
            map.get("CLAUDE_AUTOCOMPACT_PCT_OVERRIDE")
                .map(|s| s.as_str()),
            Some("70")
        );
    }

    #[test]
    fn test_overlay_max_context_tokens_sets_window() {
        let mut cfg = AppRegistry::default();
        cfg.model_settings.insert(
            "glm-5".to_string(),
            crate::config::registry::PerModelOverrides {
                max_context_tokens: Some(128000),
                compaction_threshold: None,
            },
        );
        let vars = build_compaction_env_vars("glm-5", &cfg);
        let map: HashMap<_, _> = vars.into_iter().collect();
        assert_eq!(
            map.get("CLAUDE_CODE_AUTO_COMPACT_WINDOW")
                .map(|s| s.as_str()),
            Some("128000")
        );
    }

    #[test]
    fn test_overlay_per_model_threshold_overrides_global() {
        let mut cfg = AppRegistry {
            compaction: crate::config::registry::ContextWindowPolicy {
                auto_compact: true,
                threshold: 0.8,
            },
            ..AppRegistry::default()
        };
        cfg.model_settings.insert(
            "glm-5".to_string(),
            crate::config::registry::PerModelOverrides {
                max_context_tokens: None,
                compaction_threshold: Some(0.5),
            },
        );
        let vars = build_compaction_env_vars("glm-5", &cfg);
        let map: HashMap<_, _> = vars.into_iter().collect();
        assert_eq!(
            map.get("CLAUDE_AUTOCOMPACT_PCT_OVERRIDE")
                .map(|s| s.as_str()),
            Some("50")
        );
    }

    #[test]
    fn test_overlay_threshold_clamped_to_range() {
        let cfg = AppRegistry {
            compaction: crate::config::registry::ContextWindowPolicy {
                auto_compact: true,
                threshold: 1.5, // exceeds 1.0
            },
            ..AppRegistry::default()
        };
        let vars = build_compaction_env_vars("any-model", &cfg);
        let map: HashMap<_, _> = vars.into_iter().collect();
        assert_eq!(
            map.get("CLAUDE_AUTOCOMPACT_PCT_OVERRIDE")
                .map(|s| s.as_str()),
            Some("100")
        );
    }
}
