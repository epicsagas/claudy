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
    if !target.secret_key.is_empty() {
        let status = match ctx.secrets.get(&target.secret_key) {
            Some(v) if !v.is_empty() => "configured",
            _ => "not configured",
        };
        ctx.output
            .info(&format!("Credential:  {} ({})", target.secret_key, status));
    }
    Ok(0)
}
