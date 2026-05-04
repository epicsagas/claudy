use std::sync::Arc;

use crate::config::registry::BridgeSettings;
use crate::domain::channel_events::{
    Button, ChannelIdentity, ConversationId, IncomingEvent, InteractionButtons, OutboundMessage,
    TextMessage,
};
use crate::ports::channel_ports::ChannelPort;

async fn reply(
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

fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

fn session_buttons(sessions: &[super::sessions::SessionInfo]) -> Vec<Button> {
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

fn project_buttons(projects: &[super::sessions::ProjectInfo]) -> Vec<Button> {
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

pub async fn handle_help(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
) -> anyhow::Result<()> {
    let text = "Claudy Bot Commands:\n\n\
        /help - Show available commands\n\
        /cancel - Cancel current task\n\
        /model - Change Claude model\n\
        /yolo - Toggle auto-allow permissions\n\
        /status - Show session status\n\
        /sessions - List recent sessions\n\
        /projects - List projects\n\
        /new - Start new session\n\
        /history - Show session history\n\n\
        Otherwise, just type a message to talk to Claude!";
    reply(channel, channel_id, text).await
}

pub async fn handle_cancel(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    active_claude: &Arc<tokio::sync::Mutex<Option<u32>>>,
) -> anyhow::Result<()> {
    let killed = {
        let mut active = active_claude.lock().await;
        match active.take() {
            Some(pid) => {
                // Send SIGTERM to the Claude process
                #[cfg(unix)]
                {
                    use std::process::Command;
                    let _ = Command::new("kill")
                        .arg("-TERM")
                        .arg(pid.to_string())
                        .output();
                }
                #[cfg(windows)]
                {
                    use std::process::Command;
                    let _ = Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F"])
                        .output();
                }
                tracing::info!(pid, "Killed Claude process via /cancel");
                true
            }
            None => false,
        }
    };
    let msg = if killed {
        "Task cancelled."
    } else {
        "No task running."
    };
    reply(channel, channel_id, msg).await
}

pub async fn handle_yolo(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<super::state::ChannelState>>,
) -> anyhow::Result<()> {
    let label = {
        let mut cs = state.write().await;
        let next = cs.toggle_yolo(scope);
        if let Err(e) = cs.save() {
            tracing::error!(error = %e, "Failed to persist yolo state");
        }
        if next {
            "Yolo: ON (auto-allow all permissions)"
        } else {
            "Yolo: OFF (ask permission per tool)"
        }
    };
    reply(channel, channel_id, label).await
}

pub async fn handle_model(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    args: &str,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<super::state::ChannelState>>,
) -> anyhow::Result<()> {
    if args.is_empty() {
        channel
            .send_message(&OutboundMessage {
                conversation_id: ConversationId::new(),
                channel: channel_id.clone(),
                text: "Choose a model:".to_string(),
                message_ref: None,
                interaction: Some(InteractionButtons {
                    prompt_text: "Model selection".to_string(),
                    buttons: vec![
                        Button {
                            id: "model:sonnet".to_string(),
                            label: "Sonnet".to_string(),
                        },
                        Button {
                            id: "model:opus".to_string(),
                            label: "Opus".to_string(),
                        },
                        Button {
                            id: "model:haiku".to_string(),
                            label: "Haiku".to_string(),
                        },
                    ],
                }),
            })
            .await?;
    } else {
        if !["sonnet", "opus", "haiku"].contains(&args) {
            return reply(
                channel,
                channel_id,
                &format!("Unknown model: {}. Choose from: sonnet, opus, haiku", args),
            )
            .await;
        }
        {
            let mut cs = state.write().await;
            cs.set_model(scope, args);
            if let Err(e) = cs.save() {
                tracing::error!(error = %e, "Failed to persist model state");
            }
        }
        reply(channel, channel_id, &format!("Model set to: {}", args)).await?;
    }
    Ok(())
}

fn resolve_git_branch(cwd: Option<&str>) -> Option<String> {
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

fn context_window_for(model: &str) -> i64 {
    let m = model.to_lowercase();
    if m.contains("gpt-4") || m.contains("gpt4") {
        128_000
    } else if m.contains("gemini") {
        1_000_000
    } else {
        200_000 // Claude default
    }
}

fn format_tokens(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub struct SessionStatus<'a> {
    pub session_id: Option<&'a str>,
    pub cwd: Option<&'a str>,
    pub model: Option<&'a str>,
    pub yolo: bool,
    pub branch: Option<&'a str>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub last_model: Option<&'a str>,
}

pub async fn handle_status(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    config: &BridgeSettings,
    status: SessionStatus<'_>,
) -> anyhow::Result<()> {
    let platform = channel_id.platform.as_str();
    let profile = config.profile_for(platform);
    let mode = config
        .mode_for(platform)
        .unwrap_or_else(|| "default".to_string());
    let session_info = match status.session_id {
        Some(id) => format!(
            "Session: {}... ({})",
            &id[..8.min(id.len())],
            status.cwd.unwrap_or("unknown dir")
        ),
        None => "Session: none".to_string(),
    };
    let model_info = status.last_model.or(status.model).unwrap_or("default");
    let yolo_label = if status.yolo {
        "ON (auto-allow)"
    } else {
        "OFF"
    };

    let branch_display = status
        .branch
        .filter(|b| !b.is_empty())
        .map(|b| b.to_string())
        .or_else(|| resolve_git_branch(status.cwd))
        .unwrap_or_else(|| "unknown".to_string());

    let mut text = format!(
        "Status:\n{}\nBranch: {}\nModel: {}\nProfile: {}\nMode: {}\nYolo: {}",
        session_info, branch_display, model_info, profile, mode, yolo_label,
    );

    if status.input_tokens > 0 || status.output_tokens > 0 {
        let ctx = context_window_for(model_info);
        let pct = if ctx > 0 {
            (status.input_tokens as f64 / ctx as f64 * 100.0 * 10.0).round() / 10.0
        } else {
            0.0
        };
        text.push_str(&format!(
            "\nTokens: {} in / {} out ({}% of context)",
            format_tokens(status.input_tokens),
            format_tokens(status.output_tokens),
            pct,
        ));
    }

    text.push_str(&format!("\nListen: {}", config.listen_addr));
    reply(channel, channel_id, &text).await
}

pub async fn handle_sessions(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
) -> anyhow::Result<()> {
    let Some(projects_dir) = super::sessions::claude_projects_dir() else {
        return reply(channel, channel_id, "No Claude projects found.").await;
    };

    let sessions = super::sessions::discover_sessions(&projects_dir, 5);
    if sessions.is_empty() {
        return reply(channel, channel_id, "No sessions found.").await;
    }

    let mut text = "Recent Sessions:\n\n".to_string();
    for (i, s) in sessions.iter().enumerate() {
        let preview = s.first_message.as_deref().unwrap_or("(no message)");
        let ago = format_time_ago(s.last_modified);
        text.push_str(&format!(
            "{}. {} - {} [{}]\n",
            i + 1,
            s.project_name,
            truncate_chars(preview, 50),
            ago
        ));
    }

    let buttons = session_buttons(&sessions);
    channel
        .send_message(&OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: channel_id.clone(),
            text,
            message_ref: None,
            interaction: Some(InteractionButtons {
                prompt_text: "Select a session".to_string(),
                buttons,
            }),
        })
        .await?;
    Ok(())
}

pub async fn handle_projects(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
) -> anyhow::Result<()> {
    let Some(projects_dir) = super::sessions::claude_projects_dir() else {
        return reply(channel, channel_id, "No Claude projects found.").await;
    };

    let projects = super::sessions::discover_projects(&projects_dir);
    if projects.is_empty() {
        return reply(channel, channel_id, "No projects found.").await;
    }

    let mut text = "Projects:\n\n".to_string();
    for p in projects.iter().take(10) {
        let path = p.project_path.as_deref().unwrap_or(&p.encoded_dir);
        text.push_str(&format!(
            "{} ({} sessions) - {}\n",
            p.project_name, p.session_count, path
        ));
    }

    let buttons = project_buttons(&projects[..10.min(projects.len())]);
    channel
        .send_message(&OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: channel_id.clone(),
            text,
            message_ref: None,
            interaction: Some(InteractionButtons {
                prompt_text: "Select a project".to_string(),
                buttons,
            }),
        })
        .await?;
    Ok(())
}

pub async fn handle_project_sessions(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    encoded_dir: &str,
) -> anyhow::Result<()> {
    let Some(projects_dir) = super::sessions::claude_projects_dir() else {
        return reply(channel, channel_id, "No Claude projects found.").await;
    };

    let sessions = super::sessions::discover_project_sessions(&projects_dir, encoded_dir, 5);
    if sessions.is_empty() {
        return reply(channel, channel_id, "No sessions in this project.").await;
    }

    let project_name = sessions
        .first()
        .map(|s| s.project_name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let mut text = format!("Sessions in {}:\n\n", project_name);
    for (i, s) in sessions.iter().enumerate() {
        let preview = s.first_message.as_deref().unwrap_or("(no message)");
        let ago = format_time_ago(s.last_modified);
        text.push_str(&format!(
            "{}. {} [{}]\n",
            i + 1,
            truncate_chars(preview, 60),
            ago
        ));
    }

    let buttons = session_buttons(&sessions);
    channel
        .send_message(&OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: channel_id.clone(),
            text,
            message_ref: None,
            interaction: Some(InteractionButtons {
                prompt_text: "Select a session".to_string(),
                buttons,
            }),
        })
        .await?;
    Ok(())
}

pub async fn handle_new(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<super::state::ChannelState>>,
) -> anyhow::Result<()> {
    let cwd_info = {
        let mut cs = state.write().await;
        cs.clear_session(scope);
        if let Err(e) = cs.save() {
            tracing::error!(error = %e, "Failed to persist new session state");
        }
        cs.working_dir(scope)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default workspace".to_string())
    };

    reply(
        channel,
        channel_id,
        &format!("New session started.\nWorking dir: {}", cwd_info),
    )
    .await
}

pub async fn handle_history(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<super::state::ChannelState>>,
) -> anyhow::Result<()> {
    let session_id = {
        let cs = state.read().await;
        match cs.session_id(scope) {
            Some(id) => id.to_string(),
            None => return reply(channel, channel_id, "No active session.").await,
        }
    };

    let Some(projects_dir) = super::sessions::claude_projects_dir() else {
        return reply(channel, channel_id, "No Claude projects found.").await;
    };

    let base = std::path::Path::new(&projects_dir);
    let jsonl = find_session_jsonl(base, &session_id);

    let Some(jsonl_path) = jsonl else {
        return reply(
            channel,
            channel_id,
            &format!("Session {} not found on disk.", &session_id[..8]),
        )
        .await;
    };

    let content = match std::fs::read_to_string(&jsonl_path) {
        Ok(c) => c,
        Err(_) => return reply(channel, channel_id, "Failed to read session file.").await,
    };

    let mut messages: Vec<String> = Vec::new();
    for line in content.lines().rev() {
        if messages.len() >= 10 {
            break;
        }
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let event_type = event["type"].as_str().unwrap_or("");
        match event_type {
            "user" => {
                if let Some(text) = extract_text(&event["message"]["content"]) {
                    if text.starts_with('<') {
                        continue;
                    }
                    messages.push(format!("You: {}", truncate_chars(&text, 100)));
                }
            }
            "assistant" => {
                if let Some(text) = extract_text(&event["message"]["content"])
                    && !text.trim().is_empty()
                {
                    messages.push(format!("Bot: {}", truncate_chars(&text, 100)));
                }
            }
            _ => {}
        }
    }
    messages.reverse();

    if messages.is_empty() {
        return reply(channel, channel_id, "Session has no messages yet.").await;
    }

    let text = format!(
        "History (session {}...):\n\n{}",
        &session_id[..8],
        messages.join("\n")
    );
    reply(channel, channel_id, &text).await
}

/// Context passed to callback handlers.
pub struct CallbackContext {
    pub channel: Arc<dyn ChannelPort>,
    pub channel_id: ChannelIdentity,
    pub action: String,
    pub data: String,
    pub callback_message_id: Option<i64>,
    pub scope: String,
    pub channel_state: Arc<tokio::sync::RwLock<super::state::ChannelState>>,
    pub app_state: Arc<super::server::AppState>,
}

/// Handle callback from inline keyboard buttons.
pub async fn handle_callback(ctx: CallbackContext) -> anyhow::Result<()> {
    let CallbackContext {
        channel,
        channel_id,
        action,
        data,
        callback_message_id,
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
                &scope,
                &channel_state,
            )
            .await
        }
        "reply" => handle_reply_callback(channel.as_ref(), &channel_id, callback_message_id).await,
        "choice" => {
            handle_choice_callback(
                channel.as_ref(),
                &channel_id,
                &data,
                callback_message_id,
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
    session_prefix: &str,
    callback_message_id: Option<i64>,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<super::state::ChannelState>>,
) -> anyhow::Result<()> {
    // Find full session ID by prefix
    let Some(projects_dir) = super::sessions::claude_projects_dir() else {
        return dismiss_keyboard(
            channel,
            channel_id,
            callback_message_id,
            "No projects found.",
        )
        .await;
    };

    let sessions = super::sessions::discover_sessions(&projects_dir, 50);
    let matched = sessions
        .iter()
        .find(|s| s.session_id.starts_with(session_prefix));

    let Some(session) = matched else {
        return dismiss_keyboard(
            channel,
            channel_id,
            callback_message_id,
            "Session not found.",
        )
        .await;
    };

    // Switch to this session
    {
        let mut cs = state.write().await;
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
        if let Err(e) = cs.save() {
            tracing::error!(error = %e, "Failed to persist session callback state");
        }
    }

    let preview = session.first_message.as_deref().unwrap_or("(no message)");
    let text = format!(
        "Switched to session {}...\nProject: {}\nFirst: {}",
        &session.session_id[..8],
        session.project_name,
        truncate_chars(preview, 80)
    );
    dismiss_keyboard(channel, channel_id, callback_message_id, &text).await
}

async fn handle_project_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    encoded_dir: &str,
    callback_message_id: Option<i64>,
) -> anyhow::Result<()> {
    // Dismiss the project list keyboard
    dismiss_keyboard(channel, channel_id, callback_message_id, "Loading...").await?;

    // Show sessions for this project
    handle_project_sessions(channel, channel_id, encoded_dir).await
}

async fn handle_model_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    model: &str,
    callback_message_id: Option<i64>,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<super::state::ChannelState>>,
) -> anyhow::Result<()> {
    if !["sonnet", "opus", "haiku"].contains(&model) {
        return dismiss_keyboard(channel, channel_id, callback_message_id, "Unknown model.").await;
    }
    {
        let mut cs = state.write().await;
        cs.set_model(scope, model);
        if let Err(e) = cs.save() {
            tracing::error!(error = %e, "Failed to persist model callback state");
        }
    }
    dismiss_keyboard(
        channel,
        channel_id,
        callback_message_id,
        &format!("Model set to: {}", model),
    )
    .await
}

/// Edit the message to remove inline keyboard buttons (dismiss animation).
async fn dismiss_keyboard(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    callback_message_id: Option<i64>,
    text: &str,
) -> anyhow::Result<()> {
    let Some(msg_id) = callback_message_id else {
        // Fallback: just send a new message
        return reply(channel, channel_id, text).await;
    };
    channel
        .edit_message(&OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: channel_id.clone(),
            text: text.to_string(),
            message_ref: Some(msg_id.to_string()),
            interaction: None,
        })
        .await
}

