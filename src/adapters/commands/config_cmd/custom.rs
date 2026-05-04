use crate::domain::context::Context;

use super::shared::{StepResult, maybe_config_model_limits, persist_config, valid_name};

pub(crate) fn config_custom(ctx: &mut Context) -> anyhow::Result<StepResult> {
    let name = match ctx.prompt.prompt_opt("Provider name", "")? {
        Some(val) => val,
        None => return Ok(StepResult::Back),
    };
    let name = name.trim().to_string();
    if !valid_name(&name) {
        ctx.output
            .error(&format!("The provider name '{:?}' is invalid.", name));
        return Ok(StepResult::Back);
    }

    let existing = ctx
        .config
        .custom_providers
        .get(&name)
        .cloned()
        .unwrap_or_default();

    let url_label = if existing.base_url.is_empty() {
        "Base URL".to_string()
    } else {
        ctx.output
            .info(&format!("Current URL: {}", existing.base_url));
        "Base URL (empty to keep current)".to_string()
    };
    let base_url = match ctx.prompt.prompt_opt(&url_label, "")? {
        Some(val) => val.trim().to_string(),
        None => return Ok(StepResult::Back),
    };
    let base_url = if base_url.is_empty() {
        existing.base_url.clone()
    } else {
        base_url
    };
    if base_url.is_empty() {
        ctx.output.error("A base URL is required.");
        return Ok(StepResult::Back);
    }

    let model_label = if existing.default_model.is_empty() {
        "Default model (optional)".to_string()
    } else {
        format!("Default model (empty to keep {:?})", existing.default_model)
    };
    let default_model = match ctx.prompt.prompt_opt(&model_label, "")? {
        Some(val) => val.trim().to_string(),
        None => return Ok(StepResult::Back),
    };
    let default_model = if default_model.is_empty() {
        existing.default_model.clone()
    } else {
        default_model
    };

    if !default_model.is_empty() {
        maybe_config_model_limits(ctx, &default_model)?;
    }

    let key_var = name.to_uppercase().replace('-', "_") + "_API_KEY";
    let current = ctx
        .secrets
        .get(&key_var)
        .cloned()
        .or_else(|| std::env::var(&key_var).ok())
        .unwrap_or_default();
    let current = current.trim().to_string();
    let key_label = if current.is_empty() {
        "API key"
    } else {
        ctx.output.info(&format!(
            "Current key: {}",
            crate::config::vault::redact(&current)
        ));
        "API key (empty to keep, '!' to unset)"
    };
    match ctx.prompt.prompt_secret_opt(key_label)? {
        Some(api_key) => {
            let api_key = api_key.trim();
            if api_key == "!" {
                ctx.secrets.remove(&key_var);
            } else if !api_key.is_empty() {
                ctx.secrets.insert(key_var.clone(), api_key.to_string());
            }
        }
        None => return Ok(StepResult::Back),
    }

    ctx.config.custom_providers.insert(
        name.clone(),
        crate::config::registry::UserEndpoint {
            name: name.clone(),
            display_name: name.clone(),
            base_url,
            api_key_env: key_var,
            default_model,
        },
    );
    persist_config(ctx)?;
    Ok(StepResult::Next)
}
