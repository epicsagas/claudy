mod interactive;
mod lifecycle;
mod status;

use crate::domain::commands::ChannelAction;
use crate::domain::context::Context;

pub fn run_channel(
    ctx: &mut Context,
    action: ChannelAction,
    profile: Option<&str>,
    listen: Option<&str>,
) -> anyhow::Result<i32> {
    match action {
        ChannelAction::Serve => lifecycle::run_serve(ctx, profile, listen),
        ChannelAction::Start => lifecycle::run_start(ctx, profile, listen),
        ChannelAction::Stop => lifecycle::run_stop(ctx),
        ChannelAction::Restart => lifecycle::run_restart(ctx, profile, listen),
        ChannelAction::Status => status::run_status(ctx),
        ChannelAction::Add { platform } => interactive::run_add(ctx, &platform),
        ChannelAction::Remove { platform } => interactive::run_remove(ctx, &platform),
        ChannelAction::Enable => lifecycle::run_enable(ctx, profile, listen),
        ChannelAction::Disable => lifecycle::run_disable(ctx),
    }
}
