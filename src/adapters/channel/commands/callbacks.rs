use std::sync::Arc;

use crate::domain::channel_events::{
    ChannelIdentity, ConversationId, IncomingEvent, OutboundMessage, TextMessage,
};
use crate::ports::channel_ports::ChannelPort;

use super::super::server::AppState;
use super::super::state::{ChannelState, with_write};
use super::formatting::*;
use super::handlers::handle_project_sessions;

/// Context passed to callback handlers.
pub struct CallbackContext {
    pub channel: Arc<dyn ChannelPort>,
    pub channel_id: ChannelIdentity,
    pub action: String,
    pub data: String,
    pub callback_message_id: Option<String>,
    pub original_text: Option<String>,
    pub scope: String,
    pub channel_state: Arc<tokio::sync::RwLock<ChannelState>>,
    pub app_state: Arc<AppState>,
}

/// Handle callback from inline keyboard buttons.
pub async fn handle_callback(ctx: CallbackContext) -> anyhow::Result<()> {
    let CallbackContext {
        channel,
        channel_id,
        action,
        data,
        callback_message_id,
        original_text,
        scope,
        channel_state,
        app_state,
    } = ctx;
    match action.as_str() {
        "sess" => {
            handle_session_callback(
                channel.as_ref(),
                &channel_id,
                &data,
                callback_message_id,
                original_text,
                &scope,
                &channel_state,
            )
            .await
        }
        "proj" => {
            handle_project_callback(channel.as_ref(), &channel_id, &data, callback_message_id).await
        }
        "model" => {
            handle_model_callback(
                channel.as_ref(),
                &channel_id,
                &data,
                callback_message_id,
                original_text,
                &scope,
                &channel_state,
            )
            .await
        }
        "reply" => {
            handle_reply_callback(
                channel.as_ref(),
                &channel_id,
                callback_message_id,
                original_text,
            )
            .await
        }
        "choice" => {
            handle_choice_callback(
                channel.as_ref(),
                &channel_id,
                &data,
                callback_message_id,
                original_text,
                app_state,
            )
            .await
        }
        _ => {
            tracing::warn!(action, data, "Unknown callback action");
            Ok(())
        }
    }
}

async fn handle_session_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    data: &str,
    callback_message_id: Option<String>,
    original_text: Option<String>,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<ChannelState>>,
) -> anyhow::Result<()> {
    // Parse "project_dir:session_prefix" from callback data.
    let (project_dir, session_prefix) = match data.split_once(':') {
        Some((dir, prefix)) => (dir, prefix),
        None => {
            return dismiss_keyboard(
                channel,
                channel_id,
                callback_message_id,
                original_text,
                "Invalid session reference.",
            )
            .await;
        }
    };

    let Some(projects_dir) = super::super::sessions::claude_projects_dir() else {
        return dismiss_keyboard(
            channel,
            channel_id,
            callback_message_id,
            original_text,
            "No projects found.",
        )
        .await;
    };

    // Scope search to the originating project
    let sessions =
        super::super::sessions::discover_project_sessions(&projects_dir, project_dir, 50);
    let matched = sessions
        .iter()
        .find(|s| s.session_id.starts_with(session_prefix));

    let Some(session) = matched else {
        return dismiss_keyboard(
            channel,
            channel_id,
            callback_message_id,
            original_text,
            "Session not found.",
        )
        .await;
    };

    // Switch to this session
    {
        with_write(state, |cs| {
            cs.set_session_id(scope, &session.session_id);
            // Use the session's actual cwd from JSONL, fall back to project_path
            let cwd = session
                .cwd
                .as_deref()
                .or(session.project_path.as_deref())
                .unwrap_or("");
            if !cwd.is_empty() {
                cs.set_working_dir(scope, cwd);
            }
        })
        .await;
    }

    let preview = session.first_message.as_deref().unwrap_or("(no message)");
    let result = format!(
        "Switched to session {}...\nProject: {}\nFirst: {}",
        &session.session_id[..8],
        session.project_name,
        truncate_chars(preview, 80)
    );
    dismiss_keyboard(channel, channel_id, callback_message_id, original_text, &result).await
}

