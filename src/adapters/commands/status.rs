use crate::adapters::version;
use crate::domain::context::Context;

pub fn run_status(ctx: &mut Context) -> anyhow::Result<i32> {
    let all_targets = crate::routing::resolver::all_launch_targets(&ctx.catalog, &ctx.config);

    let mut configured_count = 0;
    for target in &all_targets {
        if target.category == "openrouter"
            || target.category == "custom"
            || ctx
                .config
                .is_provider_configured(&target.profile, &ctx.catalog, &ctx.secrets)
        {
            configured_count += 1;
        }
    }

    let claudy_bin = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let claude_config_dir = std::env::var("CLAUDE_CONFIG_DIR").unwrap_or_else(|_| {
        dirs::home_dir()
            .map(|h| h.join(".claude").to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    });

    ctx.output.header("Claudy Status");
    ctx.output
        .write_line(&format!("Version:   {}", version::VALUE))?;
    ctx.output
        .write_line(&format!("Home:      {}", ctx.paths.claudy_home))?;
    ctx.output
        .write_line(&format!("Claude:    {}", claude_config_dir))?;
    ctx.output
        .write_line(&format!("Bin:       {}", claudy_bin))?;
    ctx.output
        .write_line(&format!("Profiles:  {}", configured_count))?;

    // Warn when an external CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE caps the context
    // window below a configured model's max_context_tokens — the classic cause
    // of a 1M model compacting prematurely (e.g. at 180k).
    if let Ok(raw) = std::env::var("CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE")
        && let Ok(blocking) = raw.trim().parse::<u64>()
    {
        let capped: Vec<&str> = ctx
            .config
            .model_settings
            .iter()
            .filter_map(|(model, s)| {
                s.max_context_tokens
                    .filter(|&max| (max as u64) > blocking)
                    .map(|_| model.as_str())
            })
            .collect();
        if !capped.is_empty() {
            ctx.output.warn(&format!(
                "CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE={} caps the context window below max_context_tokens for: {}. \
                 Remove this env var (from ~/.claude/settings.json or shell) to use the full window.",
                blocking,
                capped.join(", ")
            ));
        }
    }

    Ok(0)
}
