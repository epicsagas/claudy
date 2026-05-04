use crate::config::layout::AppPaths;
use crate::domain::launch_blueprint::LaunchTarget;
use crate::ports::launch_ports::{BinaryLookupPort, ProcessSpawnPort, RuntimeGateway};

pub struct LauncherAdapter<'a> {
    pub paths: &'a AppPaths,
}

impl<'a> BinaryLookupPort for LauncherAdapter<'a> {
    fn locate_claude_binary(&self) -> anyhow::Result<std::path::PathBuf> {
        crate::launcher::binary::find_claude_cli()
    }
}

impl<'a> ProcessSpawnPort for LauncherAdapter<'a> {
    fn spawn_claude(
        &self,
        binary: &std::path::Path,
        args: &[String],
        env: &[String],
    ) -> anyhow::Result<i32> {
        crate::launcher::binary::exec_claude_session(binary, args, env)
    }
}

impl<'a> RuntimeGateway for LauncherAdapter<'a> {
    fn run_target(
        &self,
        target: &LaunchTarget,
        forwarded_args: &[String],
        env: &[String],
        hide_banner: bool,
        mode: Option<&str>,
    ) -> anyhow::Result<i32> {
        crate::launcher::binary::run_session(
            self.paths,
            target,
            forwarded_args,
            env,
            crate::launcher::binary::SessionOptions {
                suppress_banner: hide_banner,
            },
            mode,
        )
    }
}
