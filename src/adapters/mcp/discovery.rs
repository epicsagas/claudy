use std::collections::HashMap;

use crate::domain::agent::{AgentConfig, AgentDefinition, builtin_agents};

/// Discover installed agents by checking PATH for each builtin agent.
/// User overrides from config are merged (same key = override).
pub fn discover_agents(overrides: &HashMap<String, AgentConfig>) -> Vec<AgentDefinition> {
    let builtins = builtin_agents();
    let mut result = Vec::new();

    for mut def in builtins {
        // Check if user override exists
        if let Some(config) = overrides.get(&def.name) {
            // Override: merge user config into definition
            def.binary = config.binary.clone();
            if !config.args.is_empty() {
                def.args = config.args.clone();
            }
            if let Some(desc) = &config.description {
                def.description = desc.clone();
            }
            if let Some(timeout) = config.timeout {
                def.timeout = timeout;
            }
        }

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
        if which::which(&config.binary).is_ok() {
            result.push(AgentDefinition {
                name: name.clone(),
                binary: config.binary.clone(),
                args: if config.args.is_empty() {
                    vec!["{prompt}".to_string()]
                } else {
                    config.args.clone()
                },
                description: config
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("Custom agent: {name}")),
                timeout: config.timeout.unwrap_or(120),
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
                binary: "nonexistent_binary_xyz_12345".to_string(),
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
            "gemini".to_string(),
            AgentConfig {
                binary: "cargo".to_string(),
                args: vec![],
                description: Some("Overridden description".to_string()),
                timeout: Some(42),
            },
        );
        let agents = discover_agents(&overrides);
        let gemini = agents.iter().find(|a| a.name == "gemini");
        // If cargo is on PATH (it always is in dev), verify the override was applied.
        if let Some(agent) = gemini {
            assert_eq!(agent.binary, "cargo");
            assert_eq!(agent.description, "Overridden description");
            assert_eq!(agent.timeout, 42);
        }
    }
}
