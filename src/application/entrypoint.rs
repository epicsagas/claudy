use crate::application::launch_orchestrator::LaunchOrchestrator;
use crate::domain::launch_blueprint::LaunchBlueprint;
use crate::launcher::adapter::LauncherAdapter;
use crate::launcher::secret_adapter::AuthEnvAdapter;
use crate::routing::adapter::RoutingAdapter;

pub fn launch_profile_session(
    paths: &crate::config::layout::AppPaths,
    catalog: &crate::providers::index::ProviderIndex,
    config: &crate::config::registry::AppRegistry,
    secrets: &crate::config::vault::SecretVault,
    profile: &str,
    args: &[String],
) -> anyhow::Result<i32> {
    let (hide_banner, mut forwarded_args) = parse_entry_args(args);
    let resolved_profile = resolve_profile_alias(profile, &forwarded_args)?;
    if profile == "or" {
        forwarded_args = forwarded_args[1..].to_vec();
    }

    let (mode, forwarded_args) = extract_mode(paths, &forwarded_args);

    // Upsert global MCP registration
    crate::adapters::mcp::server::ensure_registered_global();

    // Upsert MCP registration for the active mode only
    if let Some(ref mode_name) = mode {
        crate::adapters::mcp::server::ensure_registered_mode(&paths.modes_dir, mode_name);
    }

    let orchestrator = LaunchOrchestrator::new(
        RoutingAdapter { catalog, config },
        AuthEnvAdapter { secrets },
        LauncherAdapter { paths },
    );
    orchestrator.dispatch(LaunchBlueprint {
        profile: resolved_profile,
        forwarded_args,
        hide_banner,
        mode,
    })
}

fn extract_mode(
    paths: &crate::config::layout::AppPaths,
    args: &[String],
) -> (Option<String>, Vec<String>) {
    for (i, arg) in args.iter().enumerate() {
        if arg.starts_with('-') {
            continue;
        }
        let mode_path = std::path::Path::new(&paths.modes_dir).join(arg);
        if mode_path.is_dir() {
            let remaining = [&args[..i], &args[i + 1..]].concat();
            return (Some(arg.clone()), remaining);
        }
        break;
    }
    (None, args.to_vec())
}

fn parse_entry_args(args: &[String]) -> (bool, Vec<String>) {
    let (_launcher_options, forwarded) = crate::adapters::cli::parse::parse_launcher(args);
    (false, forwarded)
}

fn resolve_profile_alias(profile: &str, args: &[String]) -> anyhow::Result<String> {
    if profile != "or" {
        return Ok(profile.to_string());
    }
    if args.is_empty() || args[0].starts_with('-') {
        anyhow::bail!(
            "usage: claudy or <alias> [args...]\n\nRun `claudy config openrouter` to configure aliases"
        );
    }
    let alias = args[0].trim().to_string();
    if alias.is_empty() || !is_valid_route_alias(&alias) {
        anyhow::bail!("invalid alias {:?}: must match [a-z0-9][a-z0-9-]*", args[0]);
    }
    Ok(format!("or-{}", alias))
}

fn is_valid_route_alias(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_route_valid() {
        assert!(is_valid_route_alias("gpt4"));
        assert!(is_valid_route_alias("my-model-v2"));
    }

    #[test]
    fn test_alias_route_invalid() {
        assert!(!is_valid_route_alias(""));
        assert!(!is_valid_route_alias("UPPER"));
    }
}