async fn handle_reply_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    callback_message_id: Option<i64>,
) -> anyhow::Result<()> {
    dismiss_keyboard(
        channel,
        channel_id,
        callback_message_id,
        "Type your response below.",
    )
    .await
}

async fn handle_choice_callback(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    data: &str,
    callback_message_id: Option<i64>,
    app_state: Arc<super::server::AppState>,
) -> anyhow::Result<()> {
    // Reject if Claude is already running
    {
        let active = app_state.active_claude.try_lock();
        match active {
            Ok(guard) if guard.is_some() => {
                return dismiss_keyboard(
                    channel,
                    channel_id,
                    callback_message_id,
                    "Claude is busy — wait for the current response to finish.",
                )
                .await;
            }
            Err(_) => {
                return dismiss_keyboard(
                    channel,
                    channel_id,
                    callback_message_id,
                    "Claude is busy — wait for the current response to finish.",
                )
                .await;
            }
            _ => {} // Lock held, active is None — proceed
        }
    }
    dismiss_keyboard(
        channel,
        channel_id,
        callback_message_id,
        &format!("> {}", data),
    )
    .await?;
    let synthetic = IncomingEvent::TextMessage(TextMessage {
        conversation_id: ConversationId::new(),
        channel: channel_id.clone(),
        text: data.to_string(),
        reply_to_id: None,
    });
    super::server::spawn_process_event(app_state, synthetic);
    Ok(())
}

fn format_time_ago(ts: u64) -> String {
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

fn extract_text(content: &serde_json::Value) -> Option<String> {
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

fn find_session_jsonl(base: &std::path::Path, session_id: &str) -> Option<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(base) else {
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let jsonl = path.join(format!("{}.jsonl", session_id));
        if jsonl.exists() {
            return Some(jsonl);
        }
    }
    None
}
