use std::process::Stdio;

use anyhow::Context as _;

use crate::config::layout::AppPaths;
use crate::config::registry::AppRegistry;
use crate::config::vault::SecretVault;
use crate::providers::index::ProviderIndex;

pub struct ClaudeProcess {
    child: tokio::process::Child,
    stdin: Option<tokio::process::ChildStdin>,
    stdout: tokio::process::ChildStdout,
    stderr: Option<tokio::process::ChildStderr>,
}

impl ClaudeProcess {
    pub fn child_id(&self) -> Option<u32> {
        self.child.id()
    }

    pub fn take_stdin(&mut self) -> Option<tokio::process::ChildStdin> {
        self.stdin.take()
    }

    pub fn stdout(&mut self) -> &mut tokio::process::ChildStdout {
        &mut self.stdout
    }

    pub fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.stderr.take()
    }

    pub async fn wait(&mut self) -> anyhow::Result<i32> {
        let status = self.child.wait().await?;
        Ok(status.code().unwrap_or(1))
    }

    pub async fn kill(&mut self) -> anyhow::Result<()> {
        Ok(self.child.kill().await?)
    }
}

/// Session-specific parameters for launching a Claude process.
pub struct SessionConfig<'a> {
    pub profile: &'a str,
    pub mode: Option<&'a str>,
    pub resume_session: Option<&'a str>,
    pub working_dir: Option<&'a str>,
    pub model: Option<&'a str>,
    pub yolo: bool,
}

pub fn start_claude_session(
    paths: &AppPaths,
    config: &AppRegistry,
    secrets: &SecretVault,
    catalog: &ProviderIndex,
    session: &SessionConfig<'_>,
) -> anyhow::Result<ClaudeProcess> {
    let target = crate::routing::resolver::route_profile(session.profile, catalog, config)
        .context(format!("Failed to resolve profile '{}'", session.profile))?;
    let env = crate::launcher::envkit::build_auth_environment(&target, secrets)?;
    let claude_bin = crate::launcher::binary::find_claude_cli()?;

    let mut cmd = tokio::process::Command::new(claude_bin);
    cmd.args(["--output-format", "stream-json", "--print", "--verbose"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if session.yolo {
        cmd.arg("--dangerously-skip-permissions");
    }

    if let Some(session_id) = session.resume_session {
        cmd.arg("--resume").arg(session_id);
    }

    if let Some(dir) = session.working_dir {
        let path = std::path::Path::new(dir);
        anyhow::ensure!(
            path.is_absolute(),
            "working_dir must be an absolute path: {dir}"
        );
        anyhow::ensure!(
            path.is_dir(),
            "working_dir does not exist or is not a directory: {dir}"
        );
        cmd.current_dir(path);
    }

    if let Some(model_name) = session.model {
        cmd.arg("--model").arg(model_name);
    }

    for env_str in &env {
        if let Some((key, value)) = env_str.split_once('=') {
            cmd.env(key, value);
        }
    }

    if let Some(mode_name) = session.mode {
        let mode_dir = std::path::Path::new(&paths.modes_dir).join(mode_name);
        cmd.env("CLAUDE_CONFIG_DIR", mode_dir);
    } else {
        cmd.env_remove("CLAUDE_CONFIG_DIR");
    }
    cmd.env_remove("CLAUDECODE");
    cmd.env_remove("CLAUDE_CODE_ENTRYPOINT");
    cmd.env_remove("CLAUDE_CODE_OAUTH_TOKEN");
    cmd.env_remove("CLAUDE_CODE_EXECPATH");

    let mut child = cmd.spawn().context("Failed to spawn Claude process")?;
    let stdin = child.stdin.take();
    let stdout = child.stdout.take().context("No stdout on Claude process")?;
    let stderr = child.stderr.take();

    Ok(ClaudeProcess {
        child,
        stdin,
        stdout,
        stderr,
    })
}
