use super::install;
use crate::domain::context::Context;
use anyhow::Context as _;

pub fn run_update(ctx: &mut Context) -> anyhow::Result<i32> {
    if crate::launcher::envkit::is_homebrew() {
        let brew = which::which("brew").context("brew not found in PATH")?;
        let status = std::process::Command::new(brew)
            .args(["upgrade", "claudy"])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()?;
        return Ok(status.code().unwrap_or(1));
    }
    install::run_install(ctx)
}
