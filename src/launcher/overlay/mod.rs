/// Config overlay pipeline: injects provider-specific settings via env vars
/// so Claude Code can reuse its own ~/.claude/ directory (skills, plugins)
/// while overriding model, base URL, and auth per-provider.
///
/// The pipeline is decomposed into typed stages (see `stages.rs`) so each
/// boundary is testable and auditable independently.
pub mod pipeline;
pub mod stages;

pub use pipeline::prepare_provider_env;
