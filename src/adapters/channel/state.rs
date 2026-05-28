use std::collections::HashMap;
use std::sync::Arc;

/// Per-context key-value state, namespaced by (platform, channel_id, user_id).
///
/// Each conversation context gets its own isolated key-value store, so
/// multiple platforms, channels, and users can maintain independent sessions.
///
/// Persisted as JSON lines (one JSON object per line):
/// ```json
/// {"scope":"telegram:123:u1","key":"SESSION_ID","value":"abc-123"}
/// ```
///
/// Legacy `SCOPE#KEY=VALUE` and `KEY=VALUE` formats are transparently
/// migrated on read.
pub struct ChannelState {
    scopes: HashMap<String, HashMap<String, String>>,
    path: String,
    dirty: bool,
}

const MAX_SCOPES: usize = 256;

/// Build a scope key from platform, channel, and user identity.
///
/// Colons in `channel_id` or `user_id` are percent-encoded to prevent
/// scope collision attacks.
pub fn scope_key(platform: &str, channel_id: &str, user_id: &str) -> String {
    format!(
        "{}:{}:{}",
        platform,
        encode_component(channel_id),
        encode_component(user_id)
    )
}

fn encode_component(s: &str) -> String {
    if !s.contains(':') && !s.contains('%') {
        return s.to_string();
    }
    s.replace('%', "%25").replace(':', "%3A")
}

impl ChannelState {
    pub fn load(path: &str) -> Self {
        let scopes = Self::read_file(path);
        Self {
            scopes,
            path: path.to_string(),
            dirty: false,
        }
    }

    /// Get a value for the given scope and key.
    pub fn get(&self, scope: &str, key: &str) -> Option<&str> {
        self.scopes
            .get(scope)
            .and_then(|m| m.get(key))
            .map(|s| s.as_str())
    }

