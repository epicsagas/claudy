use crate::domain::context::{Context, DomainCommand};

pub trait CommandGateway {
    fn dispatch(&self, ctx: &mut Context, command: DomainCommand) -> anyhow::Result<i32>;
}
