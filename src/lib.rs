pub mod adapters;
pub mod application;
pub mod config;
pub mod domain;
pub mod launcher;
pub mod ports;
pub mod providers;
pub mod routing;

pub fn run(argv0: &str, args: &[String]) -> anyhow::Result<i32> {
    application::bootstrap::run_cli_session(argv0, args)
}