async fn handle_project_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    encoded_dir: &str,
    callback_message_id: Option<String>,
) -> anyhow::Result<()> {
    // Dismiss the project list keyboard
    dismiss_keyboard(channel, channel_id, callback_message_id, None, "Loading...").await?;

    // Show sessions for this project
    handle_project_sessions(channel, channel_id, encoded_dir).await
}

async fn handle_model_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    model: &str,
    callback_message_id: Option<String>,
    original_text: Option<String>,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<ChannelState>>,
) -> anyhow::Result<()> {
    if !["sonnet", "opus", "haiku"].contains(&model) {
        return dismiss_keyboard(
            channel,
            channel_id,
            callback_message_id,
            original_text,
            "Unknown model.",
        )
        .await;
    }
    {
        with_write(state, |cs| cs.set_model(scope, model)).await;
    }
    dismiss_keyboard(
        channel,
        channel_id,
        callback_message_id,
        original_text,
        &format!("✅ Model set to: {model}"),
    )
    .await
}

/// Edit the message to remove inline keyboard buttons and append the result.
///
/// If `original_text` is provided the edited message will be:
///
/// ```text
/// <original question>
///
/// <result>
/// ```
///
/// This way the user can still see what they were asked before they tapped
/// the button.  When `original_text` is absent (e.g. Slack/Discord which
/// don't surface message text in interaction payloads yet) only `result` is
/// shown, preserving the previous behaviour.
async fn dismiss_keyboard(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    callback_message_id: Option<String>,
    original_text: Option<String>,
    result: &str,
) -> anyhow::Result<()> {
    let text = match original_text.as_deref().filter(|t| !t.is_empty()) {
        Some(question) => format!("{question}\n\n{result}"),
        None => result.to_string(),
    };

    let Some(msg_id) = callback_message_id else {
        // Fallback: just send a new message
        return reply(channel, channel_id, &text).await;
    };
    channel
        .edit_message(&OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: channel_id.clone(),
            text,
            message_ref: Some(msg_id),
            interaction: None,
        })
        .await
}

async fn handle_reply_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    callback_message_id: Option<String>,
    original_text: Option<String>,
) -> anyhow::Result<()> {
    dismiss_keyboard(
        channel,
        channel_id,
        callback_message_id,
        original_text,
        "Type your response below.",
    )
    .await
}

async fn handle_choice_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    data: &str,
    callback_message_id: Option<String>,
    original_text: Option<String>,
    app_state: Arc<AppState>,
) -> anyhow::Result<()> {
    // Reject if Claude is already running for this scope
    {
        let scope = crate::adapters::channel::state::scope_key(
            channel_id.platform.as_str(),
            &channel_id.channel_id,
            &channel_id.user_id,
        );
        let active = app_state.active_claude.try_lock();
        match active {
            Ok(guard) if guard.contains_key(&scope) => {
                return dismiss_keyboard(
                    channel,
                    channel_id,
                    callback_message_id,
                    original_text,
                    "Claude is busy — wait for the current response to finish.",
                )
                .await;
            }
            Err(_) => {
                return dismiss_keyboard(
                    channel,
                    channel_id,
                    callback_message_id,
                    original_text,
                    "Claude is busy — wait for the current response to finish.",
                )
                .await;
            }
            _ => {} // Lock held, not active for this scope — proceed
        }
    }
    dismiss_keyboard(
        channel,
        channel_id,
        callback_message_id,
        original_text,
        &format!("> {data}"),
    )
    .await?;
    let synthetic = IncomingEvent::TextMessage(TextMessage {
        conversation_id: ConversationId::new(),
        channel: channel_id.clone(),
        text: data.to_string(),
        reply_to_id: None,
    });
    super::super::server::spawn_process_event(app_state, synthetic);
    Ok(())
}
