use crate::domain::commands::DomainCommand;
use crate::domain::context::Context;

/// Registry-based command dispatch. Each command is registered once and
/// resolved through the registry, reducing match-coupling in callers.
pub struct CommandRegistry;

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn dispatch(&self, ctx: &mut Context, command: DomainCommand) -> anyhow::Result<i32> {
        match command {
            DomainCommand::List => super::list::run_list(ctx),
            DomainCommand::Setup { provider } => {
                let args = provider.map(|p| vec![p]).unwrap_or_default();
                super::config_cmd::run_config(ctx, &args)
            }
            DomainCommand::Show { profile } => super::info::run_info(ctx, &[profile]),
            DomainCommand::Ping { profile } => {
                let args = profile.map(|p| vec![p]).unwrap_or_default();
                super::test_cmd::run_test(ctx, &args)
            }
            DomainCommand::Doctor => super::status::run_status(ctx),
            DomainCommand::Sync => super::install::run_install(ctx),
            DomainCommand::Update => super::update_cmd::run_update(ctx),
            DomainCommand::Uninstall => super::uninstall::run_uninstall(ctx),
            DomainCommand::Mode { action, name } => {
                super::mode_cmd::run_mode(ctx, &action, name.as_deref())
            }
            DomainCommand::Channel {
                action,
                profile,
                listen,
            } => {
                super::channel_cmd::run_channel(ctx, action, profile.as_deref(), listen.as_deref())
            }
            DomainCommand::Mcp(action) => match action {
                crate::domain::commands::McpAction::Run => super::mcp_cmd::run_mcp(ctx),
                crate::domain::commands::McpAction::Install => super::mcp_cmd::install_mcp(ctx),
                crate::domain::commands::McpAction::Uninstall => super::mcp_cmd::uninstall_mcp(ctx),
            },
            #[cfg(feature = "analytics")]
            DomainCommand::Analytics(action) => super::analytics_cmd::run_analytics(ctx, action),
            #[cfg(not(feature = "analytics"))]
            DomainCommand::Analytics(_) => {
                ctx.output
                    .warn("Analytics requires --features analytics build");
                Ok(1)
            }
            DomainCommand::Session(action) => match action {
                crate::domain::commands::SessionAction::Sanitize { project, all, yes } => {
                    super::session_cmd::run_session_sanitize(ctx, project.as_deref(), all, yes)
                }
            },
        }
    }
}

pub fn dispatch_new(ctx: &mut Context, command: DomainCommand) -> anyhow::Result<i32> {
    CommandRegistry::new().dispatch(ctx, command)
}
