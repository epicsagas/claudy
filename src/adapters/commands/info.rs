use crate::domain::context::Context;

pub fn run_info(ctx: &mut Context, args: &[String]) -> anyhow::Result<i32> {
    if args.is_empty() {
        anyhow::bail!("usage: claudy info <provider>");
    }
    let target = crate::routing::resolver::route_profile(&args[0], &ctx.catalog, &ctx.config)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    ctx.output.header("Provider Info");
    ctx.output.info(&format!("Profile:     {}", target.profile));
    ctx.output
        .info(&format!("Name:        {}", target.display_name));
    ctx.output.info(&format!("Family:      {}", target.family));
    ctx.output
        .info(&format!("Base URL:    {}", target.base_url));
    if !target.model.is_empty() {
        ctx.output.info(&format!("Model:       {}", target.model));
    }
    for tier in ["haiku", "sonnet", "opus"] {
        if let Some(m) = target.model_tiers.get(tier) {
            ctx.output.info(&format!("  {:<8}   {}", tier, m));
        }
    }
    if !target.secret_key.is_empty() {
        let configured = ctx
            .secrets
            .get(&target.secret_key)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
            || std::env::var(&target.secret_key)
                .map(|v| !v.is_empty())
                .unwrap_or(false);
        let status = if configured {
            "configured"
        } else {
            "not configured"
        };
        ctx.output
            .info(&format!("Credential:  {} ({})", target.secret_key, status));
    }
    Ok(0)
}
