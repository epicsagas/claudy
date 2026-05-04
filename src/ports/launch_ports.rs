use crate::domain::launch_blueprint::LaunchTarget;

/// Three-phase launch model: resolve profile -> build env -> spawn process.
/// Each phase is a separate port so they can be tested and substituted independently.
pub trait ProfileGateway {
    fn resolve_target(&self, profile: &str) -> anyhow::Result<LaunchTarget>;
}

pub trait SecretGateway {
    fn build_provider_env(&self, target: &LaunchTarget) -> anyhow::Result<Vec<String>>;
}

/// Locates the Claude binary on the filesystem.
pub trait BinaryLookupPort {
    fn locate_claude_binary(&self) -> anyhow::Result<std::path::PathBuf>;
}

/// Spawns a Claude process with the given environment.
pub trait ProcessSpawnPort {
    fn spawn_claude(
        &self,
        binary: &std::path::Path,
        args: &[String],
        env: &[String],
    ) -> anyhow::Result<i32>;
}

/// Combined trait for full runtime execution.
pub trait RuntimeGateway: BinaryLookupPort + ProcessSpawnPort {
    fn run_target(
        &self,
        target: &LaunchTarget,
        forwarded_args: &[String],
        env: &[String],
        hide_banner: bool,
        mode: Option<&str>,
    ) -> anyhow::Result<i32>;
}
