#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StepResult {
    Next,
    Back,
}

pub(crate) fn valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-' || *b == b'_')
}

pub(crate) fn default_alias_name(model: &str) -> String {
    let mut model = model.to_lowercase();
    if let Some(slash) = model.rfind('/') {
        model = model[slash + 1..].to_string();
    }
    model = model.replace(['.', '_'], "-");
    model
}

pub(crate) fn persist_config(ctx: &mut crate::domain::context::Context) -> anyhow::Result<()> {
    crate::config::vault::prune_outdated_entries(&mut ctx.secrets, &ctx.catalog);
    crate::config::registry::write_registry(&ctx.paths.config_file, &ctx.config)?;
    crate::config::vault::persist_vault(&ctx.paths.secrets_file, &ctx.secrets)?;
    ctx.output.success("configuration saved");
    Ok(())
}

pub(crate) fn select_model(
    ctx: &mut crate::domain::context::Context,
    label: &str,
    choices: &[crate::providers::index::ModelDescriptor],
    current_val: &str,
    is_default_step: bool,
) -> anyhow::Result<Option<String>> {
    let mut items = Vec::new();
    let current_val = current_val.trim();

    let first_item = if is_default_step {
        if current_val.is_empty() {
            "Use provider default [current]".to_string()
        } else {
            "Use provider default".to_string()
        }
    } else if current_val.is_empty() {
        "(none / use default) [current]".to_string()
    } else {
        "(none / use default)".to_string()
    };
    items.push(first_item);

    let mut found_current = false;
    for m in choices {
        let mut display = format!("{:<24} {}", m.id, m.description);
        if m.id == current_val {
            display = format!("{} [current]", display);
            found_current = true;
        }
        items.push(display);
    }

    let mut custom_item = "Enter custom model ID...".to_string();
    if !current_val.is_empty() && !found_current {
        custom_item = format!("{} [current: {}]", custom_item, current_val);
    }
    items.push(custom_item);

    let default_idx = if current_val.is_empty() {
        0
    } else {
        choices
            .iter()
            .position(|m| m.id == current_val)
            .map(|i| i + 1)
            .unwrap_or(items.len() - 1)
    };

    let selection = match ctx.prompt.select_opt(label, &items, default_idx)? {
        Some(s) => s,
        None => return Ok(None),
    };

    if selection == 0 {
        Ok(Some(String::new()))
    } else if selection == items.len() - 1 {
        ctx.prompt.prompt_opt("Enter custom model ID", current_val)
    } else {
        Ok(Some(choices[selection - 1].id.clone()))
    }
}

pub(crate) fn maybe_config_model_limits(
    ctx: &mut crate::domain::context::Context,
    model_id: &str,
) -> anyhow::Result<()> {
    if !ctx.prompt.confirm(
        &format!(
            "Configure limits (max tokens, compaction) for {}?",
            model_id
        ),
        false,
    )? {
        return Ok(());
    }

    let mut settings = ctx
        .config
        .model_settings
        .get(model_id)
        .cloned()
        .unwrap_or_default();

    let max_tokens_default = settings
        .max_context_tokens
        .map(|t| t.to_string())
        .unwrap_or_else(|| "200000".to_string());
    if let Some(val) = ctx
        .prompt
        .prompt_opt("Max context tokens", &max_tokens_default)?
        && let Ok(num) = val.parse::<u32>()
    {
        settings.max_context_tokens = Some(num);
    }

    let threshold_default = settings
        .compaction_threshold
        .map(|t| t.to_string())
        .unwrap_or_else(|| "0.8".to_string());
    if let Some(val) = ctx
        .prompt
        .prompt_opt("Compaction threshold (0.0-1.0)", &threshold_default)?
        && let Ok(num) = val.parse::<f64>()
    {
        settings.compaction_threshold = Some(num.clamp(0.0, 1.0));
    }

    ctx.config
        .model_settings
        .insert(model_id.to_string(), settings);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_name() {
        assert!(valid_name("my-provider"));
        assert!(valid_name("provider_1"));
        assert!(valid_name("a"));
        assert!(valid_name("0test"));
        assert!(!valid_name(""));
        assert!(!valid_name("UPPER"));
        assert!(!valid_name("has space"));
        assert!(!valid_name("!invalid"));
    }

    #[test]
    fn test_default_alias_name() {
        assert_eq!(default_alias_name("openai/gpt-4"), "gpt-4");
        assert_eq!(default_alias_name("GPT-4"), "gpt-4");
        assert_eq!(default_alias_name("model.v2"), "model-v2");
        assert_eq!(default_alias_name("model_v3"), "model-v3");
    }
}
