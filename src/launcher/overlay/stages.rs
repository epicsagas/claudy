use std::collections::HashMap;

/// Where the session model was resolved from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelSource {
    /// Model specified via --model CLI flag.
    CliOverride(String),
    /// Model from ANTHROPIC_* environment variable.
    EnvVar(String),
    /// Model resolved from a provider tier (opus/sonnet/haiku/small).
    TierFallback { tier: String, model: String },
    /// Model from the target's default_model field.
    TargetDefault(String),
}

/// Stage 1 output: resolved model for this session.
pub struct ModelResolution {
    pub session_model: String,
    pub source: ModelSource,
}

/// Stage 2 output: optional ANTHROPIC_CONFIG_OVERRIDE JSON blob.
pub struct OverlayMaterialization {
    pub config_override_json: Option<String>,
}

/// Stage 3 output: final env map with all patches applied.
pub struct SettingsPatch {
    pub env_map: HashMap<String, String>,
}

/// Stage 4: cleanup handle (reserved for temp-dir cleanup).
pub struct CleanupHandle {
    _reserved: (),
}

impl CleanupHandle {
    pub fn noop() -> Self {
        Self { _reserved: () }
    }
}
