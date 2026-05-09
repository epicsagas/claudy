use std::sync::Arc;

use crate::domain::channel_events::{
    ChannelIdentity, ConversationId, IncomingEvent, InteractionButtons, InteractionEvent,
    OutboundMessage, TextMessage,
};

use super::{AppState, THINKING_MESSAGES, TypingGuard, is_authorized, spawn_process_event};
use crate::adapters::channel::retry::{RetryPolicy, retry_send};
use crate::adapters::channel::state::scope_key;

/// Validate a stored session ID by checking if the session file still exists on disk.
/// Clears stale session state and returns `None` if the file is missing.
async fn validate_resume_session(
    state: &Arc<AppState>,
    scope: &str,
    session_id: Option<String>,
) -> Option<String> {
    let sid = session_id?;
    let projects_dir = crate::adapters::channel::sessions::claude_projects_dir();
    let found = projects_dir
        .as_ref()
        .is_some_and(|dir| crate::adapters::channel::sessions::session_file_exists(dir, &sid));
    if !found {
        tracing::info!(session_id = %sid, "Stored session not found on disk, starting fresh");
        let mut cs = state.channel_state.write().await;
        cs.clear_session(scope);
        if let Err(e) = cs.save() {
            tracing::error!(error = %e, "Failed to persist cleared session state");
        }
        None
    } else {
        Some(sid)
    }
}