    /// Set a value for the given scope and key.
    pub fn set(&mut self, scope: &str, key: &str, value: &str) {
        self.scopes
            .entry(scope.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
        self.dirty = true;

        // Evict scopes if over limit (remove arbitrary entries)
        while self.scopes.len() > MAX_SCOPES {
            if let Some(key) = self.scopes.keys().next().cloned() {
                self.scopes.remove(&key);
            } else {
                break;
            }
        }
    }

    // Typed session state accessors

    pub fn session_id(&self, scope: &str) -> Option<&str> {
        self.get(scope, "SESSION_ID").filter(|s| !s.is_empty())
    }

    pub fn set_session_id(&mut self, scope: &str, val: &str) {
        self.set(scope, "SESSION_ID", val);
    }

    pub fn working_dir(&self, scope: &str) -> Option<&str> {
        self.get(scope, "SESSION_CWD").filter(|s| !s.is_empty())
    }

    pub fn set_working_dir(&mut self, scope: &str, val: &str) {
        self.set(scope, "SESSION_CWD", val);
    }

    pub fn model(&self, scope: &str) -> Option<&str> {
        self.get(scope, "MODEL").filter(|s| !s.is_empty())
    }

    pub fn set_model(&mut self, scope: &str, val: &str) {
        self.set(scope, "MODEL", val);
    }

    pub fn last_model(&self, scope: &str) -> Option<&str> {
        self.get(scope, "LAST_MODEL").filter(|s| !s.is_empty())
    }

    pub fn set_last_model(&mut self, scope: &str, val: &str) {
        self.set(scope, "LAST_MODEL", val);
    }

    pub fn yolo(&self, scope: &str) -> bool {
        self.get(scope, "YOLO") == Some("true")
    }

    pub fn toggle_yolo(&mut self, scope: &str) -> bool {
        let next = !self.yolo(scope);
        self.set(scope, "YOLO", if next { "true" } else { "false" });
        next
    }

    pub fn branch(&self, scope: &str) -> Option<&str> {
        self.get(scope, "BRANCH").filter(|s| !s.is_empty())
    }

    pub fn set_branch(&mut self, scope: &str, val: &str) {
        self.set(scope, "BRANCH", val);
    }

    pub fn input_tokens(&self, scope: &str) -> i64 {
        self.get(scope, "SESSION_INPUT_TOKENS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    pub fn output_tokens(&self, scope: &str) -> i64 {
        self.get(scope, "SESSION_OUTPUT_TOKENS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    pub fn add_tokens(&mut self, scope: &str, input: i64, output: i64) {
        let prev_in = self.input_tokens(scope);
        let prev_out = self.output_tokens(scope);
        self.set(
            scope,
            "SESSION_INPUT_TOKENS",
            &(prev_in + input).to_string(),
        );
        self.set(
            scope,
            "SESSION_OUTPUT_TOKENS",
            &(prev_out + output).to_string(),
        );
    }

    pub fn clear_session(&mut self, scope: &str) {
        self.set(scope, "SESSION_ID", "");
        self.set(scope, "SESSION_INPUT_TOKENS", "0");
        self.set(scope, "SESSION_OUTPUT_TOKENS", "0");
        self.set(scope, "BRANCH", "");
        self.set(scope, "RECOVERY_DEPTH", "0");
    }

    pub fn waiting_for_dir(&self, scope: &str) -> bool {
        self.get(scope, "WAITING_FOR_DIR") == Some("true")
    }

    pub fn set_waiting_for_dir(&mut self, scope: &str) {
        self.set(scope, "WAITING_FOR_DIR", "true");
    }

    pub fn clear_waiting_for_dir(&mut self, scope: &str) {
        self.set(scope, "WAITING_FOR_DIR", "false");
    }

    /// How many times context-limit recovery has been attempted for this scope.
    pub fn recovery_depth(&self, scope: &str) -> u8 {
        self.get(scope, "RECOVERY_DEPTH")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Increment the recovery depth counter. Returns the new value.
    pub fn increment_recovery_depth(&mut self, scope: &str) -> u8 {
        let next = self.recovery_depth(scope).saturating_add(1);
        self.set(scope, "RECOVERY_DEPTH", &next.to_string());
        next
    }

    /// Reset the recovery depth counter to zero.
    pub fn reset_recovery_depth(&mut self, scope: &str) {
        self.set(scope, "RECOVERY_DEPTH", "0");
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        if !self.dirty {
            return Ok(());
        }
        self.dirty = false;
        let mut lines: Vec<String> = self
            .scopes
            .iter()
            .flat_map(|(scope, entries)| {
                entries.iter().map(move |(k, v)| {
                    serde_json::json!({"scope": scope, "key": k, "value": v}).to_string()
                })
            })
            .collect();
        lines.sort();
        let content = format!("{}\n", lines.join("\n"));
        crate::config::atomic::write_atomic(&self.path, content.as_bytes(), 0o600)?;
        Ok(())
    }

    fn read_file(path: &str) -> HashMap<String, HashMap<String, String>> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(error = %e, path, "State file exists but cannot be read");
                }
                return HashMap::new();
            }
        };
        let mut scopes: HashMap<String, HashMap<String, String>> = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
                if let (Some(scope), Some(key), Some(value)) = (
                    obj["scope"].as_str(),
                    obj["key"].as_str(),
                    obj["value"].as_str(),
                ) {
                    scopes
                        .entry(scope.to_string())
                        .or_default()
                        .insert(key.to_string(), value.to_string());
                }
            } else if let Some((scope_key, kv)) = line.split_once('#') {
                // Legacy format migration: SCOPE#KEY=VALUE
                if let Some((key, value)) = kv.split_once('=') {
                    scopes
                        .entry(scope_key.trim().to_string())
                        .or_default()
                        .insert(key.trim().to_string(), value.trim().to_string());
                }
            } else if let Some((key, value)) = line.split_once('=') {
                // Legacy format (no scope) — migrate to default scope
                scopes
                    .entry("default".to_string())
                    .or_default()
                    .insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        scopes
    }
}

