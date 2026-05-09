use clap::Parser;
use std::io::{self, Write};

pub fn run_cli_session(argv0: &str, args: &[String]) -> anyhow::Result<i32> {
    if argv0 == "claude" {
        return run_claude_shim(args);
    }

    let (launcher_profile, is_launcher) =
        crate::routing::resolver::detect_symlink_invocation(argv0);
    if is_launcher {
        return run_launcher(&launcher_profile, args);
    }

    if !args.is_empty() {
        let cmd_or_profile = &args[0];
        if !is_builtin_subcommand(cmd_or_profile) && !cmd_or_profile.starts_with('-') {
            return run_profile_direct(cmd_or_profile, &args[1..]);
        }
    }

    // Warn on deprecated top-level "ls" alias
    if args.first().is_some_and(|a| a == "ls") {
        eprintln!("[deprecated] 'ls' is deprecated, use 'list' instead.");
    }

    let cli = crate::adapters::cli::args::Cli::parse();
    let paths = crate::config::layout::discover()?;
    let catalog = crate::providers::index::load_index()?;
    let secrets = crate::config::vault::load_vault(&paths.secrets_file)?;
    let mut cfg = crate::config::registry::open_registry(&paths.config_file)?;
    cfg.ingest_legacy_secrets(&secrets, &catalog);
    cfg.compact(&catalog);

    let mut output =
        crate::adapters::ui::output::Output::new(crate::adapters::ui::output::Format::Human, false);
    let prompt = crate::adapters::ui::prompt::Prompter::new(io::stdin(), io::stdout());

    let show_update = !is_install_command(&cli.command);
    if let Some(msg) =
        crate::adapters::update::check::maybe_message(&paths, crate::adapters::version::VALUE)
            .ok()
            .flatten()
            .filter(|_| show_update)
    {
        writeln!(output.stderr, "{}", msg)?;
    }

    let mut ctx = crate::domain::context::Context {
        paths,
        config: cfg,
        secrets,
        catalog,
        output: Box::new(output),
        prompt: Box::new(prompt),
        options: crate::domain::context::Options {
            help: false,
            version: false,
        },
    };

    if let Some(cmd) = cli.command {
        let gateway = crate::adapters::commands::LegacyCommandAdapter;
        let domain_cmd = crate::adapters::commands::map_cli_to_domain(cmd);

        crate::application::command_bus::dispatch_command(&gateway, &mut ctx, domain_cmd)
    } else {
        crate::adapters::cli::help::show_brief(ctx.output.as_mut())?;
        Ok(0)
    }
}

fn run_claude_shim(args: &[String]) -> anyhow::Result<i32> {
    let paths = crate::config::layout::discover()?;
    crate::launcher::shim::run_claude_shim(&paths, args)
}

fn run_profile_direct(profile: &str, args: &[String]) -> anyhow::Result<i32> {
    let paths = crate::config::layout::discover()?;
    let catalog = crate::providers::index::load_index()?;
    let secrets = crate::config::vault::load_vault(&paths.secrets_file)?;
    let mut cfg = crate::config::registry::open_registry(&paths.config_file)?;
    cfg.ingest_legacy_secrets(&secrets, &catalog);
    cfg.compact(&catalog);
    crate::application::entrypoint::launch_profile_session(
        &paths, &catalog, &cfg, &secrets, profile, args,
    )
}

fn run_launcher(profile: &str, args: &[String]) -> anyhow::Result<i32> {
    let paths = crate::config::layout::discover()?;
    let catalog = crate::providers::index::load_index()?;
    let secrets = crate::config::vault::load_vault(&paths.secrets_file)?;
    let mut cfg = crate::config::registry::open_registry(&paths.config_file)?;
    cfg.ingest_legacy_secrets(&secrets, &catalog);
    cfg.compact(&catalog);
    crate::application::entrypoint::launch_profile_session(
        &paths, &catalog, &cfg, &secrets, profile, args,
    )
}

fn is_install_command(cmd: &Option<crate::adapters::cli::args::Commands>) -> bool {
    matches!(
        cmd,
        Some(
            crate::adapters::cli::args::Commands::Sync
                | crate::adapters::cli::args::Commands::Uninstall
                | crate::adapters::cli::args::Commands::Update
        )
    )
}

fn is_builtin_subcommand(name: &str) -> bool {
    matches!(
        name,
        "ls" | "list"
            | "setup"
            | "config"
            | "show"
            | "info"
            | "ping"
            | "test"
            | "doctor"
            | "status"
            | "sync"
            | "install"
            | "update"
            | "uninstall"
            | "mode"
            | "channel"
            | "mcp"
            | "analytics"
            | "help"
    )
}
