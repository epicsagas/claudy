use std::collections::HashMap;

use crate::config::vault::SecretVault;
use crate::domain::launch_blueprint::LaunchTarget;
use crate::providers::capabilities::{AuthStrategy, CapabilityProfile};

enum AuthContract {
    None,
    Literal { token: String },
    Secret { key: String },
}

struct EnvContract {
    base_url: String,
    model: String,
    model_tiers: HashMap<String, String>,
    clear_api_key: bool,
    auth: AuthContract,
}

impl EnvContract {
    fn from_target(target: &LaunchTarget) -> anyhow::Result<Self> {
        let auth = match target.auth_strategy() {
            AuthStrategy::None => AuthContract::None,
            AuthStrategy::Literal => {
                if target.literal_auth_token.trim().is_empty() {
                    anyhow::bail!("Literal authentication requires a non-empty token.");
                }
                AuthContract::Literal {
                    token: target.literal_auth_token.clone(),
                }
            }
            AuthStrategy::Secret => {
                if target.secret_key.trim().is_empty() {
                    anyhow::bail!("Secret authentication requires a non-empty secret key name.");
                }
                AuthContract::Secret {
                    key: target.secret_key.clone(),
                }
            }
            AuthStrategy::Unknown => anyhow::bail!(
                "The authentication method '{}' is not recognized.",
                target.auth_mode
            ),
        };

        Ok(Self {
            base_url: target.base_url.clone(),
            model: target.model.clone(),
            model_tiers: target.model_tiers.clone(),
            clear_api_key: target.clears_anthropic_api_key(),
            auth,
        })
    }
}

pub struct EnvironmentAssembler {
    vars: HashMap<String, String>,
}

impl EnvironmentAssembler {
    pub fn inherit() -> Self {
        let mut vars = HashMap::new();
        // Start from the current process environment.
        for (k, v) in std::env::vars() {
            vars.insert(k, v);
        }
        // Opt-in: when CLAUDY_SHELL_ENV is truthy, additionally pull in the
        // user's login-shell environment (PATH additions and exports from
        // ~/.zprofile / ~/.zshrc / ~/.bash_profile / ~/.profile). Only keys the
        // current process does NOT already have are merged, so claudy's own
        // values (provider env, explicit overrides) always win. This is
        // off by default: sourcing a shell can be slow and can pull in
        // unexpected state under GUI/launchd contexts. Sourcing failures are
        // swallowed — this must never become a new error path. See issue #41.
        if let Some(shell_vars) = (shell_env_enabled().then(load_login_shell_env)).flatten() {
            for (k, v) in shell_vars {
                vars.entry(k).or_insert(v);
            }
        }
        Self { vars }
    }

    pub fn clear_provider_vars(mut self) -> Self {
        self.vars.retain(|k, _| !k.starts_with("ANTHROPIC_"));
        self
    }

    pub fn set(mut self, key: &str, value: &str) -> Self {
        self.vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn set_if_not_empty(mut self, key: &str, value: &str) -> Self {
        if !value.is_empty() {
            self.vars.insert(key.to_string(), value.to_string());
        }
        self
    }

    pub fn map_tiers(mut self, tiers: &HashMap<String, String>) -> Self {
        for (tier, model) in tiers {
            let key = match tier.as_str() {
                "haiku" => "ANTHROPIC_DEFAULT_HAIKU_MODEL",
                "sonnet" => "ANTHROPIC_DEFAULT_SONNET_MODEL",
                "opus" => "ANTHROPIC_DEFAULT_OPUS_MODEL",
                _ => continue,
            };
            self.vars.insert(key.to_string(), model.to_string());
        }
        self
    }

    pub fn build(self) -> Vec<String> {
        self.vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect()
    }
}

pub fn build_auth_environment(
    target: &LaunchTarget,
    secrets: &SecretVault,
) -> anyhow::Result<Vec<String>> {
    let contract = EnvContract::from_target(target)?;
    let mut builder = EnvironmentAssembler::inherit()
        .clear_provider_vars()
        .set_if_not_empty("ANTHROPIC_BASE_URL", &contract.base_url)
        .set_if_not_empty("ANTHROPIC_MODEL", &contract.model)
        .map_tiers(&contract.model_tiers);

    match contract.auth {
        AuthContract::None => {}
        AuthContract::Literal { token } => {
            builder = builder
                .set("ANTHROPIC_AUTH_TOKEN", &token)
                .set("ANTHROPIC_API_KEY", "");
        }
        AuthContract::Secret { key } => {
            let value = secrets
                .get(&key)
                .cloned()
                .or_else(|| std::env::var(&key).ok())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Missing credentials for '{}'. Please configure it using 'claudy setup'.",
                        key
                    )
                })?;