/// Acquire write lock on a shared [`ChannelState`], apply a mutation, and persist.
/// Save errors are logged but not propagated.
///
/// This encapsulates the repeated pattern of `state.write().await`, mutate,
/// `cs.save()`, and error-log.
pub async fn with_write<F, R>(state: &Arc<tokio::sync::RwLock<ChannelState>>, f: F) -> R
where
    F: FnOnce(&mut ChannelState) -> R,
{
    let mut guard = state.write().await;
    let result = f(&mut guard);
    if let Err(e) = guard.save() {
        tracing::error!(error = %e, "Failed to save channel state");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoped_get_set() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state").to_string_lossy().to_string();

        let mut state = ChannelState::load(&path);
        state.set("telegram:123:u1", "SESSION_ID", "abc-123");
        state.set("telegram:123:u1", "MODEL", "sonnet");
        state.set("telegram:456:u2", "SESSION_ID", "def-456");
        state.save().unwrap();

        let state2 = ChannelState::load(&path);
        assert_eq!(state2.get("telegram:123:u1", "SESSION_ID"), Some("abc-123"));
        assert_eq!(state2.get("telegram:123:u1", "MODEL"), Some("sonnet"));
        assert_eq!(state2.get("telegram:456:u2", "SESSION_ID"), Some("def-456"));
        assert_eq!(state2.get("telegram:123:u1", "NONEXISTENT"), None);
        assert_eq!(state2.get("telegram:123:u2", "SESSION_ID"), None);
    }

    #[test]
    fn test_load_handles_missing_file() {
        let state = ChannelState::load("/tmp/nonexistent-state-file-xyz");
        assert_eq!(state.get("any", "anything"), None);
    }

    #[test]
    fn test_load_handles_comments_and_blanks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state");
        std::fs::write(&path, "# comment\n\ntelegram:1:u#KEY=val\n\n").unwrap();

        let state = ChannelState::load(&path.to_string_lossy());
        assert_eq!(state.get("telegram:1:u", "KEY"), Some("val"));
    }

    #[test]
    fn test_legacy_format_migration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state");
        std::fs::write(&path, "SESSION_ID=old-session\nMODEL=haiku\n").unwrap();

        let state = ChannelState::load(&path.to_string_lossy());
        assert_eq!(state.get("default", "SESSION_ID"), Some("old-session"));
        assert_eq!(state.get("default", "MODEL"), Some("haiku"));
    }

    #[test]
    fn test_values_with_special_characters() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state").to_string_lossy().to_string();

        let mut state = ChannelState::load(&path);
        state.set(
            "telegram:123:u1",
            "SESSION_CWD",
            "/Users/foo=bar/workspace#draft",
        );
        state.set("telegram:123:u1", "TOPIC", "x = y # hypothesis");
        state.save().unwrap();

        let state2 = ChannelState::load(&path);
        assert_eq!(
            state2.get("telegram:123:u1", "SESSION_CWD"),
            Some("/Users/foo=bar/workspace#draft")
        );
        assert_eq!(
            state2.get("telegram:123:u1", "TOPIC"),
            Some("x = y # hypothesis")
        );
    }

    #[test]
    fn test_scope_key_encodes_colons() {
        let key = super::scope_key("telegram", "ch:123", "user:id");
        assert_eq!(key, "telegram:ch%3A123:user%3Aid");

        let key2 = super::scope_key("slack", "C123", "U456");
        assert_eq!(key2, "slack:C123:U456");
    }

    #[test]
    fn test_dirty_flag_skips_save() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state").to_string_lossy().to_string();

        // Load — not dirty, save should be no-op
        let mut state = ChannelState::load(&path);
        assert!(!state.dirty);
        state.save().unwrap();
        assert!(
            !std::path::Path::new(&path).exists(),
            "file should not be created for clean state"
        );

        // Set — dirty
        state.set("test:1:a", "KEY", "val");
        assert!(state.dirty);
        state.save().unwrap();
        assert!(!state.dirty, "dirty should be cleared after save");
        assert!(std::path::Path::new(&path).exists());

        // Save again — not dirty, no-op
        state.save().unwrap();
    }
}

#[cfg(test)]
mod typed_accessor_tests {
    use super::*;

    fn test_state() -> ChannelState {
        let dir = tempfile::tempdir().unwrap();
        ChannelState::load(&dir.path().join("state").to_string_lossy())
    }

    const SCOPE: &str = "test_scope";

    #[test]
    fn session_id_returns_none_when_empty() {
        let mut cs = test_state();
        assert!(cs.session_id(SCOPE).is_none());
        cs.set(SCOPE, "SESSION_ID", "");
        assert!(cs.session_id(SCOPE).is_none());
    }

