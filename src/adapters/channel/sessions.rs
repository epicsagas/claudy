use std::path::Path;

use serde::{Deserialize, Serialize};

/// A project derived from a directory under `~/.claude/projects/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// The encoded directory name (e.g. "-Volumes-T5-projects-claudy").
    pub encoded_dir: String,
    /// Human-readable display name (e.g. "claudy").
    pub project_name: String,
    /// Decoded filesystem path (from cwd in JSONL, or best-effort decode).
    pub project_path: Option<String>,
    /// Number of session files in this project.
    pub session_count: usize,
    /// Most recent modification timestamp (unix seconds).
    pub last_modified: u64,
}

/// A Claude Code session discovered from a JSONL file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session UUID (filename without .jsonl).
    pub session_id: String,
    /// Encoded project directory name.
    pub project: String,
    /// Human-readable project display name.
    pub project_name: String,
    /// Decoded project filesystem path.
    pub project_path: Option<String>,
    /// Working directory extracted from the session JSONL (cwd field).
    pub cwd: Option<String>,
    /// First user message text (truncated).
    pub first_message: Option<String>,
    /// Last user or assistant message text (truncated).
    pub last_message: Option<String>,
    /// File modification time (unix seconds).
    pub last_modified: u64,
}

/// Extract the display name from a decoded project path.
pub fn project_display_name(path: Option<&str>) -> String {
    match path {
        Some(p) => {
            let name = Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.to_string());
            if name.is_empty() {
                "~/".to_string()
            } else {
                name
            }
        }
        None => "unknown".to_string(),
    }
}

/// Extract the `cwd` from the first line of a JSONL session file.
fn extract_cwd(path: &Path) -> Option<String> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return None;
    };
    for line in content.lines().take(50) {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if let Some(cwd) = event["cwd"].as_str() {
            return Some(cwd.to_string());
        }
    }
    None
}

/// Discover all projects under `~/.claude/projects/`.
pub fn discover_projects(claude_projects_dir: &str) -> Vec<ProjectInfo> {
    let base = Path::new(claude_projects_dir);
    if !base.exists() {
        return Vec::new();
    }

    let mut projects: Vec<ProjectInfo> = Vec::new();
    let Ok(entries) = std::fs::read_dir(base) else {
        return Vec::new();
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        if dir_name == "memory" {
            continue;
        }

        let mut session_count = 0usize;
        let mut last_modified = 0u64;
        let mut resolved_cwd: Option<String> = None;
        let mut latest_jsonl: Option<std::path::PathBuf> = None;

        if let Ok(session_entries) = std::fs::read_dir(&path) {
            for se in session_entries.flatten() {
                let sp = se.path();
                if sp.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                    continue;
                }
                session_count += 1;
                let ts = sp
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    })
                    .unwrap_or(0);
                if ts > last_modified {
                    last_modified = ts;
                    latest_jsonl = Some(sp);
                }
            }
        }

        // Extract cwd from the most recent session file
        if let Some(ref jsonl_path) = latest_jsonl {
            resolved_cwd = extract_cwd(jsonl_path);
        }

        let name = project_display_name(resolved_cwd.as_deref());

        if session_count > 0 {
            projects.push(ProjectInfo {
                encoded_dir: dir_name,
                project_name: name,
                project_path: resolved_cwd,
                session_count,
                last_modified,
            });
        }
    }

    projects.sort_by_key(|b| std::cmp::Reverse(b.last_modified));
    projects
}

/// Extract first and last message text from a Claude session JSONL file.
fn extract_messages(path: &Path) -> (Option<String>, Option<String>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return (None, None);
    };

    let mut first_message: Option<String> = None;
    let mut last_message: Option<String> = None;

    for line in content.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        let event_type = event["type"].as_str().unwrap_or("");
        match event_type {
            "user" => {
                if let Some(text) = extract_text_from_content(&event["message"]["content"]) {
                    if text.starts_with("<local-command-caveat>")
                        || text.starts_with("<command-name>")
                        || text.starts_with("<command-message>")
                    {
                        continue;
                    }
                    if first_message.is_none() {
                        first_message = Some(truncate(&text, 120));
                    }
                    last_message = Some(truncate(&text, 120));
                }
            }
            "assistant" => {
                if let Some(text) = extract_text_from_content(&event["message"]["content"])
                    && !text.trim().is_empty()
                {
                    last_message = Some(truncate(&text, 120));
                }
            }
            "summary" => {
                if let Some(text) = event["summary"].as_str() {
                    last_message = Some(truncate(text, 120));
                }
            }
            _ => {}
        }
    }

    (first_message, last_message)
}

