use std::collections::HashMap;

use crate::domain::context::Context;
use crate::providers;

use super::shared::{StepResult, maybe_config_model_limits, persist_config, select_model};

pub(crate) fn config_builtin(
    ctx: &mut Context,
    provider: &providers::index::ServiceDescriptor,
) -> anyhow::Result<StepResult> {
    if provider.auth_mode == "secret" {
        let current_secret = ctx
            .secrets
            .get(&provider.key_var)
            .cloned()
            .or_else(|| std::env::var(&provider.key_var).ok())
            .unwrap_or_default();
        let current_secret = current_secret.trim().to_string();

        let label = if current_secret.is_empty() {
            format!("Enter {} API key", provider.id)
        } else {
            ctx.output.info(&format!(
                "Current key: {}",
                crate::config::vault::redact(&current_secret)
            ));
            format!(
                "Enter {} API key (empty to keep, '!' to unset)",
                provider.id
            )
        };

        match ctx.prompt.prompt_secret_opt(&label)? {
            Some(value) => {
                let value = value.trim();
                if value == "!" {
                    ctx.secrets.remove(&provider.key_var);
                } else if !value.is_empty() {
                    ctx.secrets
                        .insert(provider.key_var.clone(), value.to_string());
                }
            }
            None => return Ok(StepResult::Back),
        }
    }

    let mut dynamic_choices = Vec::new();
    let is_local =
        provider.id == "ollama" || provider.id == "lmstudio" || provider.id == "llamacpp";

    if is_local {
        let models = if provider.id == "ollama" {
            providers::models::fetch_ollama_models(&provider.base_url).ok()
        } else {
            providers::models::fetch_openai_compatible_models(&provider.base_url).ok()
        };

        if let Some(m) = models {
            dynamic_choices = m
                .into_iter()
                .map(|id| providers::index::ModelDescriptor {
                    id: id.clone(),
                    description: format!("Local model: {}", id),
                })
                .collect();
        } else {
            ctx.output.warn(&format!(
                "⚠️  Could not connect to {} at {}. Make sure the server is running.",
                provider.id, provider.base_url
            ));
        }
    }

    let choices = if !dynamic_choices.is_empty() {
        &dynamic_choices
    } else {
        &provider.model_choices
    };

    if !provider.default_model.is_empty() || !choices.is_empty() || is_local {
        let mut override_cfg = ctx
            .config
            .provider_overrides
            .get(&provider.id)
            .cloned()
            .unwrap_or_default();

        let label = format!("Choose default model for {}", provider.id);
        match select_model(ctx, &label, choices, &override_cfg.model, true)? {
            Some(answer) => {
                override_cfg.model = answer.clone();
                if !answer.is_empty() {
                    maybe_config_model_limits(ctx, &answer)?;
                }
            }
            None => return Ok(StepResult::Back),
        }

        ctx.output.info(&format!(
            "\nMap Claude tiers to specific models for {}",
            provider.id
        ));

        let tier_descriptions = [
            ("opus", "most capable, heavy tasks"),
            ("sonnet", "balanced capability & speed"),
            ("haiku", "fast, lightweight tasks"),
        ];

        let mut tiers: HashMap<String, String> = override_cfg.model_tiers.clone();

        for (tier, desc) in tier_descriptions {
            let current_tier_val = tiers.get(tier).cloned().unwrap_or_default();
            let label = format!("{} ({})", tier, desc);
            match select_model(ctx, &label, choices, &current_tier_val, false)? {
                Some(answer) => {
                    if !answer.is_empty() {
                        tiers.insert(tier.to_string(), answer.clone());
                        maybe_config_model_limits(ctx, &answer)?;
                    } else {
                        tiers.remove(tier);
                    }
                }
                None => return Ok(StepResult::Back),
            }
        }
        override_cfg.model_tiers = tiers;

        if !override_cfg.model.is_empty() || !override_cfg.model_tiers.is_empty() {
            ctx.config
                .provider_overrides
                .insert(provider.id.clone(), override_cfg);
        } else {
            ctx.config.provider_overrides.remove(&provider.id);
        }
    }

    persist_config(ctx)?;
    Ok(StepResult::Next)
}
