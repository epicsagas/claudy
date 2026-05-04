use crate::config;
use crate::domain::context::Context;
use crate::domain::launch_blueprint::LaunchTarget;

pub fn run_list(ctx: &mut Context) -> anyhow::Result<i32> {
    let targets = crate::routing::resolver::all_launch_targets(&ctx.catalog, &ctx.config);
    ctx.output
        .header(&format!("Available Profiles ({})", targets.len()));
    for target in &targets {
        let status = if configured(target, &ctx.secrets) {
            "configured"
        } else {
            "not configured"
        };
        ctx.output
            .write_line(&format!("  {:<18} {}", target.profile, status))?;
    }
    if !targets.is_empty() {
        ctx.output.write_line("")?;
        ctx.output.write_line("Run: claudy <name>")?;
    }
    Ok(0)
}

fn configured(target: &LaunchTarget, secrets: &config::vault::SecretVault) -> bool {
    match target.auth_mode.as_str() {
        "none" | "literal" => true,
        "secret" => {
            if let Some(value) = secrets.get(&target.secret_key) {
                !value.trim().is_empty()
            } else {
                false
            }
        }
        _ => false,
    }
}
