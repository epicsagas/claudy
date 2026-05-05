use crate::domain::channel_events::{
    Button, ChannelIdentity, ConversationId, InteractionButtons, OutboundMessage,
};
use crate::ports::channel_ports::ChannelPort;

pub(crate) async fn reply(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    text: &str,
) -> anyhow::Result<()> {
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
                id: format!("sess:{}", &s.session_id[..8]),
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
                id: format!("proj:{}", p.encoded_dir),
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
