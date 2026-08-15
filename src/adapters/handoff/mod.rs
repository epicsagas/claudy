//! Foreign CLI session readers for `claudy handoff`.
//!
//! Each reader exposes a cheap listing pass (metadata only) and a full event
//! extraction pass that runs only for the session the user selected. Store
//! layouts are undocumented third-party formats — every reader degrades to an
//! empty result instead of failing the whole command.

pub mod agy;
pub mod codex;

use std::path::{Path, PathBuf};

use crate::domain::handoff::ForeignSessionSummary;

/// Cap on bytes read from a single foreign session file (~2 MB observed max).
pub const MAX_SESSION_BYTES: u64 = 8 * 1024 * 1024;

pub fn codex_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codex"))
}

pub fn agy_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".gemini").join("antigravity-cli"))
}

/// True when the summary's workspace matches `target` (both canonicalized).
/// Tolerates missing metadata (matches nothing).
pub fn cwd_matches(summary: &ForeignSessionSummary, target: &Path) -> bool {
    let Some(cwd) = &summary.cwd else {
        return false;
    };
    let Some(cwd_path) = canonicalish(Path::new(cwd)) else {
        return false;
    };
    let Some(target_path) = canonicalish(target) else {
        return false;
    };
    cwd_path == target_path
}

/// Canonicalize without failing on nonexistent paths (compare what we can).
fn canonicalish(path: &Path) -> Option<PathBuf> {
    if let Ok(p) = dunce::canonicalize(path) {
        return Some(p);
    }
    let parent = dunce::canonicalize(path.parent()?).ok()?;
    Some(parent.join(path.file_name()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::handoff::HandoffSource;

    #[test]
    fn cwd_matches_canonical() {
        let s = ForeignSessionSummary {
            source: HandoffSource::Codex,
            id: "x".into(),
            title: None,
            cwd: Some("/tmp".into()),
            last_modified: 0,
            path: None,
        };
        assert!(cwd_matches(&s, Path::new("/private/tmp")));
        assert!(!cwd_matches(&s, Path::new("/var")));
    }

    #[test]
    fn cwd_missing_never_matches() {
        let s = ForeignSessionSummary {
            source: HandoffSource::Agy,
            id: "x".into(),
            title: None,
            cwd: None,
            last_modified: 0,
            path: None,
        };
        assert!(!cwd_matches(&s, Path::new("/tmp")));
    }
}
