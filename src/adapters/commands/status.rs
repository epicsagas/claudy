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
    Ok(0)
}
