use crate::config::registry::GuardSettings;
use crate::ports::guard_ports::{GuardAction, GuardPolicy};

/// MVP inline `GuardPolicy`: direct mapping from `GuardSettings`.
pub struct SettingsPolicy {
    settings: GuardSettings,
}

impl SettingsPolicy {
    pub fn new(settings: GuardSettings) -> Self {
        SettingsPolicy { settings }
    }
}

impl GuardPolicy for SettingsPolicy {
    fn action_for(&self, _provider_id: &str, finding_kind: &str) -> GuardAction {
        if finding_kind == "image" {
            if self.settings.strip_images {
                GuardAction::Redact
            } else {
                GuardAction::Allow
            }
        } else if matches!(
            finding_kind,
            "local_path" | "phone" | "bank_account" | "rrn"
        ) {
            // KoreanPii / FileSystemPath: record-only. Redacting paths or
            // routine PII breaks coding sessions; the ledger still shows them.
            GuardAction::Warn
        } else {
            match self.settings.on_secret {
                crate::config::registry::SecretPolicy::Allow => GuardAction::Allow,
                crate::config::registry::SecretPolicy::Redact => GuardAction::Redact,
                crate::config::registry::SecretPolicy::Warn => GuardAction::Warn,
                crate::config::registry::SecretPolicy::Block => GuardAction::Block,
            }
        }
    }

    fn is_trusted(&self, provider_id: &str) -> bool {
        self.settings
            .trusted_providers
            .iter()
            .any(|p| p == provider_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::registry::SecretPolicy;

    fn settings(on_secret: SecretPolicy, strip_images: bool) -> GuardSettings {
        GuardSettings {
            strip_images,
            on_secret,
            trusted_providers: vec!["native".to_string()],
        }
    }

    #[test]
    fn secret_action_mapped_from_settings() {
        let policy = SettingsPolicy::new(settings(SecretPolicy::Warn, true));
        assert_eq!(policy.action_for("zai", "api_key"), GuardAction::Warn);
        assert_eq!(policy.action_for("native", "aws_key"), GuardAction::Warn);
    }

    #[test]
    fn image_action_respects_strip_images() {
        let policy = SettingsPolicy::new(settings(SecretPolicy::Redact, false));
        assert_eq!(policy.action_for("zai", "image"), GuardAction::Allow);
        let policy = SettingsPolicy::new(settings(SecretPolicy::Redact, true));
        assert_eq!(policy.action_for("zai", "image"), GuardAction::Redact);
    }

    #[test]
    fn trusted_provider_lookup() {
        let policy = SettingsPolicy::new(settings(SecretPolicy::Redact, true));
        assert!(policy.is_trusted("native"));
        assert!(!policy.is_trusted("zai"));
    }
}
