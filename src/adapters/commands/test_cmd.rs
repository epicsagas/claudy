use std::time::Duration;
use ureq::config::Config;

use crate::domain::context::Context;

pub fn run_test(ctx: &mut Context, args: &[String]) -> anyhow::Result<i32> {
    let targets = if !args.is_empty() {
        let target = crate::routing::resolver::route_profile(&args[0], &ctx.catalog, &ctx.config)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        vec![target]
    } else {
        crate::routing::resolver::all_launch_targets(&ctx.catalog, &ctx.config)
    };

    let config = Config::builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let agent = ureq::Agent::new_with_config(config);

    let mut ok_count = 0;
    let mut fail_count = 0;

    for target in &targets {
        if target.profile == "native" {
            continue;
        }
        match agent.get(&target.test_url).call() {
            Ok(resp) => {
                let status = resp.status();
                ctx.output.write_line(&format!(
                    "  {:<18} reachable (HTTP {})",
                    target.profile, status
                ))?;
                ok_count += 1;
            }
            Err(_) => {
                ctx.output
                    .write_line(&format!("  {:<18} unreachable", target.profile))?;
                fail_count += 1;
            }
        }
    }
    ctx.output.write_line(&format!(
        "\nResults: {} reachable, {} failed",
        ok_count, fail_count
    ))?;
    Ok(0)
}
