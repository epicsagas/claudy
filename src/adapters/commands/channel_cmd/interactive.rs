use crate::config::registry::{open_registry, write_registry};
use crate::config::vault::{load_vault, persist_vault, redact_credential};
use crate::domain::context::Context;

const TELEGRAM_BOT_TOKEN_KEY: &str = "TELEGRAM_BOT_TOKEN";
const SLACK_BOT_TOKEN_KEY: &str = "SLACK_BOT_TOKEN";
const DISCORD_BOT_TOKEN_KEY: &str = "DISCORD_BOT_TOKEN";

const VALID_PLATFORMS: &[&str] = &["telegram", "slack", "discord"];

fn bot_token_key(platform: &str) -> &'static str {
    match platform {
        "telegram" => TELEGRAM_BOT_TOKEN_KEY,
        "slack" => SLACK_BOT_TOKEN_KEY,
        "discord" => DISCORD_BOT_TOKEN_KEY,
        _ => "",
    }
}

fn validate_platform(platform: &str) -> anyhow::Result<()> {
    if VALID_PLATFORMS.contains(&platform) {
        Ok(())
    } else {
        anyhow::bail!(
            "Unknown platform '{}'. Valid platforms: {}",
            platform,
            VALID_PLATFORMS.join(", ")
        )
    }
}