            if value.trim().is_empty() {
                anyhow::bail!(
                    "API key for '{}' is empty. Please check your configuration.",
                    key
                );
            }
            builder = builder.set("ANTHROPIC_AUTH_TOKEN", &value);
            if contract.clear_api_key {
                builder = builder.set("ANTHROPIC_API_KEY", "");
            }
        }
    }

    Ok(builder.build())
}

pub fn prepare_env(target: &LaunchTarget, secrets: &SecretVault) -> anyhow::Result<Vec<String>> {
    build_auth_environment(target, secrets)
}

pub fn is_homebrew() -> bool {
    if std::env::var("HOMEBREW_PREFIX").is_ok() {
        return true;
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| std::fs::canonicalize(exe).ok())
        .is_some_and(|resolved| resolved.to_string_lossy().contains("/Cellar/"))
}

/// Whether to merge the user's login-shell environment into spawned processes.
///
/// Off by default; enabled by `CLAUDY_SHELL_ENV=1` (or `true`). See issue #41.
fn shell_env_enabled() -> bool {
    matches!(
        std::env::var("CLAUDY_SHELL_ENV").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

/// Capture the user's login-shell environment by evaluating `$SHELL -l -c env`.
///
/// Returns `None` on any failure (no `$SHELL`, spawn error, non-UTF-8 output, or
/// a non-zero exit) so callers can treat this as best-effort. Login mode (`-l`)
/// sources `~/.zprofile` / `~/.bash_profile` / `~/.profile`; we deliberately do
/// NOT pass `-i` (interactive), which depends on a TTY and can hang. Unix-only.
#[cfg(unix)]
fn load_login_shell_env() -> Option<HashMap<String, String>> {
    use std::process::Command;

    let shell = std::env::var("SHELL").ok().filter(|s| !s.is_empty())?;
    // `env -0` emits NUL-separated KEY=VALUE pairs, robust against values that
    // contain newlines.
    let output = Command::new(&shell)
        .args(["-l", "-c", "env -0"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut vars = HashMap::new();
    for chunk in output.stdout.split(|&b| b == 0) {
        if chunk.is_empty() {
            continue;
        }
        let Ok(s) = std::str::from_utf8(chunk) else {
            continue;
        };
        if let Some((k, v)) = s.split_once('=') {
            vars.insert(k.to_string(), v.to_string());
        }
    }
    Some(vars)
}

#[cfg(not(unix))]
fn load_login_shell_env() -> Option<HashMap<String, String>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::launch_blueprint::LaunchTarget;
    use std::env;

    fn env_to_map(env: &[String]) -> HashMap<String, String> {
        env.iter()
            .filter_map(|s| s.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_prepare_env_for_open_router() {
        let target = LaunchTarget {
            profile: "or-kimi".to_string(),
            display_name: "OpenRouter: kimi".to_string(),
            description: String::new(),
            category: "openrouter".to_string(),
            family: "openrouter".to_string(),
            base_url: "https://openrouter.ai/api".to_string(),
            model: String::new(),
            model_tiers: HashMap::from([
                ("haiku".to_string(), "moonshotai/kimi-k2.5".to_string()),
                ("sonnet".to_string(), "moonshotai/kimi-k2.5".to_string()),
                ("opus".to_string(), "moonshotai/kimi-k2.5".to_string()),
            ]),
            auth_mode: "secret".to_string(),
            secret_key: "OPENROUTER_API_KEY".to_string(),
            literal_auth_token: String::new(),
            test_url: String::new(),
        };

        let secrets = SecretVault::from(HashMap::from([(
            "OPENROUTER_API_KEY".to_string(),
            "sk-openrouter".to_string(),
        )]));

        let env = build_auth_environment(&target, &secrets).expect("prepare_env");
        let text: String = env.iter().map(|s| format!("{}\n", s)).collect();

        assert!(text.contains("ANTHROPIC_BASE_URL=https://openrouter.ai/api"));
        assert!(text.contains("ANTHROPIC_AUTH_TOKEN=sk-openrouter"));
        assert!(text.contains("ANTHROPIC_API_KEY="));
        assert!(text.contains("ANTHROPIC_DEFAULT_OPUS_MODEL=moonshotai/kimi-k2.5"));
    }

    #[test]
    fn test_prepare_env_custom_provider_clears_api_key() {
        let target = LaunchTarget {
            profile: "myprovider".to_string(),
            display_name: String::new(),
            description: String::new(),
            category: "custom".to_string(),
            family: "custom_unknown".to_string(),
            base_url: "https://api.example.com/anthropic".to_string(),
            model: String::new(),
            model_tiers: HashMap::new(),
            auth_mode: "secret".to_string(),
            secret_key: "MYPROVIDER_API_KEY".to_string(),
            literal_auth_token: String::new(),
            test_url: String::new(),
        };

        let secrets = SecretVault::from(HashMap::from([(
            "MYPROVIDER_API_KEY".to_string(),
            "sk-custom".to_string(),
        )]));

        let env = build_auth_environment(&target, &secrets).expect("prepare_env");
        let map = env_to_map(&env);

        assert_eq!(
            map.get("ANTHROPIC_AUTH_TOKEN").map(|s| s.as_str()),
            Some("sk-custom")
        );
        assert_eq!(map.get("ANTHROPIC_API_KEY").map(|s| s.as_str()), Some(""));
    }

    #[test]
    fn test_prepare_env_fails_when_secret_missing() {
        let target = LaunchTarget {
            profile: "nonexistent".to_string(),
            display_name: String::new(),
            description: String::new(),
            category: "builtin".to_string(),
            family: "anthropic_compatible_non_claude".to_string(),
            base_url: "https://api.z.ai/api/anthropic".to_string(),
            model: String::new(),
            model_tiers: HashMap::new(),
            auth_mode: "secret".to_string(),
            secret_key: "NONEXISTENT_KEY_FOR_TESTING_PURPOSES".to_string(),
            literal_auth_token: String::new(),
            test_url: String::new(),
        };

        let result = build_auth_environment(&target, &SecretVault::empty());
        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_prepare_env_clears_unused_tier_variables() {
        unsafe {
            env::set_var("ANTHROPIC_DEFAULT_HAIKU_MODEL", "stale-haiku");
            env::set_var("ANTHROPIC_DEFAULT_SONNET_MODEL", "stale-sonnet");
            env::set_var("ANTHROPIC_DEFAULT_OPUS_MODEL", "stale-opus");
        }

        let target = LaunchTarget {
            profile: "zai".to_string(),
            display_name: String::new(),
            description: String::new(),
            category: "builtin".to_string(),
            family: "anthropic_compatible_non_claude".to_string(),
            base_url: "https://api.z.ai/api/anthropic".to_string(),
            model: String::new(),
            model_tiers: HashMap::from([("opus".to_string(), "glm-5".to_string())]),
            auth_mode: "secret".to_string(),
            secret_key: "ZAI_API_KEY".to_string(),
            literal_auth_token: String::new(),
            test_url: String::new(),
        };

        let secrets = SecretVault::from(HashMap::from([(
            "ZAI_API_KEY".to_string(),
            "sk-zai".to_string(),
        )]));

        let env = build_auth_environment(&target, &secrets).expect("prepare_env");
        let map = env_to_map(&env);

        assert_eq!(
            map.get("ANTHROPIC_DEFAULT_OPUS_MODEL").map(|s| s.as_str()),
            Some("glm-5")
        );
        assert!(!map.contains_key("ANTHROPIC_DEFAULT_HAIKU_MODEL"));
        assert!(!map.contains_key("ANTHROPIC_DEFAULT_SONNET_MODEL"));

        unsafe {
            env::remove_var("ANTHROPIC_DEFAULT_HAIKU_MODEL");
            env::remove_var("ANTHROPIC_DEFAULT_SONNET_MODEL");
            env::remove_var("ANTHROPIC_DEFAULT_OPUS_MODEL");
        }
    }

    #[test]
    fn test_prepare_env_literal_requires_token() {
        let target = LaunchTarget {
            profile: "literal".to_string(),
            display_name: String::new(),
            description: String::new(),
            category: "custom".to_string(),
            family: "custom_unknown".to_string(),
            base_url: String::new(),
            model: String::new(),
            model_tiers: HashMap::new(),
            auth_mode: "literal".to_string(),
            secret_key: String::new(),
            literal_auth_token: "   ".to_string(),
            test_url: String::new(),
        };
        let result = build_auth_environment(&target, &SecretVault::empty());
        assert!(result.is_err());
    }

    #[test]
    fn test_prepare_env_secret_requires_key_name() {
        let target = LaunchTarget {
            profile: "secret".to_string(),
            display_name: String::new(),
            description: String::new(),
            category: "custom".to_string(),
            family: "custom_unknown".to_string(),
            base_url: String::new(),
            model: String::new(),
            model_tiers: HashMap::new(),
            auth_mode: "secret".to_string(),
            secret_key: " ".to_string(),
            literal_auth_token: String::new(),
            test_url: String::new(),
        };
        let result = build_auth_environment(&target, &SecretVault::empty());
        assert!(result.is_err());
    }

    // ── CLAUDY_SHELL_ENV opt-in (issue #41) ─────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_shell_env_disabled_by_default() {
        unsafe {
            env::remove_var("CLAUDY_SHELL_ENV");
        }
        assert!(!shell_env_enabled());
    }

    #[test]
    #[serial_test::serial]
    fn test_shell_env_opt_in_truthy_values() {
        for val in ["1", "true", "TRUE"] {
            unsafe {
                env::set_var("CLAUDY_SHELL_ENV", val);
            }
            assert!(
                shell_env_enabled(),
                "CLAUDY_SHELL_ENV={val} should enable shell env"
            );
        }
        unsafe {
            env::remove_var("CLAUDY_SHELL_ENV");
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_shell_env_opt_in_rejects_other_values() {
        for val in ["0", "false", "yes", ""] {
            unsafe {
                env::set_var("CLAUDY_SHELL_ENV", val);
            }
            assert!(
                !shell_env_enabled(),
                "CLAUDY_SHELL_ENV={val:?} should NOT enable shell env"
            );
        }
        unsafe {
            env::remove_var("CLAUDY_SHELL_ENV");
        }
    }

    /// End-to-end: with CLAUDY_SHELL_ENV=1 and a real $SHELL, the login-shell
    /// env is captured and the inherit() merge fills in keys the process
    /// doesn't already have (e.g. HOME) without clobbering existing ones.
    /// Unix-only; skipped when no $SHELL is set.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn test_inherit_merges_login_shell_env_when_enabled() {
        let Some(shell) = env::var("SHELL").ok().filter(|s| !s.is_empty()) else {
            eprintln!("skipping: $SHELL not set");
            return;
        };
        // Sanity: the configured shell must actually exist for the test to be
        // meaningful (CI runners always have one).
        assert!(
            std::path::Path::new(&shell).exists(),
            "test premise: $SHELL ({shell}) should exist"
        );

        unsafe {
            env::set_var("CLAUDY_SHELL_ENV", "1");
        }
        let merged = EnvironmentAssembler::inherit();
        unsafe {
            env::remove_var("CLAUDY_SHELL_ENV");
        }

        // A login shell always exports HOME — it should be present after merge.
        assert!(
            merged.vars.contains_key("HOME"),
            "HOME missing from merged env; shell sourcing likely failed"
        );

        // Existing process values must survive: PATH is inherited from the
        // process first and only filled-in if absent, so the merged value must
        // equal what the process already had.
        if let Ok(path) = env::var("PATH") {
            assert_eq!(
                merged.vars.get("PATH").map(String::as_str),
                Some(path.as_str())
            );
        }
    }
}
