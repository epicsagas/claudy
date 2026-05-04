/// Observable states of the update check lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateState {
    /// No check has been initiated.
    Idle,
    /// Cache is stale, a remote refresh is needed.
    RefreshNeeded,
    /// Remote version has been fetched.
    Refreshed,
    /// Current and latest versions have been compared.
    Compared,
    /// User has been notified about an available update.
    Notified,
    /// Check cycle is complete.
    Complete,
}

/// Reason an update check was skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// Running a development build (version == "dev").
    DevBuild,
    /// Version string is empty.
    EmptyVersion,
}

/// Events emitted during update checking.
#[derive(Debug, Clone)]
pub enum UpdateEvent {
    /// Check was skipped for a known reason.
    Skipped(SkipReason),
    /// Check completed; indicates whether an update exists.
    Checked { has_update: bool },
    /// Update available with current and latest version strings.
    UpdateAvailable { current: String, latest: String },
    /// No update needed.
    NoUpdate,
}

/// Structured result of an update check cycle.
#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub state: UpdateState,
    pub message: Option<String>,
    pub event: UpdateEvent,
}
