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
    let mut env = crate::launcher::envkit::build_auth_environment(&target, secrets)?;

    // Inject compaction env vars (max_context_tokens / compaction_threshold) so
    // channel/headless sessions honor `model_settings`, mirroring the
    // interactive launch path. Without this the overlay pipeline is bypassed
    // and Claude Code falls back to its default compaction behavior.
    let effective_model = session.model.unwrap_or(&target.model);
    let overlay = crate::launcher::overlay::materialize_overlay(effective_model, config);
    for (key, value) in overlay.env_overrides {
        env.push(format!("{key}={value}"));
    }

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
        // Resolve symlinks so a configured project dir that points through a
        // symlink (common for dotfile/vault-managed layouts) lands at its real
        // target. `dunce::canonicalize` mirrors `std::fs::canonicalize` on
        // Unix and strips the `\\?\` prefix std adds on Windows. Fall back to
        // the raw path if canonicalization fails so we never harden the
        // existing `is_dir()` guard into a new error path (issue #40).
        let resolved = resolve_working_dir(path);
        cmd.current_dir(&resolved);
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

/// Resolve a project working directory, following symlinks to the real target.
///
/// `dunce::canonicalize` mirrors `std::fs::canonicalize` on Unix (resolves all
/// symlinks, collapses `..`) and additionally strips the `\\?\` verbatim prefix
/// that `std` prepends on Windows. If canonicalization fails (e.g. a dangling
/// symlink component) the raw path is returned unchanged so this never turns a
/// passing `is_dir()` guard into a new error path. See issue #40.
pub fn resolve_working_dir(path: &std::path::Path) -> std::path::PathBuf {
    dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::resolve_working_dir;

    /// A symlinked project directory must resolve to its real target (issue #40).
    #[test]
    fn resolve_working_dir_follows_symlink() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let real = tmp.path().join("real_project");
        std::fs::create_dir(&real).expect("mkdir real");
        std::fs::write(real.join("marker"), b"x").expect("write marker");

        let link = tmp.path().join("link_project");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real, &link).expect("symlink");
        // Symlink support is Unix-only in claudy; skip elsewhere.
        #[cfg(not(unix))]
        return;

        let resolved = resolve_working_dir(&link);
        assert_ne!(resolved, link, "symlink was not resolved");
        assert_eq!(resolved, real, "resolved path should point at the real dir");
        assert!(resolved.join("marker").exists());
    }

    /// Canonicalization failure must fall back to the raw path, not error.
    #[test]
    fn resolve_working_dir_falls_back_on_failure() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dangling = tmp.path().join("does_not_exist");
        // Raw path returned verbatim — no panic, no error.
        assert_eq!(resolve_working_dir(&dangling), dangling);
    }
}
