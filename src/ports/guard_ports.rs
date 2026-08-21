//! Guard (DLP) boundary contract. The MVP implementation lives in
//! `crate::guard` (regex scan + image strip); llm-kernel's safety engine
//! will replace it behind these same traits without touching the proxy.
//!
//! Sync traits, matching the launch/config port conventions — scanning is
//! pure CPU work with no I/O.

/// Decision applied to a finding before the request egresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardAction {
    /// Forward untouched.
    Allow,
    /// Rewrite the offending span/block, then forward.
    Redact,
    /// Forward untouched, record in the ledger.
    Warn,
    /// Advisory only: suggests switching to a trusted provider. Never
    /// returned by the MVP engine (mid-session provider swap is
    /// impossible); exists for the phase-2 contract.
    ReRoute,
    /// Refuse the request (HTTP 400), never contact upstream.
    Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// Machine-readable category, e.g. "image", "api_key", "non_json".
    pub kind: String,
    pub action: GuardAction,
    /// Redacted preview of the match — never contains raw secret material.
    pub preview: String,
}

#[derive(Debug, Clone, Default)]
pub struct ScanReport {
    pub findings: Vec<Finding>,
    /// `Some` when the body was rewritten (forward these bytes);
    /// `None` when the original bytes must be forwarded byte-identical.
    pub redacted_body: Option<Vec<u8>>,
    pub images_stripped: usize,
}

pub trait ContentScanner {
    fn scan(&self, body: &[u8], content_type: &str) -> ScanReport;
}

pub trait GuardPolicy {
    fn action_for(&self, provider_id: &str, finding_kind: &str) -> GuardAction;
    fn is_trusted(&self, provider_id: &str) -> bool;
}
