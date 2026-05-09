#[cfg(feature = "analytics")]
pub mod analytics_cmd;
pub mod channel_cmd;
pub mod config_cmd;
pub mod dispatch;
pub mod info;
pub mod install;
pub mod list;
pub mod mcp_cmd;
pub mod mode_cmd;
pub mod status;
pub mod test_cmd;
pub mod uninstall;
pub mod update_cmd;

use crate::adapters::cli::args::Commands;
use crate::domain::commands::{AnalyticsAction, ChannelAction, DomainCommand, McpAction};
use crate::domain::context::Context;

pub struct LegacyCommandAdapter;

impl crate::ports::command_ports::CommandGateway for LegacyCommandAdapter {
    fn dispatch(&self, ctx: &mut Context, command: DomainCommand) -> anyhow::Result<i32> {
        crate::adapters::commands::dispatch::dispatch_new(ctx, command)
    }
}

pub fn map_cli_to_domain(command: Commands) -> DomainCommand {
    match command {
        Commands::List => DomainCommand::List,
        Commands::Setup { provider } => DomainCommand::Setup { provider },
        Commands::Show { profile } => DomainCommand::Show { profile },
        Commands::Ping { profile } => DomainCommand::Ping { profile },
        Commands::Doctor => DomainCommand::Doctor,
        Commands::Sync => DomainCommand::Sync,
        Commands::Update => DomainCommand::Update,
        Commands::Uninstall => DomainCommand::Uninstall,
        Commands::Mode { action, name } => DomainCommand::Mode { action, name },
        Commands::Channel(sub) => match sub {
            crate::adapters::cli::args::ChannelCommands::Serve { profile, listen } => {
                DomainCommand::Channel {
                    action: ChannelAction::Serve,
                    profile,
                    listen,
                }
            }
            crate::adapters::cli::args::ChannelCommands::Start { profile, listen } => {
                DomainCommand::Channel {
                    action: ChannelAction::Start,
                    profile,
                    listen,
                }
            }
            crate::adapters::cli::args::ChannelCommands::Stop => DomainCommand::Channel {
                action: ChannelAction::Stop,
                profile: None,
                listen: None,
            },
            crate::adapters::cli::args::ChannelCommands::Restart { profile, listen } => {
                DomainCommand::Channel {
                    action: ChannelAction::Restart,
                    profile,
                    listen,
                }
            }
            crate::adapters::cli::args::ChannelCommands::Status => DomainCommand::Channel {
                action: ChannelAction::Status,
                profile: None,
                listen: None,
            },
            crate::adapters::cli::args::ChannelCommands::Add { platform } => {
                DomainCommand::Channel {
                    action: ChannelAction::Add { platform },
                    profile: None,
                    listen: None,
                }
            }
            crate::adapters::cli::args::ChannelCommands::Remove { platform } => {
                DomainCommand::Channel {
                    action: ChannelAction::Remove { platform },
                    profile: None,
                    listen: None,
                }
            }
            crate::adapters::cli::args::ChannelCommands::Enable => DomainCommand::Channel {
                action: ChannelAction::Enable,
                profile: None,
                listen: None,
            },
            crate::adapters::cli::args::ChannelCommands::Disable => DomainCommand::Channel {
                action: ChannelAction::Disable,
                profile: None,
                listen: None,
            },
        },
        Commands::Mcp(sub) => DomainCommand::Mcp(match sub {
            crate::adapters::cli::args::McpCommands::Run => McpAction::Run,
            crate::adapters::cli::args::McpCommands::Install => McpAction::Install,
            crate::adapters::cli::args::McpCommands::Uninstall => McpAction::Uninstall,
        }),
        Commands::Analytics(sub) => match sub {
            crate::adapters::cli::args::AnalyticsCommands::Dashboard => {
                DomainCommand::Analytics(AnalyticsAction::Dashboard)
            }
            crate::adapters::cli::args::AnalyticsCommands::Ingest { full, project } => {
                DomainCommand::Analytics(AnalyticsAction::Ingest { full, project })
            }
            crate::adapters::cli::args::AnalyticsCommands::Recommend => {
                DomainCommand::Analytics(AnalyticsAction::Recommend)
            }
            crate::adapters::cli::args::AnalyticsCommands::Export {
                format,
                project,
                days,
            } => DomainCommand::Analytics(AnalyticsAction::Export {
                format,
                project,
                days,
            }),
            crate::adapters::cli::args::AnalyticsCommands::SyncPricing => {
                DomainCommand::Analytics(AnalyticsAction::SyncPricing)
            }
            crate::adapters::cli::args::AnalyticsCommands::Insights {
                days,
                from,
                to,
                project,
            } => DomainCommand::Analytics(AnalyticsAction::Insights {
                days,
                from,
                to,
                project,
            }),
            crate::adapters::cli::args::AnalyticsCommands::Recalculate => {
                DomainCommand::Analytics(AnalyticsAction::Recalculate)
            }
        },
    }
}
