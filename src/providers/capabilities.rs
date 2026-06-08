// Re-export auth types from llm-kernel. ServiceDescriptor impl lives there.
pub use llm_kernel::provider::{AuthStrategy, CapabilityProfile};

use crate::domain::launch_blueprint::LaunchTarget;

fn auth_mode_to_strategy(value: &str) -> AuthStrategy {
    match value {
        "none" => AuthStrategy::None,
        "literal" => AuthStrategy::Literal,
        "secret" => AuthStrategy::Secret,
        _ => AuthStrategy::Unknown,
    }
}

fn clears_api_key_for_family(family: &str) -> bool {
    matches!(family, "openrouter" | "local" | "custom_unknown")
}

fn supports_tiers_for_family(family: &str) -> bool {
    !matches!(family, "claude_strict")
}

impl CapabilityProfile for LaunchTarget {
    fn auth_strategy(&self) -> AuthStrategy {
        auth_mode_to_strategy(&self.auth_mode)
    }

    fn clears_anthropic_api_key(&self) -> bool {
        clears_api_key_for_family(&self.family)
    }

    fn supports_model_tiers(&self) -> bool {
        supports_tiers_for_family(&self.family)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_auth_mode_mapping() {
        assert_eq!(auth_mode_to_strategy("none"), AuthStrategy::None);
        assert_eq!(auth_mode_to_strategy("literal"), AuthStrategy::Literal);
        assert_eq!(auth_mode_to_strategy("secret"), AuthStrategy::Secret);
        assert_eq!(auth_mode_to_strategy("other"), AuthStrategy::Unknown);
    }

    #[test]
    fn test_target_capability_flags() {
        let target = LaunchTarget {
            profile: "or-kimi".to_string(),
            display_name: String::new(),
            description: String::new(),
            category: "openrouter".to_string(),
            family: "openrouter".to_string(),
            base_url: String::new(),
            model: String::new(),
            model_tiers: HashMap::new(),
            auth_mode: "secret".to_string(),
            secret_key: "OPENROUTER_API_KEY".to_string(),
            literal_auth_token: String::new(),
            test_url: String::new(),
        };

        assert_eq!(target.auth_strategy(), AuthStrategy::Secret);
        assert!(target.clears_anthropic_api_key());
        assert!(target.supports_model_tiers());
    }

    #[test]
    fn test_claude_strict_invariants() {
        assert!(!clears_api_key_for_family("claude_strict"));
        assert!(!supports_tiers_for_family("claude_strict"));
        assert_eq!(auth_mode_to_strategy("none"), AuthStrategy::None);
    }

    #[test]
    fn test_openrouter_invariants() {
        assert!(clears_api_key_for_family("openrouter"));
        assert!(supports_tiers_for_family("openrouter"));
    }

    #[test]
    fn test_local_family_clears_api_key() {
        assert!(clears_api_key_for_family("local"));
    }

    #[test]
    fn test_custom_unknown_clears_api_key() {
        assert!(clears_api_key_for_family("custom_unknown"));
    }
}