/// Extract text from a content field (string or array of blocks).
fn extract_text_from_content(content: &serde_json::Value) -> Option<String> {
    if let Some(s) = content.as_str()
        && !s.is_empty()
    {
        return Some(s.to_string());
    }
    if let Some(arr) = content.as_array() {
        for block in arr {
            if block["type"].as_str() == Some("text")
                && let Some(text) = block["text"].as_str()
                && !text.is_empty()
            {
                return Some(text.to_string());
            }
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .take(max)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(max);
        format!("{}...", &s[..end])
    }
}

/// Discover recent sessions across all projects.
pub fn discover_sessions(claude_projects_dir: &str, limit: usize) -> Vec<SessionInfo> {
    let projects = discover_projects(claude_projects_dir);
    let mut sessions: Vec<SessionInfo> = Vec::new();
    let base = Path::new(claude_projects_dir);

    for proj in &projects {
        let proj_dir = base.join(&proj.encoded_dir);
        let Ok(entries) = std::fs::read_dir(&proj_dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            if session_id.len() != 36 || !session_id.contains('-') {
                continue;
            }

            let last_modified = path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0);

            let (first_message, last_message) = extract_messages(&path);
            let cwd = extract_cwd(&path);

            sessions.push(SessionInfo {
                session_id,
                project: proj.encoded_dir.clone(),
                project_name: proj.project_name.clone(),
                project_path: proj.project_path.clone(),
                cwd,
                first_message,
                last_message,
                last_modified,
            });
        }
    }

    sessions.sort_by_key(|b| std::cmp::Reverse(b.last_modified));
    sessions.truncate(limit);
    sessions
}

/// Reject encoded directory names that could escape the projects root.
fn is_safe_encoded_dir(dir: &str) -> bool {
    !dir.is_empty() && !dir.contains("..") && !dir.contains('/') && !dir.contains('\\')
}

/// Find sessions for a specific project directory.
pub fn discover_project_sessions(
    claude_projects_dir: &str,
    encoded_dir: &str,
    limit: usize,
) -> Vec<SessionInfo> {
    if !is_safe_encoded_dir(encoded_dir) {
        return Vec::new();
    }
    let proj_dir = Path::new(claude_projects_dir).join(encoded_dir);
    if !proj_dir.exists() {
        return Vec::new();
    }

    let resolved_cwd = std::fs::read_dir(&proj_dir).ok().and_then(|mut entries| {
        entries
            .by_ref()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("jsonl"))
            .max_by_key(|e| e.path().metadata().ok().and_then(|m| m.modified().ok()))
            .and_then(|e| extract_cwd(&e.path()))
    });

    let name = project_display_name(resolved_cwd.as_deref());

    let mut sessions: Vec<SessionInfo> = Vec::new();
    let Ok(entries) = std::fs::read_dir(&proj_dir) else {
        return Vec::new();
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }

        let session_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if session_id.len() != 36 || !session_id.contains('-') {
            continue;
        }

        let last_modified = path
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
            .unwrap_or(0);

        let (first_message, last_message) = extract_messages(&path);
        let cwd = extract_cwd(&path);

        sessions.push(SessionInfo {
            session_id,
            project: encoded_dir.to_string(),
            project_name: name.clone(),
            project_path: resolved_cwd.clone(),
            cwd,
            first_message,
            last_message,
            last_modified,
        });
    }

    sessions.sort_by_key(|b| std::cmp::Reverse(b.last_modified));
    sessions.truncate(limit);
    sessions
}

