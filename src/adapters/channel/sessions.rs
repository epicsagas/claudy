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

/// Count thinking blocks with empty/invalid signatures in a session file.
pub fn count_invalid_thinking_blocks(claude_projects_dir: &str, session_id: &str) -> usize {
    let Some(path) = find_session_file(claude_projects_dir, session_id) else {
        return 0;
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return 0;
    };
    let mut count = 0usize;
    for line in content.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if event.pointer("/message/role").and_then(|v| v.as_str()) != Some("assistant") {
            continue;
        }
        if let Some(arr) = event.pointer("/message/content").and_then(|v| v.as_array()) {
            for block in arr {
                if block["type"].as_str() == Some("thinking")
                    && block["signature"].as_str().unwrap_or("").is_empty()
                {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Locate a session JSONL file by scanning all project subdirectories.
fn find_session_file(claude_projects_dir: &str, session_id: &str) -> Option<std::path::PathBuf> {
    let base = Path::new(claude_projects_dir);
    std::fs::read_dir(base).ok()?.flatten().find_map(|entry| {
        let candidate = entry.path().join(format!("{}.jsonl", session_id));
        candidate.exists().then_some(candidate)
    })
}

/// Strip thinking blocks with empty or missing signatures from a session JSONL file.
///
/// ZAI and other non-Anthropic providers write thinking blocks without valid
/// Anthropic signatures. When Claude CLI resumes such a session it sends those
/// blocks to the API, which rejects them with HTTP 400. This function removes
/// the offending blocks in-place so the next `--resume` succeeds.
///
/// Returns the number of thinking blocks removed. Returns `Ok(0)` when the file
/// is clean or does not exist.
pub fn sanitize_session_thinking_blocks(
    claude_projects_dir: &str,
    session_id: &str,
) -> anyhow::Result<usize> {
    let Some(path) = find_session_file(claude_projects_dir, session_id) else {
        return Ok(0);
    };

    let content = std::fs::read_to_string(&path)?;
    let mut removed = 0usize;
    let mut out = String::with_capacity(content.len());
    let mut changed = false;

    for line in content.lines() {
        let Ok(mut event) = serde_json::from_str::<serde_json::Value>(line) else {
            out.push_str(line);
            out.push('\n');
            continue;
        };

        // Only assistant messages can carry thinking blocks.
        let is_assistant =
            event.pointer("/message/role").and_then(|v| v.as_str()) == Some("assistant");

        if is_assistant
            && let Some(arr) = event
                .pointer_mut("/message/content")
                .and_then(|v| v.as_array_mut())
        {
            let mut converted = 0usize;
            for block in arr.iter_mut() {
                if block["type"].as_str() == Some("thinking")
                    && block["signature"].as_str().unwrap_or("").is_empty()
                {
                    // Convert to a text block instead of removing.
                    // Stripping the block entirely violates the Anthropic API requirement
                    // that thinking blocks be included verbatim in conversation history.
                    // A text block has no signature and passes validation while keeping
                    // the reasoning content readable for subsequent turns.
                    let text = block["thinking"].as_str().unwrap_or("").to_string();
                    *block = serde_json::json!({"type": "text", "text": text});
                    converted += 1;
                }
            }
            if converted > 0 {
                removed += converted;
                changed = true;
                out.push_str(&serde_json::to_string(&event)?);
                out.push('\n');
                continue;
            }
        }

        out.push_str(line);
        out.push('\n');
    }

    if !changed {
        return Ok(0);
    }

    // Atomic replace: write to a sibling temp file then rename.
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp = parent.join(format!(".{}.tmp", session_id));
    std::fs::write(&tmp, &out)?;
    std::fs::rename(&tmp, &path)?;

    Ok(removed)
}

/// Rewrite non-conforming `server_tool_use` IDs in a session JSONL file.
///
/// ZAI and other non-Anthropic providers write `server_tool_use` blocks whose
/// `id` field does not match the pattern `^srvtoolu_[a-zA-Z0-9_]+$` required
/// by the Anthropic API. When Claude CLI resumes such a session the API rejects
/// the request with HTTP 400. This function:
///
/// 1. Scans assistant messages for `server_tool_use` blocks with invalid IDs.
/// 2. Builds a remapping table: old ID → valid `srvtoolu_<sanitized>` ID.
/// 3. Rewrites every occurrence — both `server_tool_use.id` in assistant messages
///    and the paired `server_tool_result.tool_use_id` in user messages — so the
///    conversation remains internally consistent.
///
/// Returns the number of IDs remapped. Returns `Ok(0)` when the file is clean
/// or does not exist.
pub fn sanitize_session_server_tool_use_ids(
    claude_projects_dir: &str,
    session_id: &str,
) -> anyhow::Result<usize> {
    let Some(path) = find_session_file(claude_projects_dir, session_id) else {
        return Ok(0);
    };

    let content = std::fs::read_to_string(&path)?;

    // Pass 1: collect all non-conforming server_tool_use IDs.
    let mut id_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut counter = 0usize;
    for line in content.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let is_assistant =
            event.pointer("/message/role").and_then(|v| v.as_str()) == Some("assistant");
        if !is_assistant {
            continue;
        }
        let Some(arr) = event.pointer("/message/content").and_then(|v| v.as_array()) else {
            continue;
        };
        for block in arr {
            if block["type"].as_str() != Some("server_tool_use") {
                continue;
            }
            let Some(id) = block["id"].as_str() else {
                continue;
            };
            if id_map.contains_key(id) {
                continue;
            }
            if is_valid_server_tool_use_id(id) {
                continue;
            }
            // Sanitize: keep only [a-zA-Z0-9_] chars from the original ID.
            let sanitized: String = id
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let new_id = if sanitized.is_empty() {
                format!("srvtoolu_patched{}", counter)
            } else {
                format!("srvtoolu_{}", sanitized)
            };
            id_map.insert(id.to_string(), new_id);
            counter += 1;
        }
    }

    if id_map.is_empty() {
        return Ok(0);
    }

    // Pass 2: rewrite both server_tool_use.id and server_tool_result.tool_use_id.
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        let Ok(mut event) = serde_json::from_str::<serde_json::Value>(line) else {
            out.push_str(line);
            out.push('\n');
            continue;
        };

        let role = event
            .pointer("/message/role")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut changed_line = false;

        if let Some(arr) = event
            .pointer_mut("/message/content")
            .and_then(|v| v.as_array_mut())
        {
            for block in arr.iter_mut() {
                match role.as_str() {
                    "assistant" if block["type"].as_str() == Some("server_tool_use") => {
                        if let Some(old_id) = block["id"].as_str().map(|s| s.to_string())
                            && let Some(new_id) = id_map.get(&old_id)
                        {
                            block["id"] = serde_json::Value::String(new_id.clone());
                            changed_line = true;
                        }
                    }
                    "user" if block["type"].as_str() == Some("server_tool_result") => {
                        if let Some(old_id) = block["tool_use_id"].as_str().map(|s| s.to_string())
                            && let Some(new_id) = id_map.get(&old_id)
                        {
                            block["tool_use_id"] = serde_json::Value::String(new_id.clone());
                            changed_line = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        if changed_line {
            out.push_str(&serde_json::to_string(&event)?);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }

    // Atomic replace.
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp = parent.join(format!(".{}.tmp", session_id));
    std::fs::write(&tmp, &out)?;
    std::fs::rename(&tmp, &path)?;

    Ok(id_map.len())
}

/// Rewrite non-conforming `tool_use` ids in a session JSONL file.
///
/// ZAI/GLM and other OpenAI-compatible providers emit `tool_use` blocks whose
/// `id` follows the `call_<hex>` pattern instead of the Anthropic-required
/// `toolu_[a-zA-Z0-9_]+`. When Claude CLI resumes such a session it forwards
/// the conversation history to the Anthropic API, which rejects the malformed
/// ids with HTTP 400. This mirrors [`sanitize_session_server_tool_use_ids`]:
///
/// 1. Scans assistant messages for `tool_use` blocks with invalid ids.
/// 2. Builds a remapping table: old id → valid `toolu_<sanitized>` id.
/// 3. Rewrites every occurrence — both `tool_use.id` in assistant messages and
///    the paired `tool_result.tool_use_id` in user messages — so the
///    conversation stays internally consistent.
///
/// Returns the number of ids remapped. Returns `Ok(0)` when the file is clean
/// or does not exist.
pub fn sanitize_session_tool_use_ids(
    claude_projects_dir: &str,
    session_id: &str,
) -> anyhow::Result<usize> {
    let Some(path) = find_session_file(claude_projects_dir, session_id) else {
        return Ok(0);
    };

    let content = std::fs::read_to_string(&path)?;

    // Pass 1: collect all non-conforming tool_use ids.
    let mut id_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut counter = 0usize;
    for line in content.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let is_assistant =
            event.pointer("/message/role").and_then(|v| v.as_str()) == Some("assistant");
        if !is_assistant {
            continue;
        }
        let Some(arr) = event.pointer("/message/content").and_then(|v| v.as_array()) else {
            continue;
        };
        for block in arr {
            if block["type"].as_str() != Some("tool_use") {
                continue;
            }
            let Some(id) = block["id"].as_str() else {
                continue;
            };
            if id_map.contains_key(id) {
                continue;
            }
            if is_valid_tool_use_id(id) {
                continue;
            }
            // Sanitize: keep only [a-zA-Z0-9_] chars from the original id.
            let sanitized: String = id
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let new_id = if sanitized.is_empty() {
                format!("toolu_patched{}", counter)
            } else {
                format!("toolu_{}", sanitized)
            };
            id_map.insert(id.to_string(), new_id);
            counter += 1;
        }
    }

    if id_map.is_empty() {
        return Ok(0);
    }

    // Pass 2: rewrite both tool_use.id and tool_result.tool_use_id.
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        let Ok(mut event) = serde_json::from_str::<serde_json::Value>(line) else {
            out.push_str(line);
            out.push('\n');
            continue;
        };

        let role = event
            .pointer("/message/role")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut changed_line = false;

        if let Some(arr) = event
            .pointer_mut("/message/content")
            .and_then(|v| v.as_array_mut())
        {
            for block in arr.iter_mut() {
                match role.as_str() {
                    "assistant" if block["type"].as_str() == Some("tool_use") => {
                        if let Some(old_id) = block["id"].as_str().map(|s| s.to_string())
                            && let Some(new_id) = id_map.get(&old_id)
                        {
                            block["id"] = serde_json::Value::String(new_id.clone());
                            changed_line = true;
                        }
                    }
                    "user" if block["type"].as_str() == Some("tool_result") => {
                        if let Some(old_id) = block["tool_use_id"].as_str().map(|s| s.to_string())
                            && let Some(new_id) = id_map.get(&old_id)
                        {
                            block["tool_use_id"] = serde_json::Value::String(new_id.clone());
                            changed_line = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        if changed_line {
            out.push_str(&serde_json::to_string(&event)?);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }

    // Atomic replace.
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp = parent.join(format!(".{}.tmp", session_id));
    std::fs::write(&tmp, &out)?;
    std::fs::rename(&tmp, &path)?;

    Ok(id_map.len())
}

fn is_valid_server_tool_use_id(id: &str) -> bool {
    id.starts_with("srvtoolu_")
        && id.len() > "srvtoolu_".len()
        && id["srvtoolu_".len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Whether a `tool_use` block id conforms to the Anthropic pattern
/// `toolu_[a-zA-Z0-9_]+`. Non-Anthropic providers (ZAI/GLM) emit OpenAI-style
/// `call_<hex>` ids which the Anthropic API rejects on resume.
fn is_valid_tool_use_id(id: &str) -> bool {
    id.starts_with("toolu_")
        && id.len() > "toolu_".len()
        && id["toolu_".len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Summary of fixes applied by [`sanitize_session`].
#[derive(Debug, Default)]
pub struct SanitizeReport {
    /// `thinking` blocks with no/empty signature converted to `text` blocks.
    pub thinking_converted: usize,
    /// `tool_result` blocks stripped from `assistant` messages.
    pub misplaced_tool_results: usize,
    /// `server_tool_use` blocks stripped from `assistant` messages.
    pub server_tool_uses: usize,
    /// `server_tool_use` IDs remapped to conform to Anthropic spec.
    pub server_tool_use_ids_remapped: usize,
    /// `tool_use` IDs remapped to conform to Anthropic spec (`toolu_*`).
    pub tool_use_ids_remapped: usize,
}

impl SanitizeReport {
    pub fn total(&self) -> usize {
        self.thinking_converted
            + self.misplaced_tool_results
            + self.server_tool_uses
            + self.server_tool_use_ids_remapped
            + self.tool_use_ids_remapped
    }
}

/// Run all known session sanitizers in one pass and return a combined report.
///
/// Covers every Anthropic API incompatibility written by non-Anthropic providers
/// (GLM, ZAI, …):
///
/// 1. `thinking` blocks with empty/missing signature → converted to `text`
/// 2. `tool_result` in `assistant` messages → stripped
/// 3. `server_tool_use` in `assistant` messages (webReader, analyze_image) → stripped
/// 4. `server_tool_use` IDs with invalid format → remapped to `srvtoolu_*`
///
/// Returns `Ok(report)` with all counts zero when the file is already clean or
/// does not exist.
pub fn sanitize_session(
    claude_projects_dir: &str,
    session_id: &str,
) -> anyhow::Result<SanitizeReport> {
    let mut report = SanitizeReport {
        thinking_converted: sanitize_session_thinking_blocks(claude_projects_dir, session_id)?,
        server_tool_use_ids_remapped: sanitize_session_server_tool_use_ids(
            claude_projects_dir,
            session_id,
        )?,
        tool_use_ids_remapped: sanitize_session_tool_use_ids(claude_projects_dir, session_id)?,
        ..Default::default()
    };

    // Strip misplaced tool_result and server_tool_use blocks from assistant messages.
    let Some(path) = find_session_file(claude_projects_dir, session_id) else {
        return Ok(report);
    };
    let content = std::fs::read_to_string(&path)?;
    let mut out = String::with_capacity(content.len());
    let mut changed = false;

    for line in content.lines() {
        let Ok(mut event) = serde_json::from_str::<serde_json::Value>(line) else {
            out.push_str(line);
            out.push('\n');
            continue;
        };

        let is_assistant =
            event.pointer("/message/role").and_then(|v| v.as_str()) == Some("assistant");

        if is_assistant
            && let Some(arr) = event
                .pointer_mut("/message/content")
                .and_then(|v| v.as_array_mut())
        {
            let before = arr.len();
            // Count per type before retain so we can attribute correctly.
            for b in arr.iter() {
                match b["type"].as_str() {
                    Some("tool_result") => report.misplaced_tool_results += 1,
                    Some("server_tool_use") => report.server_tool_uses += 1,
                    _ => {}
                }
            }
            arr.retain(|b| {
                !matches!(
                    b["type"].as_str(),
                    Some("tool_result") | Some("server_tool_use")
                )
            });
            if arr.len() != before {
                changed = true;
                out.push_str(&serde_json::to_string(&event)?);
                out.push('\n');
                continue;
            }
        }

        out.push_str(line);
        out.push('\n');
    }

    if changed {
        let parent = path.parent().unwrap_or(Path::new("."));
        let tmp = parent.join(format!(".{}.tmp", session_id));
        std::fs::write(&tmp, &out)?;
        std::fs::rename(&tmp, &path)?;
    }

    Ok(report)
}

/// Find the most recent session ID for the current working directory.
///
/// Claude stores sessions under `~/.claude/projects/<encoded-cwd>/`.
/// The encoding replaces every `/` in the path with `-`, so
/// `/home/user/myapp` → `-home-user-myapp`.
///
/// Returns `None` when the current directory cannot be determined, when no
/// project directory exists for it, or when the project has no sessions yet.
pub fn find_most_recent_session_id_for_cwd(claude_projects_dir: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let encoded = cwd.to_string_lossy().replace('/', "-");
    let sessions = discover_project_sessions(claude_projects_dir, &encoded, 1);
    sessions.into_iter().next().map(|s| s.session_id)
}

/// Check whether a session JSONL file exists for the given session ID
/// within any project subdirectory of the Claude projects directory.
pub fn session_file_exists(claude_projects_dir: &str, session_id: &str) -> bool {
    find_session_file(claude_projects_dir, session_id).is_some()
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
            project_display_name(Some("/home/user/projects/myapp")),
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

    #[test]
    fn sanitize_converts_empty_signature_thinking_to_text() {
        let dir = tempfile::tempdir().unwrap();
        let proj_dir = dir.path().join("-test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let session_id = "550e8400-e29b-41d4-a716-446655440001";
        let jsonl = "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"text\",\"text\":\"hello\"}]}}\n{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"model\":\"glm-5.1\",\"content\":[{\"type\":\"thinking\",\"thinking\":\"some thoughts\",\"signature\":\"\"},{\"type\":\"text\",\"text\":\"response\"}]}}\n".to_string();
        std::fs::write(proj_dir.join(format!("{}.jsonl", session_id)), &jsonl).unwrap();

        let removed =
            sanitize_session_thinking_blocks(dir.path().to_str().unwrap(), session_id).unwrap();
        assert_eq!(removed, 1);

        let patched =
            std::fs::read_to_string(proj_dir.join(format!("{}.jsonl", session_id))).unwrap();
        // thinking block converted to text block — type field gone, content preserved
        assert!(
            !patched.contains(r#""type":"thinking""#),
            "thinking block type should be replaced"
        );
        assert!(
            patched.contains("some thoughts"),
            "thinking content must be preserved as text"
        );
        assert!(
            patched.contains("\"response\""),
            "original text block must survive"
        );
    }

    #[test]
    fn sanitize_keeps_valid_signature_thinking_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let proj_dir = dir.path().join("-test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let session_id = "550e8400-e29b-41d4-a716-446655440002";
        let jsonl = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"thinking\",\"thinking\":\"thoughts\",\"signature\":\"valid-sig-abc123\"},{\"type\":\"text\",\"text\":\"ok\"}]}}\n".to_string();
        std::fs::write(proj_dir.join(format!("{}.jsonl", session_id)), &jsonl).unwrap();

        let removed =
            sanitize_session_thinking_blocks(dir.path().to_str().unwrap(), session_id).unwrap();
        assert_eq!(removed, 0, "valid signature must not be stripped");
    }

    #[test]
    fn sanitize_returns_zero_for_missing_session() {
        let dir = tempfile::tempdir().unwrap();
        let removed = sanitize_session_thinking_blocks(
            dir.path().to_str().unwrap(),
            "nonexistent-0000-0000-0000-000000000000",
        )
        .unwrap();
        assert_eq!(removed, 0);
    }

    #[test]
    fn sanitize_server_tool_use_ids_remaps_invalid_ids() {
        let dir = tempfile::tempdir().unwrap();
        let proj_dir = dir.path().join("-test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let session_id = "550e8400-e29b-41d4-a716-446655440010";
        // zai-style ID that does not match ^srvtoolu_[a-zA-Z0-9_]+$
        let jsonl = concat!(
            "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"server_tool_use\",\"id\":\"toolu_glm_abc123\",\"name\":\"web_search\",\"input\":{}}]}}\n",
            "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"server_tool_result\",\"tool_use_id\":\"toolu_glm_abc123\",\"content\":\"result\"}]}}\n",
        );
        std::fs::write(proj_dir.join(format!("{}.jsonl", session_id)), jsonl).unwrap();

        let remapped =
            sanitize_session_server_tool_use_ids(dir.path().to_str().unwrap(), session_id).unwrap();
        assert_eq!(remapped, 1, "one ID should be remapped");

        let patched =
            std::fs::read_to_string(proj_dir.join(format!("{}.jsonl", session_id))).unwrap();
        assert!(
            !patched.contains("\"toolu_glm_abc123\""),
            "original invalid ID must be replaced"
        );
        assert!(
            patched.contains("\"srvtoolu_"),
            "replacement ID must start with srvtoolu_"
        );
        // tool_use_id in user message must reference the same new ID
        let lines: Vec<&str> = patched.lines().collect();
        let asst: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let user: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let new_id = asst["message"]["content"][0]["id"].as_str().unwrap();
        let ref_id = user["message"]["content"][0]["tool_use_id"]
            .as_str()
            .unwrap();
        assert_eq!(
            new_id, ref_id,
            "server_tool_use.id and tool_use_id must match"
        );
    }

    #[test]
    fn sanitize_server_tool_use_ids_skips_valid_ids() {
        let dir = tempfile::tempdir().unwrap();
        let proj_dir = dir.path().join("-test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let session_id = "550e8400-e29b-41d4-a716-446655440011";
        let jsonl = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"server_tool_use\",\"id\":\"srvtoolu_abc123\",\"name\":\"web_search\",\"input\":{}}]}}\n";
        std::fs::write(proj_dir.join(format!("{}.jsonl", session_id)), jsonl).unwrap();

        let remapped =
            sanitize_session_server_tool_use_ids(dir.path().to_str().unwrap(), session_id).unwrap();
        assert_eq!(remapped, 0, "valid ID must not be remapped");
    }

    #[test]
    fn is_valid_server_tool_use_id_accepts_conforming() {
        assert!(is_valid_server_tool_use_id("srvtoolu_abc123"));
        assert!(is_valid_server_tool_use_id("srvtoolu_ABC_xyz_0"));
    }

    #[test]
    fn is_valid_server_tool_use_id_rejects_nonconforming() {
        assert!(!is_valid_server_tool_use_id("toolu_glm_abc"));
        assert!(!is_valid_server_tool_use_id("srvtoolu_"));
        assert!(!is_valid_server_tool_use_id("srvtoolu_abc-def"));
        assert!(!is_valid_server_tool_use_id(""));
    }

    #[test]
    fn is_valid_tool_use_id_accepts_conforming() {
        assert!(is_valid_tool_use_id("toolu_01WbH7drr7XYVvBJYhemu62f"));
        assert!(is_valid_tool_use_id("toolu_ABC_xyz_0"));
    }

    #[test]
    fn is_valid_tool_use_id_rejects_nonconforming() {
        // OpenAI-style ids emitted by ZAI/GLM must be rejected.
        assert!(!is_valid_tool_use_id("call_c6e7fbb6a99d4dd98d921764"));
        assert!(!is_valid_tool_use_id("toolu_"));
        assert!(!is_valid_tool_use_id("toolu_abc-def"));
        assert!(!is_valid_tool_use_id(""));
    }

    #[test]
    fn sanitize_tool_use_ids_remaps_call_ids_on_both_sides() {
        let dir = tempfile::tempdir().unwrap();
        let proj_dir = dir.path().join("-test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let session_id = "660e8400-e29b-41d4-a716-446655440020";
        // ZAI/GLM-style tool_use id that does not match ^toolu_[a-zA-Z0-9_]+$
        let jsonl = concat!(
            "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"id\":\"call_c6e7fbb6a99d4dd98d921764\",\"name\":\"get_weather\",\"input\":{\"location\":\"SF\"}}]}}\n",
            "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"tool_result\",\"tool_use_id\":\"call_c6e7fbb6a99d4dd98d921764\",\"content\":\"65F\"}]}}\n",
        );
        std::fs::write(proj_dir.join(format!("{}.jsonl", session_id)), jsonl).unwrap();

        let remapped =
            sanitize_session_tool_use_ids(dir.path().to_str().unwrap(), session_id).unwrap();
        assert_eq!(remapped, 1, "one tool_use id should be remapped");

        let patched =
            std::fs::read_to_string(proj_dir.join(format!("{}.jsonl", session_id))).unwrap();
        // No id field may still start with the OpenAI-style "call_" prefix.
        assert!(
            !patched.contains("\"id\":\"call_"),
            "OpenAI-style tool_use id must be replaced"
        );
        assert!(
            patched.contains("\"toolu_"),
            "replacement id must start with toolu_"
        );

        // tool_use.id and tool_result.tool_use_id must reference the same new id
        let lines: Vec<&str> = patched.lines().collect();
        let asst: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let user: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let new_id = asst["message"]["content"][0]["id"].as_str().unwrap();
        let ref_id = user["message"]["content"][0]["tool_use_id"]
            .as_str()
            .unwrap();
        assert_eq!(
            new_id, ref_id,
            "tool_use.id and tool_use_id must match after remap"
        );
    }

    #[test]
    fn sanitize_tool_use_ids_skips_conforming_toolu_ids() {
        let dir = tempfile::tempdir().unwrap();
        let proj_dir = dir.path().join("-test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let session_id = "660e8400-e29b-41d4-a716-446655440021";
        // Already-conforming Anthropic session must be untouched (regression guard).
        let jsonl = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"id\":\"toolu_01WbH7drr7XYVvBJYhemu62f\",\"name\":\"get_weather\",\"input\":{}}]}}\n";
        std::fs::write(proj_dir.join(format!("{}.jsonl", session_id)), jsonl).unwrap();

        let remapped =
            sanitize_session_tool_use_ids(dir.path().to_str().unwrap(), session_id).unwrap();
        assert_eq!(remapped, 0, "conforming toolu_ id must not be remapped");
    }

    #[test]
    fn sanitize_tool_use_ids_does_not_touch_server_tool_use() {
        let dir = tempfile::tempdir().unwrap();
        let proj_dir = dir.path().join("-test-project");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let session_id = "660e8400-e29b-41d4-a716-446655440022";
        // server_tool_use blocks have their own sanitizer; this one must be left
        // for sanitize_session_server_tool_use_ids and not touched here.
        let jsonl = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"server_tool_use\",\"id\":\"toolu_glm_abc123\",\"name\":\"web_search\",\"input\":{}}]}}\n";
        std::fs::write(proj_dir.join(format!("{}.jsonl", session_id)), jsonl).unwrap();

        let remapped =
            sanitize_session_tool_use_ids(dir.path().to_str().unwrap(), session_id).unwrap();
        assert_eq!(remapped, 0, "server_tool_use must not be remapped here");

        let patched =
            std::fs::read_to_string(proj_dir.join(format!("{}.jsonl", session_id))).unwrap();
        assert!(
            patched.contains("toolu_glm_abc123"),
            "server_tool_use content must be untouched"
        );
    }

    #[test]
    fn sanitize_tool_use_ids_returns_zero_for_missing_session() {
        let dir = tempfile::tempdir().unwrap();
        let removed = sanitize_session_tool_use_ids(
            dir.path().to_str().unwrap(),
            "nonexistent-0000-0000-0000-000000000000",
        )
        .unwrap();
        assert_eq!(removed, 0);
    }
}
