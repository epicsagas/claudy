use crate::domain::context::Context;

use super::shared::{
    StepResult, default_alias_name, maybe_config_model_limits, persist_config, valid_name,
};

pub(crate) fn config_open_router(ctx: &mut Context) -> anyhow::Result<StepResult> {
    let current = ctx
        .secrets
        .get("OPENROUTER_API_KEY")
        .cloned()
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .unwrap_or_default();
    let current = current.trim().to_string();
    if !current.is_empty() {
        ctx.output.info(&format!(
            "Current key: {}",
            crate::config::vault::redact(&current)
        ));
    }
    match ctx
        .prompt
        .prompt_secret_opt("OpenRouter API key (empty to keep, '!' to unset)")?
    {
        Some(value) => {
            let value = value.trim();
            if value == "!" {
                ctx.secrets.remove("OPENROUTER_API_KEY");
            } else if !value.is_empty() {
                ctx.secrets
                    .insert("OPENROUTER_API_KEY".to_string(), value.to_string());
            }
        }
        None => return Ok(StepResult::Back),
    }

    loop {
        let model = match ctx
            .prompt
            .prompt_opt("Model ID (Enter to stop, Esc to back)", "")?
        {
            Some(val) => val,
            None => return Ok(StepResult::Back),
        };
        if model.trim().is_empty() {
            break;
        }

        maybe_config_model_limits(ctx, &model)?;

        let name = match ctx
            .prompt
            .prompt_opt("Alias", &default_alias_name(&model))?
        {
            Some(val) => val,
            None => return Ok(StepResult::Back),
        };
        if !valid_name(&name) {
            ctx.output
                .error(&format!("The alias '{:?}' is invalid.", name));
            continue;
        }
        ctx.config.openrouter_aliases.insert(name, model);
    }
    persist_config(ctx)?;
    Ok(StepResult::Next)
}
