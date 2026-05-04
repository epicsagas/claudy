use crate::domain::launch_blueprint::LaunchBlueprint;
use crate::ports::launch_ports::{ProfileGateway, RuntimeGateway, SecretGateway};

pub struct LaunchOrchestrator<P, S, R> {
    profile_gateway: P,
    secret_gateway: S,
    runtime_gateway: R,
}

impl<P, S, R> LaunchOrchestrator<P, S, R>
where
    P: ProfileGateway,
    S: SecretGateway,
    R: RuntimeGateway,
{
    pub fn new(profile_gateway: P, secret_gateway: S, runtime_gateway: R) -> Self {
        Self {
            profile_gateway,
            secret_gateway,
            runtime_gateway,
        }
    }

    pub fn dispatch(self, blueprint: LaunchBlueprint) -> anyhow::Result<i32> {
        let target = self.profile_gateway.resolve_target(&blueprint.profile)?;
        let env = self.secret_gateway.build_provider_env(&target)?;
        self.runtime_gateway.run_target(
            &target,
            &blueprint.forwarded_args,
            &env,
            blueprint.hide_banner,
            blueprint.mode.as_deref(),
        )
    }
}
