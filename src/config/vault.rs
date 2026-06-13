use std::path::Path;

use crate::providers::index::ProviderIndex;

// Re-export SecretVault and redact_credential from llm-kernel.
// The struct, Deref/DerefMut, load_from, persist_to, etc. all come from there.
pub use llm_kernel::secrets::SecretVault;
pub use llm_kernel::secrets::redact_credential;

// --- Free functions (public API kept for backward compat) ---

pub fn load_vault(path: impl AsRef<Path>) -> anyhow::Result<SecretVault> {
    Ok(SecretVault::load_from(path)?)
}

pub fn persist_vault(path: impl AsRef<Path>, secrets: &SecretVault) -> anyhow::Result<()> {
    Ok(secrets.persist_to(path)?)
}

/// Strip entries that were valid in older versions but are now redundant.
pub fn prune_outdated_entries(secrets: &mut SecretVault, catalog: &ProviderIndex) {
    let builtin_keys = catalog.builtin_secret_keys();
    let stale: Vec<String> = secrets
        .iter()
        .filter(|(k, v)| is_stale_legacy_entry(k, v, &builtin_keys))
        .map(|(k, _)| k.clone())
        .collect();
    for k in &stale {
        tracing::debug!(key = %k, "pruning stale vault entry");
        secrets.remove(k);
    }
}

/// Redact a credential value for safe display.
///
/// Alias for [`redact_credential`]. Kept for backward compatibility.
pub fn redact(value: &str) -> String {
    redact_credential(value)
}

// --- Internal helpers ---

fn is_stale_legacy_entry(
    key: &str,
    val: &str,
    builtin_keys: &std::collections::HashSet<String>,
) -> bool {
    if let Some(suffix) = key.strip_prefix("OPENROUTER_MODEL_") {
        let alias = crate::config::registry::normalize_openrouter_name(
            &suffix.to_lowercase().replace('_', "-"),
        );
        return alias.is_empty() || crate::config::registry::is_launcher_placeholder(val);
    }
    if let Some(rest) = key
        .strip_prefix("CLAUDY_")
        .and_then(|r| r.strip_suffix("_BASE_URL"))
    {
        return builtin_keys.contains(rest);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::index as providers;
    use std::collections::{HashMap, HashSet};

    fn load_catalog() -> providers::ProviderIndex {
        providers::load_index().expect("catalog should load")
    }

    #[test]
    fn test_prune_outdated_drops_invalid_entries() {
        let catalog = load_catalog();
        let mut secrets = SecretVault::from(HashMap::from([
            (
                "OPENROUTER_MODEL_CLAUDY_OR_KIMI_K25".to_string(),
                "claudy-or-kimi-k25".to_string(),
            ),
            (
                "OPENROUTER_MODEL_KIMI_K25".to_string(),
                "moonshotai/kimi-k2.5".to_string(),
            ),
            (
                "CLAUDY_ALIBABA_API_KEY_BASE_URL".to_string(),
                "https://example.com/unused".to_string(),
            ),
            ("ALIBABA_API_KEY".to_string(), "secret".to_string()),
        ]));

        prune_outdated_entries(&mut secrets, &catalog);

        assert!(
            !secrets.contains_key("OPENROUTER_MODEL_CLAUDY_OR_KIMI_K25"),
            "expected invalid OpenRouter launcher-shaped entry to be removed"
        );
        assert!(
            !secrets.contains_key("CLAUDY_ALIBABA_API_KEY_BASE_URL"),
            "expected builtin provider legacy base URL to be removed"
        );
        assert_eq!(
            secrets.get("OPENROUTER_MODEL_KIMI_K25").map(|s| s.as_str()),
            Some("moonshotai/kimi-k2.5"),
        );
    }

    #[test]
    fn test_roundtrip_via_impl_methods() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("secrets.env");

        let secrets = SecretVault::from(HashMap::from([
            ("MY_KEY".to_string(), "my-value".to_string()),
            ("OTHER_KEY".to_string(), "other".to_string()),
        ]));

        secrets.persist_to(&path).expect("persist");
        let loaded = SecretVault::load_from(&path).expect("load");

        assert_eq!(loaded.get("MY_KEY").map(|s| s.as_str()), Some("my-value"));
        assert_eq!(loaded.get("OTHER_KEY").map(|s| s.as_str()), Some("other"));
    }

    #[test]
    fn test_load_missing_returns_empty() {
        let secrets =
            SecretVault::load_from("/nonexistent/path/secrets.env").expect("load missing");
        assert!(secrets.is_empty());
    }

    #[test]
    fn test_roundtrip_with_special_chars() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("secrets.env");

        let secrets = SecretVault::from(HashMap::from([(
            "MY_KEY".to_string(),
            "value with spaces\nand newlines".to_string(),
        )]));

        secrets.persist_to(&path).expect("persist");
        let loaded = SecretVault::load_from(&path).expect("load");

        assert_eq!(
            loaded.get("MY_KEY").map(|s| s.as_str()),
            Some("value with spaces\nand newlines")
        );
    }

    // --- is_stale_legacy_entry unit tests ---

    #[test]
    fn test_stale_openrouter_launcher_placeholder() {
        let keys = HashSet::new();
        assert!(is_stale_legacy_entry(
            "OPENROUTER_MODEL_CLAUDY_OR_KIMI_K25",
            "claudy-or-kimi-k25",
            &keys,
        ));
    }

    #[test]
    fn test_stale_openrouter_empty_alias() {
        let keys = HashSet::new();
        // After normalization, "CLAUDY-OR-" prefix is stripped, leaving empty → stale
        assert!(is_stale_legacy_entry(
            "OPENROUTER_MODEL_CLAUDY_OR_",
            "some-value",
            &keys,
        ));
    }

    #[test]
    fn test_not_stale_openrouter_valid_model() {
        let keys = HashSet::new();
        assert!(!is_stale_legacy_entry(
            "OPENROUTER_MODEL_KIMI_K25",
            "moonshotai/kimi-k2.5",
            &keys,
        ));
    }

    #[test]
    fn test_stale_claudy_base_url_for_builtin() {
        let keys = HashSet::from(["ALIBABA_API_KEY".to_string()]);
        assert!(is_stale_legacy_entry(
            "CLAUDY_ALIBABA_API_KEY_BASE_URL",
            "https://example.com/unused",
            &keys,
        ));
    }

    #[test]
    fn test_not_stale_claudy_base_url_for_custom() {
        let keys = HashSet::new(); // no builtins → custom provider URL is NOT stale
        assert!(!is_stale_legacy_entry(
            "CLAUDY_MYCUSTOM_API_KEY_BASE_URL",
            "https://mycustom.example.com",
            &keys,
        ));
    }

    #[test]
    fn test_not_stale_unrelated_key() {
        let keys = HashSet::from(["ALIBABA_API_KEY".to_string()]);
        assert!(!is_stale_legacy_entry("SOME_OTHER_KEY", "value", &keys,));
    }
}
