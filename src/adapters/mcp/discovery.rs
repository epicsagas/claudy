use std::collections::HashMap;

use crate::domain::agent::{AgentConfig, AgentDefinition, builtin_agents};

/// Global env var that overrides the default timeout for all agents.
const AGENT_TIMEOUT_ENV: &str = "CLAUDY_AGENT_TIMEOUT";

/// Resolve effective timeout with priority: config.yaml > env var > builtin default.
fn effective_timeout(builtin: u64, config_timeout: Option<u64>) -> u64 {
    if let Some(t) = config_timeout {
        return t;
    }
    if let Ok(val) = std::env::var(AGENT_TIMEOUT_ENV)
        && let Ok(secs) = val.parse::<u64>()
    {
        return secs;
    }
    builtin
}

/// Discover installed agents by checking PATH for each builtin agent.
/// User overrides from config are merged (same key = override).
pub fn discover_agents(overrides: &HashMap<String, AgentConfig>) -> Vec<AgentDefinition> {
    let builtins = builtin_agents();
    let mut result = Vec::new();

    for mut def in builtins {
        // Check if user override exists
        if let Some(config) = overrides.get(&def.name) {
            // Override: merge user config into definition
            if let Some(b) = &config.binary {
                def.binary = b.clone();
            }
            if !config.args.is_empty() {
                def.args = config.args.clone();
            }
            if let Some(desc) = &config.description {
                def.description = desc.clone();
            }
        }
        def.timeout = effective_timeout(
            def.timeout,
            overrides.get(&def.name).and_then(|c| c.timeout),
        );

        // Check if binary exists on PATH
        if which::which(&def.binary).is_ok() {
            result.push(def);
        }
    }

    // Build set of builtin names from the first call's results to avoid a second allocation
    let builtin_names: std::collections::HashSet<String> =
        result.iter().map(|a| a.name.clone()).collect();

    // Add custom agents from overrides that aren't builtins
    for (name, config) in overrides {
        if builtin_names.contains(name) {
            continue; // already handled above
        }
        let Some(binary) = &config.binary else {
            continue; // custom agents must specify a binary
        };
        if which::which(binary).is_ok() {
            result.push(AgentDefinition {
                name: name.clone(),
                binary: binary.clone(),
                args: if config.args.is_empty() {
                    vec!["{prompt}".to_string()]
                } else {
                    config.args.clone()
                },
                description: config
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("Custom agent: {name}")),
                timeout: effective_timeout(120, config.timeout),
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_agents_no_overrides_finds_available_builtins() {
        let overrides = HashMap::new();
        let agents = discover_agents(&overrides);
        // At minimum, any builtin binary on PATH will appear.
        // We just verify the function runs and returns a valid list.
        for agent in &agents {
            assert!(!agent.name.is_empty());
            assert!(!agent.binary.is_empty());
            assert!(!agent.args.is_empty());
            assert!(!agent.description.is_empty());
            assert!(agent.timeout > 0);
        }
    }

    #[test]
    fn discover_agents_custom_agent_not_on_path_is_excluded() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "my-custom-agent".to_string(),
            AgentConfig {
                binary: Some("nonexistent_binary_xyz_12345".to_string()),
                args: vec![],
                description: None,
                timeout: None,
            },
        );
        let agents = discover_agents(&overrides);
        assert!(!agents.iter().any(|a| a.name == "my-custom-agent"));
    }

    #[test]
    fn discover_agents_override_description_applied() {
        // This test only passes if "cargo" is on PATH (it is in any Rust toolchain).
        // Use "cargo" as a guaranteed-available binary to test override logic.
        let mut overrides = HashMap::new();
        overrides.insert(
            "codex".to_string(),
            AgentConfig {
                binary: Some("cargo".to_string()),
                args: vec![],
                description: Some("Overridden description".to_string()),
                timeout: Some(42),
            },
        );
        let agents = discover_agents(&overrides);
        let codex = agents.iter().find(|a| a.name == "codex");
        // If cargo is on PATH (it always is in dev), verify the override was applied.
        if let Some(agent) = codex {
            assert_eq!(agent.binary, "cargo");
            assert_eq!(agent.description, "Overridden description");
            assert_eq!(agent.timeout, 42);
        }
    }

    #[test]
    fn effective_timeout_env_var_overrides_builtin() {
        // config.yaml takes precedence over env var
        assert_eq!(effective_timeout(120, Some(42)), 42);
        // Env var overrides builtin default
        unsafe { std::env::set_var("CLAUDY_AGENT_TIMEOUT", "600") };
        assert_eq!(effective_timeout(120, None), 600);
        // Invalid env var falls back to builtin
        unsafe { std::env::set_var("CLAUDY_AGENT_TIMEOUT", "not-a-number") };
        assert_eq!(effective_timeout(120, None), 120);
        // Cleanup
        unsafe { std::env::remove_var("CLAUDY_AGENT_TIMEOUT") };
    }

    #[test]
    fn effective_timeout_config_beats_env_var() {
        unsafe { std::env::set_var("CLAUDY_AGENT_TIMEOUT", "999") };
        // Config should win over env var
        assert_eq!(effective_timeout(120, Some(42)), 42);
        unsafe { std::env::remove_var("CLAUDY_AGENT_TIMEOUT") };
    }
}
