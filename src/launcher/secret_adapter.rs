use crate::config::vault::SecretVault;
use crate::domain::launch_blueprint::LaunchTarget;
use crate::ports::launch_ports::SecretGateway;

pub struct AuthEnvAdapter<'a> {
    pub secrets: &'a SecretVault,
}

impl<'a> SecretGateway for AuthEnvAdapter<'a> {
    fn build_provider_env(&self, target: &LaunchTarget) -> anyhow::Result<Vec<String>> {
        crate::launcher::envkit::build_auth_environment(target, self.secrets)
    }
}