fn list_modes(modes_dir: &str) -> Vec<String> {
    std::fs::read_dir(modes_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn prompt_bot_token(ctx: &mut Context, platform: &str) -> anyhow::Result<String> {
    let key = bot_token_key(platform);
    let existing = ctx.secrets.get(key).cloned().unwrap_or_default();
    let display_label = format!(
        "{} bot token{}",
        platform,
        if existing.is_empty() {
            String::new()
        } else {
            format!(" (current: {})", redact_credential(&existing))
        }
    );

    let token = if existing.is_empty() {
        let t = ctx.prompt.prompt_secret(&display_label)?;
        if t.trim().is_empty() {
            ctx.output.error("Bot token is required.");
            return Ok(String::new());
        }
        t.trim().to_string()
    } else {
        let t = ctx.prompt.prompt_secret_opt(&display_label)?;
        match t {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => existing.clone(),
        }
    };
    Ok(token)
}

fn prompt_allowed_users(ctx: &mut Context, platform: &str) -> anyhow::Result<Vec<String>> {
    let current_users = ctx
        .config
        .channel
        .platform_allowed_users
        .get(platform)
        .cloned()
        .unwrap_or_default()
        .join(", ");
    let users_label = format!(
        "Allowed user IDs/usernames (comma-separated){}",
        if current_users.is_empty() {
            String::new()
        } else {
            format!(" [{}]", current_users)
        }
    );
    let users_input = ctx
        .prompt
        .prompt_opt(&users_label, &current_users)?
        .unwrap_or_default();
    Ok(users_input
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

fn prompt_profile_and_mode(
    ctx: &mut Context,
    platform: &str,
    cfg: &crate::config::registry::AppRegistry,
) -> anyhow::Result<(String, String)> {
    let current_profile = cfg
        .channel
        .platform_profiles
        .get(platform)
        .cloned()
        .unwrap_or_default();
    let default_profile = if current_profile.is_empty() {
        cfg.channel.default_profile.clone()
    } else {
        current_profile.clone()
    };
    let profile_label = format!("Provider profile for {} [{}]", platform, default_profile);
    let profile_input = ctx
        .prompt
        .prompt_opt(&profile_label, &default_profile)?
        .unwrap_or_default();

    let available_modes = list_modes(&ctx.paths.modes_dir);
    let current_mode = cfg
        .channel
        .platform_modes
        .get(platform)
        .cloned()
        .unwrap_or_else(|| cfg.channel.default_mode.clone());
    let mode_default = if current_mode.is_empty() {
        "default".to_string()
    } else {
        current_mode.clone()
    };
    let mode_hint = if available_modes.is_empty() {
        " (no modes found in ~/.claudy/modes/)"
    } else {
        &format!(" (available: {})", available_modes.join(", "))
    };
    let mode_label = format!("Mode for {} [{}]{}", platform, mode_default, mode_hint);
    let mode_input = ctx
        .prompt
        .prompt_opt(&mode_label, &mode_default)?
        .unwrap_or_default();

    Ok((profile_input, mode_input))
}

/// Prompt for platform-specific extra secrets (e.g. Discord APPLICATION_ID).
fn prompt_extra_secrets(
    ctx: &mut Context,
    platform: &str,
    secrets: &mut crate::config::vault::SecretVault,
) -> anyhow::Result<()> {
    struct Extra {
        key: &'static str,
        label: &'static str,
        required: bool,
    }
    let extras: &[Extra] = match platform {
        "discord" => &[],
        "slack" => &[
            Extra {
                key: "SLACK_SIGNING_SECRET",
                label: "Signing Secret",
                required: false,
            },
            Extra {
                key: "SLACK_APP_TOKEN",
                label: "App Token (xapp-)",
                required: false,
            },
        ],
        _ => &[],
    };
    for extra in extras {
        let existing = secrets.get(extra.key).cloned().unwrap_or_default();
        let display = format!(
            "{} {}{}",
            platform,
            extra.label,
            if existing.is_empty() {
                String::new()
            } else {
                format!(" (current: {})", redact_credential(&existing))
            }
        );
        let value = if existing.is_empty() {
            let v = ctx.prompt.prompt_secret_opt(&display)?;
            match v {
                Some(s) if !s.trim().is_empty() => s.trim().to_string(),
                _ => {
                    if extra.required {
                        ctx.output.error(&format!("{} is required.", extra.label));
                        return Ok(());
                    }
                    continue;
                }
            }
        } else {
            let v = ctx.prompt.prompt_secret_opt(&display)?;
            match v {
                Some(s) if !s.trim().is_empty() => s.trim().to_string(),
                _ => existing,
            }
        };
        secrets.insert(extra.key.to_string(), value);
    }
    Ok(())
}

/// Prompt for per-channel profile/mode overrides (Discord guilds, Slack channels).
fn prompt_single_channel_override(
    ctx: &mut Context,
    platform: &str,
    cid: &str,
    platform_profile: &str,
    available_modes: &[String],
    cfg: &mut crate::config::registry::AppRegistry,
) -> anyhow::Result<()> {
    let channel_key = format!("{}:{}", platform, cid);

    // Profile
    let current_profile = cfg
        .channel
        .channel_profiles
        .get(&channel_key)
        .cloned()
        .unwrap_or_else(|| platform_profile.to_string());
    let profile_label = format!(
        "Profile for {} channel {} [{}]",
        platform, cid, current_profile
    );
    if let Some(input) = ctx.prompt.prompt_opt(&profile_label, &current_profile)? {
        if !input.is_empty() && input != platform_profile {
            cfg.channel
                .channel_profiles
                .insert(channel_key.clone(), input);
        } else {
            cfg.channel.channel_profiles.remove(&channel_key);
        }
    }

    // Mode
    let current_mode = cfg
        .channel
        .channel_modes
        .get(&channel_key)
        .cloned()
        .unwrap_or_default();
    let mode_default = if current_mode.is_empty() {
        "default".to_string()
    } else {
        current_mode.clone()
    };
    let mode_hint = if available_modes.is_empty() {
        String::new()
    } else {
        format!(" (available: {})", available_modes.join(", "))
    };
    let mode_label = format!(
        "Mode for {} channel {} [{}]{}",
        platform, cid, mode_default, mode_hint
    );
    if let Some(input) = ctx.prompt.prompt_opt(&mode_label, &mode_default)? {
        if !input.is_empty() && input != "default" {
            cfg.channel.channel_modes.insert(channel_key.clone(), input);
        } else {
            cfg.channel.channel_modes.remove(&channel_key);
        }
    }

    Ok(())
}

fn prompt_channel_overrides(
    ctx: &mut Context,
    platform: &str,
    cfg: &mut crate::config::registry::AppRegistry,
) -> anyhow::Result<()> {
    let existing_channels: Vec<String> = cfg
        .channel
        .channel_profiles
        .keys()
        .chain(cfg.channel.channel_modes.keys())
        .filter(|k| k.starts_with(&format!("{}:", platform)))
        .map(|k| k.to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let existing_list = if existing_channels.is_empty() {
        String::new()
    } else {
        format!(" (existing: {})", existing_channels.join(", "))
    };

    let channels_label = format!(
        "{} channel IDs for per-channel overrides (comma-separated, e.g. guild or channel IDs){}",
        platform, existing_list
    );
    let channels_input = ctx
        .prompt
        .prompt_opt(&channels_label, "")?
        .unwrap_or_default();

    let channel_ids: Vec<&str> = channels_input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if channel_ids.is_empty() {
        return Ok(());
    }

    let platform_profile = cfg
        .channel
        .platform_profiles
        .get(platform)
        .cloned()
        .unwrap_or_else(|| cfg.channel.default_profile.clone());

    let available_modes = list_modes(&ctx.paths.modes_dir);

    for cid in &channel_ids {
        prompt_single_channel_override(
            ctx,
            platform,
            cid,
            &platform_profile,
            &available_modes,
            cfg,
        )?;
    }

    Ok(())
}

pub(super) fn run_add(ctx: &mut Context, platform: &str) -> anyhow::Result<i32> {
    validate_platform(platform)?;

    let token = prompt_bot_token(ctx, platform)?;
    if token.is_empty() {
        return Ok(1);
    }

    let allowed_users = prompt_allowed_users(ctx, platform)?;
    if allowed_users.is_empty() {
        ctx.output
            .error("At least one allowed user ID or username is required.");
        return Ok(1);
    }

    let mut cfg = open_registry(&ctx.paths.config_file)?;
    let (profile_input, mode_input) = prompt_profile_and_mode(ctx, platform, &cfg)?;

    let key = bot_token_key(platform);
    let mut secrets = load_vault(&ctx.paths.secrets_file)?;
    secrets.insert(key.to_string(), token);
    prompt_extra_secrets(ctx, platform, &mut secrets)?;
    persist_vault(&ctx.paths.secrets_file, &secrets)?;
    ctx.output
        .success(&format!("Saved {} bot token.", platform));

    cfg.channel
        .platform_allowed_users
        .insert(platform.to_string(), allowed_users);
    if !profile_input.is_empty() {
        cfg.channel
            .platform_profiles
            .insert(platform.to_string(), profile_input);
    }
    if !mode_input.is_empty() && mode_input != "default" {
        cfg.channel
            .platform_modes
            .insert(platform.to_string(), mode_input);
    } else {
        cfg.channel.platform_modes.remove(platform);
    }
    if !cfg.channel.enabled_platforms.iter().any(|p| p == platform) {
        cfg.channel.enabled_platforms.push(platform.to_string());
    }

    // Per-channel profile/mode overrides (Discord guilds, Slack channels)
    if platform == "discord" || platform == "slack" {
        prompt_channel_overrides(ctx, platform, &mut cfg)?;
    }

    write_registry(&ctx.paths.config_file, &cfg)?;

    ctx.output
        .success(&format!("Channel '{}' added and configured.", platform));
    Ok(0)
}

pub(super) fn run_remove(ctx: &mut Context, platform: &str) -> anyhow::Result<i32> {
    validate_platform(platform)?;

    let key = bot_token_key(platform);
    let mut secrets = load_vault(&ctx.paths.secrets_file)?;
    let had_secret = secrets.remove(key).is_some();
    if had_secret {
        persist_vault(&ctx.paths.secrets_file, &secrets)?;
        ctx.output
            .success(&format!("Removed {} bot token.", platform));
    }

    let mut cfg = open_registry(&ctx.paths.config_file)?;
    let before = cfg.channel.enabled_platforms.len();
    cfg.channel.enabled_platforms.retain(|p| p != platform);
    let was_enabled = cfg.channel.enabled_platforms.len() < before;
    if was_enabled {
        write_registry(&ctx.paths.config_file, &cfg)?;
    }

    if !had_secret && !was_enabled {
        ctx.output
            .info(&format!("Platform '{}' was not configured.", platform));
        return Ok(0);
    }

    ctx.output
        .success(&format!("Channel '{}' removed.", platform));
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_platform_accepts_known_values() {
        assert!(validate_platform("telegram").is_ok());
        assert!(validate_platform("slack").is_ok());
        assert!(validate_platform("discord").is_ok());
    }

    #[test]
    fn test_validate_platform_rejects_unknown_value() {
        let err = validate_platform("teams").expect_err("teams should be rejected");
        assert!(err.to_string().contains("Unknown platform 'teams'"));
    }

    #[test]
    fn test_bot_token_key_mapping() {
        assert_eq!(bot_token_key("telegram"), "TELEGRAM_BOT_TOKEN");
        assert_eq!(bot_token_key("slack"), "SLACK_BOT_TOKEN");
        assert_eq!(bot_token_key("discord"), "DISCORD_BOT_TOKEN");
        assert_eq!(bot_token_key("unknown"), "");
    }

    #[test]
    fn test_list_modes_only_returns_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mode1 = dir.path().join("work");
        let mode2 = dir.path().join("personal");
        let file = dir.path().join("README.txt");
        std::fs::create_dir_all(&mode1).expect("create mode1");
        std::fs::create_dir_all(&mode2).expect("create mode2");
        std::fs::write(&file, "not-a-mode").expect("write non-directory file");

        let mut all_modes = list_modes(&dir.path().to_string_lossy());
        all_modes.sort();

        assert_eq!(all_modes, vec!["personal".to_string(), "work".to_string()]);
    }

    #[test]
    fn test_list_modes_returns_empty_for_missing_path() {
        let missing = "/tmp/claudy-modes-path-that-should-not-exist";
        let modes = list_modes(missing);
        assert!(modes.is_empty());
    }
}