/// Check whether a session JSONL file exists for the given session ID
/// within any project subdirectory of the Claude projects directory.
pub fn session_file_exists(claude_projects_dir: &str, session_id: &str) -> bool {
    let base = Path::new(claude_projects_dir);
    if !base.is_dir() {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(base) else {
        return false;
    };
    for entry in entries.flatten() {
        let candidate = entry.path().join(format!("{}.jsonl", session_id));
        if candidate.exists() {
            return true;
        }
    }
    false
}

/// Resolve the `~/.claude/projects/` directory path.
pub fn claude_projects_dir() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = format!("{}/.claude/projects", home);
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_display_name() {
        assert_eq!(
            project_display_name(Some("/Users/hackme/projects/myapp")),
            "myapp"
        );
        assert_eq!(project_display_name(None), "unknown");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        let long = "a".repeat(200);
        let truncated = truncate(&long, 120);
        assert!(truncated.len() <= 123);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_discover_projects_returns_empty_for_missing_dir() {
        let projects = discover_projects("/tmp/nonexistent-claudy-projects-test");
        assert!(projects.is_empty());
    }

    #[test]
    fn test_extract_messages_with_real_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test-session.jsonl");
        let content = r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"Hello Claude"}]}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hi there!"}]}}
{"type":"user","message":{"role":"user","content":"Second message"}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let (first, last) = extract_messages(&file_path);
        assert_eq!(first.as_deref(), Some("Hello Claude"));
        assert_eq!(last.as_deref(), Some("Second message"));
    }

    #[test]
    fn test_extract_messages_skips_commands() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test-session.jsonl");
        let content = r#"{"type":"user","message":{"role":"user","content":"<command-name>/clear</command-name>"}}
{"type":"user","message":{"role":"user","content":"Real message"}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let (first, _last) = extract_messages(&file_path);
        assert_eq!(first.as_deref(), Some("Real message"));
    }

    #[test]
    fn test_discover_projects_finds_real_projects() {
        let Some(projects_dir) = claude_projects_dir() else {
            return; // Skip if no Claude projects dir
        };
        let projects = discover_projects(&projects_dir);
        assert!(!projects.is_empty());
        // At least one project should have sessions
        assert!(projects.iter().any(|p| p.session_count > 0));
    }

    #[test]
    fn test_discover_sessions_finds_real_sessions() {
        let Some(projects_dir) = claude_projects_dir() else {
            return;
        };
        let sessions = discover_sessions(&projects_dir, 5);
        assert!(!sessions.is_empty());
        // Each session should have a UUID
        assert!(sessions[0].session_id.contains('-'));
    }

    #[test]
    fn test_session_file_exists_found() {
        let dir = tempfile::tempdir().unwrap();
        let proj_dir = dir.path().join("-tmp-test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();
        let session_id = "550e8400-e29b-41d4-a716-446655440000";
        std::fs::write(proj_dir.join(format!("{}.jsonl", session_id)), "").unwrap();

        assert!(session_file_exists(
            dir.path().to_str().unwrap(),
            session_id
        ));
    }

    #[test]
    fn test_session_file_exists_not_found() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!session_file_exists(
            dir.path().to_str().unwrap(),
            "nonexistent-session-id"
        ));
    }

    #[test]
    fn test_safe_encoded_dir_rejects_traversal() {
        assert!(!is_safe_encoded_dir(".."));
        assert!(!is_safe_encoded_dir("../etc"));
        assert!(!is_safe_encoded_dir("foo/../../etc"));
        assert!(!is_safe_encoded_dir("foo\\bar"));
        assert!(!is_safe_encoded_dir(""));
    }

    #[test]
    fn test_safe_encoded_dir_accepts_valid() {
        assert!(is_safe_encoded_dir("-Volumes-T5-projects-claudy"));
        assert!(is_safe_encoded_dir("-home-user-myapp"));
    }

    #[test]
    fn test_discover_project_sessions_rejects_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let sessions = discover_project_sessions(dir.path().to_str().unwrap(), "../etc", 5);
        assert!(sessions.is_empty());
        let sessions = discover_project_sessions(dir.path().to_str().unwrap(), "..", 5);
        assert!(sessions.is_empty());
    }
}
