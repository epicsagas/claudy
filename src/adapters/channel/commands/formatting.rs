use crate::domain::channel_events::{
    Button, ChannelIdentity, ConversationId, InteractionButtons, OutboundMessage,
};
use crate::ports::channel_ports::ChannelPort;

pub(crate) async fn reply(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    text: &str,
) -> anyhow::Result<()> {
    let max = channel_id.platform.max_message_length();
    let text = truncate_chars(text, max);
    channel
        .send_message(&OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: channel_id.clone(),
            text: text.to_string(),
            message_ref: None,
            interaction: None,
        })
        .await?;
    Ok(())
}

pub(crate) async fn reply_with_buttons(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    text: String,
    prompt_text: &str,
    buttons: Vec<Button>,
) -> anyhow::Result<()> {
    let max = channel_id.platform.max_message_length();
    let text = truncate_chars(&text, max).to_string();
    channel
        .send_message(&OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: channel_id.clone(),
            text,
            message_ref: None,
            interaction: Some(InteractionButtons {
                prompt_text: prompt_text.to_string(),
                buttons,
            }),
        })
        .await?;
    Ok(())
}

pub(crate) fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

pub(crate) fn session_buttons(sessions: &[super::super::sessions::SessionInfo]) -> Vec<Button> {
    sessions
        .iter()
        .map(|s| {
            let preview = s.first_message.as_deref().unwrap_or("(no message)");
            let short = truncate_chars(preview, 40);
            let label = format!("{} - {}", s.project_name, short);
            Button {
                id: truncate_button_id(&format!(
                    "sess:{}:{}",
                    s.project,
                    &s.session_id[..8.min(s.session_id.len())]
                )),
                label,
            }
        })
        .collect()
}

pub(crate) fn project_buttons(projects: &[super::super::sessions::ProjectInfo]) -> Vec<Button> {
    projects
        .iter()
        .map(|p| {
            let label = format!("{} ({} sessions)", p.project_name, p.session_count);
            Button {
                id: truncate_button_id(&format!("proj:{}", p.encoded_dir)),
                label,
            }
        })
        .collect()
}

pub(crate) fn resolve_git_branch(cwd: Option<&str>) -> Option<String> {
    let dir = cwd.filter(|d| !d.is_empty())?;
    let head_path = std::path::Path::new(dir).join(".git/HEAD");
    let head = std::fs::read_to_string(&head_path).ok()?;
    let head = head.trim();
    head.strip_prefix("ref: refs/heads/")
        .map(|s| s.to_string())
        .or_else(|| {
            // Detached HEAD — return short hash
            Some(format!("{}...", &head[..8.min(head.len())]))
        })
}

pub(crate) fn context_window_for(model: &str) -> i64 {
    let m = model.to_lowercase();
    if m.contains("gpt-4") || m.contains("gpt4") {
        128_000
    } else if m.contains("gemini") {
        1_000_000
    } else {
        200_000 // Claude default
    }
}

pub(crate) fn format_tokens(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub(crate) fn format_time_ago(ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let diff = now.saturating_sub(ts);
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

const BUTTON_ID_MAX_BYTES: usize = 62;

pub(crate) fn truncate_button_id(id: &str) -> String {
    if id.len() <= BUTTON_ID_MAX_BYTES {
        return id.to_string();
    }
    let mut end = BUTTON_ID_MAX_BYTES;
    while !id.is_char_boundary(end) {
        end -= 1;
    }
    id[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_button_id_short_unchanged() {
        let id = "sess:my-project:abc12345";
        assert_eq!(truncate_button_id(id), id);
    }

    #[test]
    fn truncate_button_id_within_limit() {
        let id = "proj:short-dir";
        assert_eq!(truncate_button_id(id), id);
    }

    #[test]
    fn truncate_button_id_long_sess_truncated() {
        let long_project = "a".repeat(80);
        let id = format!("sess:{}:abc12345", long_project);
        let result = truncate_button_id(&id);
        assert!(result.len() <= 62);
        assert!(result.starts_with("sess:"));
    }

    #[test]
    fn truncate_button_id_long_proj_truncated() {
        let long_dir = "x".repeat(120);
        let id = format!("proj:{}", long_dir);
        let result = truncate_button_id(&id);
        assert!(result.len() <= 62);
        assert!(result.starts_with("proj:"));
    }

    #[test]
    fn truncate_button_id_preserves_prefix() {
        let long_project = "b".repeat(80);
        let id = format!("sess:{}:abc12345", long_project);
        let result = truncate_button_id(&id);
        assert!(result.starts_with("sess:"));
        assert!(result.len() <= 62);
    }

    #[test]
    fn truncate_button_id_utf8_boundary_safe() {
        // Each Korean character is 3 bytes in UTF-8
        let korean = "한".repeat(30); // 90 bytes
        let id = format!("proj:{}", korean);
        let result = truncate_button_id(&id);
        assert!(result.len() <= 62);
        assert!(result.is_char_boundary(result.len()));
        assert!(result.starts_with("proj:"));
    }

    #[test]
    fn session_buttons_long_project_truncated() {
        use super::super::super::sessions::SessionInfo;

        let long_project = "p".repeat(80);
        let sessions = vec![SessionInfo {
            project: long_project.clone(),
            project_name: "test".to_string(),
            session_id: "abcdefgh12345678".to_string(),
            project_path: None,
            cwd: None,
            first_message: Some("hello".to_string()),
            last_message: None,
            last_modified: 0,
        }];
        let buttons = session_buttons(&sessions);
        assert_eq!(buttons.len(), 1);
        assert!(buttons[0].id.len() <= 62);
        assert!(buttons[0].id.starts_with("sess:"));
    }

    #[test]
    fn project_buttons_long_encoded_dir_truncated() {
        use super::super::super::sessions::ProjectInfo;

        let long_dir = "d".repeat(120);
        let projects = vec![ProjectInfo {
            project_name: "test".to_string(),
            encoded_dir: long_dir,
            project_path: None,
            session_count: 3,
            last_modified: 0,
        }];
        let buttons = project_buttons(&projects);
        assert_eq!(buttons.len(), 1);
        assert!(buttons[0].id.len() <= 62);
        assert!(buttons[0].id.starts_with("proj:"));
    }
}