    #[test]
    fn session_id_returns_some_when_set() {
        let mut cs = test_state();
        cs.set_session_id(SCOPE, "abc123");
        assert_eq!(cs.session_id(SCOPE), Some("abc123"));
    }

    #[test]
    fn working_dir_filters_empty() {
        let mut cs = test_state();
        assert!(cs.working_dir(SCOPE).is_none());
        cs.set(SCOPE, "SESSION_CWD", "");
        assert!(cs.working_dir(SCOPE).is_none());
        cs.set_working_dir(SCOPE, "/home/user/project");
        assert_eq!(cs.working_dir(SCOPE), Some("/home/user/project"));
    }

    #[test]
    fn model_filters_empty() {
        let mut cs = test_state();
        assert!(cs.model(SCOPE).is_none());
        cs.set_model(SCOPE, "sonnet");
        assert_eq!(cs.model(SCOPE), Some("sonnet"));
    }

    #[test]
    fn yolo_defaults_false() {
        let cs = test_state();
        assert!(!cs.yolo(SCOPE));
    }

    #[test]
    fn toggle_yolo_roundtrip() {
        let mut cs = test_state();
        assert!(!cs.yolo(SCOPE));
        let next = cs.toggle_yolo(SCOPE);
        assert!(next);
        assert!(cs.yolo(SCOPE));
        let next2 = cs.toggle_yolo(SCOPE);
        assert!(!next2);
        assert!(!cs.yolo(SCOPE));
    }

    #[test]
    fn branch_filters_empty() {
        let mut cs = test_state();
        assert!(cs.branch(SCOPE).is_none());
        cs.set_branch(SCOPE, "main");
        assert_eq!(cs.branch(SCOPE), Some("main"));
    }

    #[test]
    fn last_model_filters_empty() {
        let mut cs = test_state();
        assert!(cs.last_model(SCOPE).is_none());
        cs.set_last_model(SCOPE, "claude-sonnet-4-6");
        assert_eq!(cs.last_model(SCOPE), Some("claude-sonnet-4-6"));
    }

    #[test]
    fn tokens_default_zero() {
        let cs = test_state();
        assert_eq!(cs.input_tokens(SCOPE), 0);
        assert_eq!(cs.output_tokens(SCOPE), 0);
    }

    #[test]
    fn add_tokens_accumulates() {
        let mut cs = test_state();
        cs.add_tokens(SCOPE, 100, 50);
        assert_eq!(cs.input_tokens(SCOPE), 100);
        assert_eq!(cs.output_tokens(SCOPE), 50);
        cs.add_tokens(SCOPE, 200, 150);
        assert_eq!(cs.input_tokens(SCOPE), 300);
        assert_eq!(cs.output_tokens(SCOPE), 200);
    }

    #[test]
    fn clear_session_resets_all() {
        let mut cs = test_state();
        cs.set_session_id(SCOPE, "sid123");
        cs.set(SCOPE, "SESSION_INPUT_TOKENS", "500");
        cs.set(SCOPE, "SESSION_OUTPUT_TOKENS", "300");
        cs.set_branch(SCOPE, "develop");
        cs.clear_session(SCOPE);
        assert!(cs.session_id(SCOPE).is_none());
        assert_eq!(cs.input_tokens(SCOPE), 0);
        assert_eq!(cs.output_tokens(SCOPE), 0);
        assert!(cs.branch(SCOPE).is_none());
        assert_eq!(cs.recovery_depth(SCOPE), 0);
    }

    #[test]
    fn recovery_depth_increments_and_resets() {
        let mut cs = test_state();
        assert_eq!(cs.recovery_depth(SCOPE), 0);
        let d1 = cs.increment_recovery_depth(SCOPE);
        assert_eq!(d1, 1);
        assert_eq!(cs.recovery_depth(SCOPE), 1);
        cs.reset_recovery_depth(SCOPE);
        assert_eq!(cs.recovery_depth(SCOPE), 0);
    }

    #[test]
    fn recovery_depth_saturates() {
        let mut cs = test_state();
        cs.set(SCOPE, "RECOVERY_DEPTH", "255");
        let d = cs.increment_recovery_depth(SCOPE);
        assert_eq!(d, 255); // u8 saturating add: 255 + 1 = 255
    }
}
