use crate::domain::context::Context;

use super::shared::{StepResult, persist_config};

pub(crate) fn config_general_settings(ctx: &mut Context) -> anyhow::Result<StepResult> {
    ctx.output.header("General Claudy Settings");

    let auto_compact = match ctx.prompt.confirm_opt(
        "Enable automatic history compaction?",
        ctx.config.compaction.auto_compact,
    )? {
        Some(val) => val,
        None => return Ok(StepResult::Back),
    };
    ctx.config.compaction.auto_compact = auto_compact;

    if ctx.config.compaction.auto_compact {
        let threshold_str = match ctx.prompt.prompt_opt(
            "Compaction threshold (0.0-1.0)",
            &ctx.config.compaction.threshold.to_string(),
        )? {
            Some(val) => val,
            None => return Ok(StepResult::Back),
        };
        if let Ok(val) = threshold_str.parse::<f64>() {
            ctx.config.compaction.threshold = val.clamp(0.0, 1.0);
        }
    }

    persist_config(ctx)?;
    Ok(StepResult::Next)
}
