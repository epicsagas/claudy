mod builtin;
mod custom;
mod general_settings;
mod openrouter;
mod shared;

use crate::domain::context::Context;

pub fn run_config(ctx: &mut Context, args: &[String]) -> anyhow::Result<i32> {
    if let Some(id) = args.first() {
        if id == "settings" {
            general_settings::config_general_settings(ctx)?;
        } else {
            match id.as_str() {
                "openrouter" => {
                    openrouter::config_open_router(ctx)?;
                }
                "custom" => {
                    custom::config_custom(ctx)?;
                }
                _ => {
                    if let Some(provider) = ctx.catalog.get(id).cloned() {
                        builtin::config_builtin(ctx, &provider)?;
                    } else {
                        anyhow::bail!(
                            "The provider '{:?}' is not recognized. Use 'claudy ls' to see available providers.",
                            id
                        );
                    }
                }
            }
        }
        return Ok(0);
    }

    loop {
        let current_id = match choose_provider(ctx)? {
            Some(id) => id,
            None => return Ok(0),
        };

        if current_id == "done" {
            return Ok(0);
        }

        if current_id == "settings" {
            general_settings::config_general_settings(ctx)?;
            continue;
        }

        let secrets_snapshot = ctx.secrets.clone();
        let config_snapshot = ctx.config.clone();

        let res = match current_id.as_str() {
            "openrouter" => openrouter::config_open_router(ctx)?,
            "custom" => custom::config_custom(ctx)?,
            _ => {
                if let Some(provider) = ctx.catalog.get(&current_id).cloned() {
                    builtin::config_builtin(ctx, &provider)?
                } else {
                    anyhow::bail!("The provider '{:?}' is not recognized.", current_id);
                }
            }
        };

        match res {
            shared::StepResult::Back => {
                ctx.secrets = secrets_snapshot;
                ctx.config = config_snapshot;
                continue;
            }
            shared::StepResult::Next => {
                ctx.output.success("Configuration updated.");
            }
        }
    }
}

pub fn choose_provider(ctx: &mut Context) -> anyhow::Result<Option<String>> {
    let mut items = Vec::new();
    let mut ids = Vec::new();

    for category in ctx.catalog.categories() {
        for provider in ctx.catalog.providers_by_category(&category) {
            let status =
                if ctx
                    .config
                    .is_provider_configured(&provider.id, &ctx.catalog, &ctx.secrets)
                {
                    " [configured]"
                } else {
                    ""
                };
            items.push(format!(
                "{:<14} {}{}",
                provider.id, provider.description, status
            ));
            ids.push(provider.id.clone());
        }
    }

    let or_status = if ctx.config.is_openrouter_configured(&ctx.secrets) {
        " [configured]"
    } else {
        ""
    };
    items.push(format!("{:<14} 100+ models{}", "openrouter", or_status));
    ids.push("openrouter".to_string());

    let custom_status = if !ctx.config.custom_providers.is_empty() {
        " [configured]"
    } else {
        ""
    };
    items.push(format!(
        "{:<14} Anthropic-compatible endpoint{}",
        "custom", custom_status
    ));
    ids.push("custom".to_string());

    items.push(format!(
        "{:<14} General settings (compaction, etc.)",
        "settings"
    ));
    ids.push("settings".to_string());

    items.push(format!("{:<14} Exit wizard", "done"));
    ids.push("done".to_string());

    let selection =
        ctx.prompt
            .select_opt("Choose provider to configure (Esc to exit)", &items, 0)?;

    if let Some(idx) = selection {
        Ok(Some(ids[idx].clone()))
    } else {
        Ok(None)
    }
}
