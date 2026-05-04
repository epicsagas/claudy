use crate::domain::context::{Context, DomainCommand};
use crate::ports::command_ports::CommandGateway;

pub fn dispatch_command(
    gateway: &dyn CommandGateway,
    ctx: &mut Context,
    command: DomainCommand,
) -> anyhow::Result<i32> {
    gateway.dispatch(ctx, command)
}