/// Start a Claude subprocess, register its PID for cancellation, and spawn a stderr monitor.
async fn start_claude_and_track(
    state: &Arc<AppState>,
    scope: &str,
    config: &crate::adapters::channel::claude_process::SessionConfig<'_>,
) -> anyhow::Result<crate::adapters::channel::claude_process::ClaudeProcess> {
    let mut claude = crate::adapters::channel::claude_process::start_claude_session(
        &state.paths,
        &state.config,
        &state.secrets,
        &state.catalog,
        config,
    )?;

    // Store child PID for /cancel
    if let Some(pid) = claude.child_id() {
        state
            .active_claude
            .lock()
            .await
            .insert(scope.to_string(), pid);
    }

    // Spawn stderr reader to detect stale session errors
    if let Some(stderr) = claude.take_stderr() {
        let stderr_state = state.channel_state.clone();
        let stderr_scope = scope.to_string();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut reader = tokio::io::BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let trimmed = line.trim();
                        tracing::warn!(stderr = trimmed, "Claude stderr");
                        if trimmed.contains("No conversation found with session ID") {
                            tracing::info!("Clearing stale session due to Claude resume error");
                            let mut cs = stderr_state.write().await;
                            cs.clear_session(&stderr_scope);
                            if let Err(e) = cs.save() {
                                tracing::error!(error = %e, "Failed to clear stale session");
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    Ok(claude)
}

/// Process the streamed response: update message, handle interactive buttons,
/// YOLO auto-continue, and capture session metadata.
async fn process_stream_result(
    state: &Arc<AppState>,
    scope: &str,
    channel: &dyn crate::ports::channel_ports::ChannelPort,
    msg: &TextMessage,
    delivery: &crate::domain::channel_events::MessageDelivery,
    result: crate::adapters::channel::stream_handler::StreamResult,
    yolo: bool,
) -> anyhow::Result<()> {
    // Clear active process for this scope
    state.active_claude.lock().await.remove(scope);

    if !result.has_content {
        let _ = channel
            .edit_message(&OutboundMessage {
                conversation_id: msg.conversation_id.clone(),
                channel: msg.channel.clone(),
                text: "No response".to_string(),
                message_ref: Some(delivery.platform_message_id.clone()),
                interaction: None,
            })
            .await;
    } else if !result.accumulated_text.is_empty() {
        let analysis =
            crate::adapters::channel::response_analyzer::analyze_response(&result.accumulated_text);
        if analysis.needs_interaction {
            let max_len = msg.channel.platform.max_message_length();
            let text = crate::adapters::channel::stream_handler::truncate_message(
                &result.accumulated_text,
                max_len,
            );
            let _ = channel
                .edit_message(&OutboundMessage {
                    conversation_id: msg.conversation_id.clone(),
                    channel: msg.channel.clone(),
                    text,
                    message_ref: Some(delivery.platform_message_id.clone()),
                    interaction: Some(InteractionButtons {
                        prompt_text: "Choose or type your response".into(),
                        buttons: analysis.buttons,
                    }),
                })
                .await;

            if yolo
                && crate::adapters::channel::response_analyzer::is_auto_continuable(
                    &result.accumulated_text,
                )
            {
                schedule_yolo_auto_continue(state, scope, &msg.channel, &msg.conversation_id);
            }
        }
    }

    if let Some(ref sid) = result.session_id {
        let mut cs = state.channel_state.write().await;
        cs.set_session_id(scope, sid);
        if let Some(ref c) = result.cwd {
            cs.set_working_dir(scope, c);
        }
        if let Some(ref b) = result.branch {
            cs.set_branch(scope, b);
        }
        if let Some(ref m) = result.model {
            cs.set_last_model(scope, m);
        }
        if result.input_tokens > 0 || result.output_tokens > 0 {
            cs.add_tokens(scope, result.input_tokens, result.output_tokens);
        }
        if let Err(e) = cs.save() {
            tracing::error!(error = %e, "Failed to persist session capture");
        }
        tracing::info!(session_id = %sid, cwd = ?result.cwd, "Session captured");
    }

    Ok(())
}

fn schedule_yolo_auto_continue(
    state: &Arc<AppState>,
    scope: &str,
    channel_id: &ChannelIdentity,
    conversation_id: &ConversationId,
) {
    let spawn_state = state.clone();
    let ac_state = state.clone();
    let channel_id = channel_id.clone();
    let conversation_id = conversation_id.clone();
    let scope = scope.to_string();
    let handle = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        tracing::info!("YOLO auto-continue: sending 'proceed'");
        let synthetic = IncomingEvent::TextMessage(TextMessage {
            conversation_id,
            channel: channel_id,
            text: "proceed".to_string(),
            reply_to_id: None,
        });
        spawn_process_event(spawn_state, synthetic);
    });
    // Cancel old timer + store new one in a single lock scope
    let ac_state_clone = ac_state.clone();
    std::mem::drop(tokio::spawn(async move {
        let mut ac = ac_state_clone.auto_continue.lock().await;
        if let Some(h) = ac.remove(&scope) {
            h.abort();
        }
        ac.insert(scope, handle);
    }));
}

pub(super) async fn handle_text_message(
    state: &Arc<AppState>,
    msg: TextMessage,
) -> anyhow::Result<()> {
    let platform = msg.channel.platform;
    let channel = state
        .channels
        .get(&platform)
        .ok_or_else(|| anyhow::anyhow!("{platform:?} adapter not registered"))?;

    let scope = scope_key(
        msg.channel.platform.as_str(),
        &msg.channel.channel_id,
        &msg.channel.user_id,
    );

    // Reject if a Claude process is already running for this scope,
    // but clean up stale PIDs where the process has already exited.
    {
        let mut active = state.active_claude.lock().await;
        if let Some(&pid) = active.get(&scope) {
            if is_pid_alive(pid) {
                drop(active);
                let _ = channel
                    .send_message(&OutboundMessage {
                        conversation_id: msg.conversation_id.clone(),
                        channel: msg.channel.clone(),
                        text: "Busy — type /cancel first, then retry.".to_string(),
                        message_ref: None,
                        interaction: None,
                    })
                    .await;
                return Ok(());
            }
            tracing::warn!(pid, scope = %scope, "Cleaning up stale Claude PID");
            active.remove(&scope);
        }
    }

    let profile = state.channel_config.profile_for_channel(
        platform.as_str(),
        &msg.channel.channel_id,
        msg.channel.guild_id.as_deref(),
    );
    let mode = state.channel_config.mode_for_channel(
        platform.as_str(),
        &msg.channel.channel_id,
        msg.channel.guild_id.as_deref(),
    );
    let config_project = state.channel_config.project_for_channel(
        platform.as_str(),
        &msg.channel.channel_id,
        msg.channel.guild_id.as_deref(),
    );

    let _typing = TypingGuard::start(channel.clone(), msg.channel.clone());

    // Read current session state for resume
    let (resume_session, working_dir, model, yolo) = {
        let cs = state.channel_state.read().await;
        let session = cs.session_id(&scope).map(|s| s.to_string());
        let cwd = cs
            .working_dir(&scope)
            .map(|s| s.to_string())
            .or(config_project);
        let m = cs.model(&scope).map(|s| s.to_string());
        let y = cs.yolo(&scope);
        (session, cwd, m, y)
    };

    let resume_session = validate_resume_session(state, &scope, resume_session).await;

    let mut claude = match start_claude_and_track(
        state,
        &scope,
        &crate::adapters::channel::claude_process::SessionConfig {
            profile: &profile,
            mode: mode.as_deref(),
            resume_session: resume_session.as_deref(),
            working_dir: working_dir.as_deref(),
            model: model.as_deref(),
            yolo,
        },
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Failed to start Claude session");
            let err_msg = OutboundMessage {
                conversation_id: msg.conversation_id.clone(),
                channel: msg.channel.clone(),
                text: format!("Failed to start Claude session: {}", e),
                message_ref: None,
                interaction: None,
            };
            let policy = RetryPolicy::for_platform(msg.channel.platform);
            let _ = retry_send(channel.as_ref(), &err_msg, &policy).await;
            return Err(e);
        }
    };

    {
        use tokio::io::AsyncWriteExt;
        if let Some(mut stdin) = claude.take_stdin() {
            stdin.write_all(msg.text.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            // Drop stdin to send EOF — Claude CLI --print mode waits for
            // stdin EOF before processing. Keeping it open causes an
            // indefinite hang.
            drop(stdin);
        }
    }

    let thinking_msg = OutboundMessage {
        conversation_id: msg.conversation_id.clone(),
        channel: msg.channel.clone(),
        text: THINKING_MESSAGES[rand::random_range(0..THINKING_MESSAGES.len())].to_string(),
        message_ref: None,
        interaction: None,
    };
    let policy = RetryPolicy::for_platform(msg.channel.platform);
    let delivery = retry_send(channel.as_ref(), &thinking_msg, &policy).await?;

    let stdout = claude.stdout();
    let stream_result = crate::adapters::channel::stream_handler::stream_response(
        stdout,
        channel.as_ref(),
        &msg.channel,
        &delivery.platform_message_id,
        state.channel_config.stream_timeout_secs,
    )
    .await;

    match stream_result {
        Ok(result) => {
            process_stream_result(
                state,
                &scope,
                channel.as_ref(),
                &msg,
                &delivery,
                result,
                yolo,
            )
            .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Stream error");
            state.active_claude.lock().await.remove(&scope);
            let err_msg = OutboundMessage {
                conversation_id: msg.conversation_id.clone(),
                channel: msg.channel.clone(),
                text: format!("Error: {}", e),
                message_ref: None,
                interaction: None,
            };
            let policy = RetryPolicy::for_platform(msg.channel.platform);
            let _ = retry_send(channel.as_ref(), &err_msg, &policy).await;
            return Err(e);
        }
    }

    Ok(())
}

pub(super) async fn handle_interaction(
    state: &Arc<AppState>,
    inter: InteractionEvent,
) -> anyhow::Result<()> {
    let platform = inter.channel.platform;

    // Defense-in-depth: authorize_and_spawn checks at the entry point,
    // but reject here if somehow bypassed.
    if !is_authorized(state, platform, &inter.channel.user_id) {
        tracing::warn!(
            user_id = %inter.channel.user_id,
            "Interaction reached handler without authorization"
        );
        return Ok(());
    }

    let channel = state
        .channels
        .get(&platform)
        .ok_or_else(|| anyhow::anyhow!("{platform:?} adapter not registered"))?;

    // Acknowledge the callback immediately (stops loading spinner)
    if let Some(ref query_id) = inter.callback_query_id
        && let Err(e) = channel.ack_interaction(&inter.channel, query_id).await
    {
        tracing::warn!(error = %e, "Failed to ack callback (may have expired)");
    }

    let action = &inter.action_id;
    if action.starts_with("allow") || action.starts_with("deny") {
        // Permission buttons are no longer interactive (stdin is closed
        // for --print mode). Ack and ignore.
        return Ok(());
    }

    let scope = scope_key(
        inter.channel.platform.as_str(),
        &inter.channel.channel_id,
        &inter.channel.user_id,
    );

    // Cancel auto-continue timer on any button press
    {
        let mut ac = state.auto_continue.lock().await;
        if let Some(h) = ac.remove(&scope) {
            h.abort();
        }
    }

    crate::adapters::channel::commands::handle_callback(
        crate::adapters::channel::commands::CallbackContext {
            channel: channel.clone(),
            channel_id: inter.channel.clone(),
            action: inter.action_id.clone(),
            data: inter.message_ref.clone(),
            callback_message_id: inter.callback_message_id,
            original_text: inter.original_text,
            scope,
            channel_state: state.channel_state.clone(),
            app_state: state.clone(),
        },
    )
    .await
}

pub(super) async fn handle_bot_command(
    state: &Arc<AppState>,
    command: &str,
    args: &str,
    bot_channel: ChannelIdentity,
) -> anyhow::Result<()> {
    let platform = bot_channel.platform;
    let adapter = state
        .channels
        .get(&platform)
        .ok_or_else(|| anyhow::anyhow!("{platform:?} adapter not registered"))?;

    let scope = scope_key(
        bot_channel.platform.as_str(),
        &bot_channel.channel_id,
        &bot_channel.user_id,
    );

    match command {
        "/help" | "/start" => {
            crate::adapters::channel::commands::handle_help(adapter.as_ref(), &bot_channel).await
        }
        "/cancel" => {
            crate::adapters::channel::commands::handle_cancel(
                adapter.as_ref(),
                &bot_channel,
                &scope,
                &state.active_claude,
            )
            .await
        }
        "/yolo" => {
            crate::adapters::channel::commands::handle_yolo(
                adapter.as_ref(),
                &bot_channel,
                &scope,
                &state.channel_state,
            )
            .await
        }
        "/model" => {
            crate::adapters::channel::commands::handle_model(
                adapter.as_ref(),
                &bot_channel,
                args,
                &scope,
                &state.channel_state,
            )
            .await
        }
        "/status" => {
            let cs = state.channel_state.read().await;
            let active_session = cs.session_id(&scope);
            let active_cwd = cs.working_dir(&scope);
            let active_model = cs.model(&scope).map(|s| s.to_string());
            let yolo = cs.yolo(&scope);
            let branch = cs.branch(&scope).map(|s| s.to_string());
            let input_tokens = cs.input_tokens(&scope);
            let output_tokens = cs.output_tokens(&scope);
            let last_model = cs.last_model(&scope).map(|s| s.to_string());
            let resolved_project = state.channel_config.project_for_channel(
                platform.as_str(),
                &bot_channel.channel_id,
                bot_channel.guild_id.as_deref(),
            );
            crate::adapters::channel::commands::handle_status(
                adapter.as_ref(),
                &bot_channel,
                &state.channel_config,
                crate::adapters::channel::commands::SessionStatus {
                    session_id: active_session,
                    cwd: active_cwd,
                    model: active_model.as_deref(),
                    yolo,
                    branch: branch.as_deref(),
                    input_tokens,
                    output_tokens,
                    last_model: last_model.as_deref(),
                    project: resolved_project.as_deref(),
                },
            )
            .await
        }
        "/sessions" => {
            crate::adapters::channel::commands::handle_sessions(adapter.as_ref(), &bot_channel)
                .await
        }
        "/projects" => {
            crate::adapters::channel::commands::handle_projects(adapter.as_ref(), &bot_channel)
                .await
        }
        "/new" => {
            crate::adapters::channel::commands::handle_new(
                adapter.as_ref(),
                &bot_channel,
                &scope,
                &state.channel_state,
            )
            .await
        }
        "/history" => {
            crate::adapters::channel::commands::handle_history(
                adapter.as_ref(),
                &bot_channel,
                &scope,
                &state.channel_state,
            )
            .await
        }
        _ => {
            adapter
                .send_message(&OutboundMessage {
                    conversation_id: ConversationId::new(),
                    channel: bot_channel.clone(),
                    text: format!("Unknown command: {}", command),
                    message_ref: None,
                    interaction: None,
                })
                .await?;
            Ok(())
        }
    }
}

/// Check if a process with the given PID is still alive (Unix signal-0 probe).
fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_pid_alive_returns_true_for_current_process() {
        let pid = std::process::id();
        assert!(is_pid_alive(pid), "Current process PID should be alive");
    }

    #[test]
    fn is_pid_alive_returns_false_for_nonexistent_pid() {
        assert!(!is_pid_alive(99_999_999), "Very large PID should not exist");
    }
}
