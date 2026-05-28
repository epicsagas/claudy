use std::sync::Arc;

use crate::domain::channel_events::{
    ChannelIdentity, ConversationId, IncomingEvent, InteractionButtons, InteractionEvent,
    OutboundMessage, Platform, TextMessage,
};

use super::{AppState, THINKING_MESSAGES, TypingGuard, is_authorized, spawn_process_event};
use crate::adapters::channel::retry::{RetryPolicy, retry_send};
use crate::adapters::channel::state::{scope_key, with_write};

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
                        if trimmed.contains("No conversation found with session ID")
                            || trimmed.contains("Invalid `signature` in `thinking` block")
                            || trimmed.contains("Invalid signature in thinking block")
                        {
                            tracing::info!(
                                stderr = trimmed,
                                "Clearing session due to resume incompatibility"
                            );
                            let mut cs = stderr_state.write().await;
                            cs.clear_session(&stderr_scope);
                            if let Err(e) = cs.save() {
                                tracing::error!(error = %e, "Failed to clear session after resume error");
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

    // Handle "waiting for directory input" state (from New project flow)
    {
        let waiting = {
            let cs = state.channel_state.read().await;
            cs.waiting_for_dir(&scope)
        };
        if waiting {
            let path = msg.text.trim();
            if std::path::Path::new(path).is_dir() {
                with_write(&state.channel_state, |cs| {
                    cs.set_working_dir(&scope, path);
                    cs.clear_session(&scope);
                    cs.clear_waiting_for_dir(&scope);
                })
                .await;
                let display = std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string());
                let _ = channel
                    .send_message(&OutboundMessage {
                        conversation_id: msg.conversation_id.clone(),
                        channel: msg.channel.clone(),
                        text: format!("New session started.\nProject: {}", display),
                        message_ref: None,
                        interaction: None,
                    })
                    .await;
            } else {
                let _ = channel
                    .send_message(&OutboundMessage {
                        conversation_id: msg.conversation_id.clone(),
                        channel: msg.channel.clone(),
                        text: "Directory not found. Try again or type /cancel.".to_string(),
                        message_ref: None,
                        interaction: None,
                    })
                    .await;
            }
            return Ok(());
        }
    }

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

    // Strip thinking blocks with empty/invalid signatures written by non-Anthropic
    // providers (e.g. ZAI/GLM). The Anthropic API rejects these with HTTP 400 when
    // the session is resumed. Sanitization is a no-op when the file is already clean.
    if let (Some(sid), Some(projects_dir)) = (
        resume_session.as_deref(),
        crate::adapters::channel::sessions::claude_projects_dir(),
    ) {
        match crate::adapters::channel::sessions::sanitize_session_thinking_blocks(
            &projects_dir,
            sid,
        ) {
            Ok(0) => {}
            Ok(n) => tracing::info!(
                count = n,
                session_id = %sid,
                "Stripped invalid thinking blocks before resume"
            ),
            Err(e) => tracing::warn!(
                error = %e,
                session_id = %sid,
                "Could not sanitize thinking blocks; resume may fail"
            ),
        }
    }

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
            // R1/R3: If context window limit was detected, attempt auto-compact
            let ctx_limit = result.context_limit.clone();
            if let Some(ref ctx_err) = ctx_limit {
                tracing::warn!(
                    error = %ctx_err.message,
                    "Context window limit detected — attempting auto-compact recovery"
                );
                // Skip process_stream_result — sending the error text to the channel
                // before the recovery message would confuse the user. Just clean up
                // the PID and proceed to recovery.
                state.active_claude.lock().await.remove(&scope);

                return handle_context_limit_recovery(state, &scope, channel.as_ref(), &msg, yolo)
                    .await;
            }

            // Clear recovery flag before processing the result — even if
            // process_stream_result fails, the recovery cycle is over and the
            // flag should not block future auto-recovery attempts.
            {
                let cs = state.channel_state.read().await;
                let is_recovery = cs.get(&scope, "AUTO_COMPACT_TRIGGERED") == Some("true");
                if is_recovery {
                    drop(cs);
                    with_write(&state.channel_state, |cs| {
                        cs.set(&scope, "AUTO_COMPACT_TRIGGERED", "false");
                    })
                    .await;
                }
            }

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
            let err_str = e.to_string();
            // R1: Detect context limit in generic stream errors too
            if crate::adapters::channel::stream_handler::is_context_limit_error(&err_str) {
                tracing::warn!(
                    error = %err_str,
                    "Context window limit in stream error — attempting recovery"
                );
                state.active_claude.lock().await.remove(&scope);

                return handle_context_limit_recovery(state, &scope, channel.as_ref(), &msg, yolo)
                    .await;
            }

            // Non-context-limit error: clear the recovery flag so future
            // context-limit hits can trigger auto-recovery again.
            {
                let cs = state.channel_state.read().await;
                let is_recovery = cs.get(&scope, "AUTO_COMPACT_TRIGGERED") == Some("true");
                if is_recovery {
                    drop(cs);
                    with_write(&state.channel_state, |cs| {
                        cs.set(&scope, "AUTO_COMPACT_TRIGGERED", "false");
                    })
                    .await;
                }
            }

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
        "/start" => {
            // Set up Telegram persistent bottom keyboard
            if platform == Platform::Telegram
                && let Some(tg) = adapter
                    .as_any()
                    .downcast_ref::<super::super::telegram::TelegramAdapter>()
                && let Err(e) = tg.send_reply_keyboard(&bot_channel.channel_id).await
            {
                tracing::warn!(error = %e, "Failed to send Telegram reply keyboard");
            }
            crate::adapters::channel::commands::handle_help(adapter.as_ref(), &bot_channel).await
        }
        "/help" => {
            crate::adapters::channel::commands::handle_help(adapter.as_ref(), &bot_channel).await
        }
        "/cancel" | "/stop" => {
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
        "/compact" => {
            let has_session = {
                let cs = state.channel_state.read().await;
                cs.session_id(&scope).is_some()
            };
            if !has_session {
                crate::adapters::channel::commands::handle_help(adapter.as_ref(), &bot_channel)
                    .await?;
                return Ok(());
            }
            // R2: Use dedicated compact path that provides user feedback
            let (session_id, working_dir, model, yolo) = {
                let cs = state.channel_state.read().await;
                let sid = cs.session_id(&scope).map(|s| s.to_string());
                let cwd = cs.working_dir(&scope).map(|s| s.to_string());
                let m = cs.model(&scope).map(|s| s.to_string());
                let y = cs.yolo(&scope);
                (sid, cwd, m, y)
            };

            if let Some(ref sid) = session_id {
                let before_tokens = {
                    let cs = state.channel_state.read().await;
                    cs.input_tokens(&scope)
                };

                let result = run_compact_command(CompactParams {
                    state,
                    scope: &scope,
                    channel: adapter.as_ref(),
                    channel_id: &bot_channel,
                    session_id: sid,
                    working_dir: working_dir.as_deref(),
                    model: model.as_deref(),
                    yolo,
                })
                .await;

                match result {
                    Ok(Some(r)) => {
                        let after_tokens = r.input_tokens;
                        // Update session state
                        if let Some(ref new_sid) = r.session_id {
                            with_write(&state.channel_state, |cs| {
                                cs.set_session_id(&scope, new_sid);
                                if r.input_tokens > 0 || r.output_tokens > 0 {
                                    cs.add_tokens(&scope, r.input_tokens, r.output_tokens);
                                }
                                cs.set(&scope, "AUTO_COMPACT_TRIGGERED", "false");
                            })
                            .await;
                        }
                        let text = match (before_tokens, after_tokens) {
                            (0, 0) => "Compaction complete.".to_string(),
                            (b, a) => format!("Compaction complete ({} -> {} tokens).", b, a),
                        };
                        let msg = OutboundMessage {
                            conversation_id: ConversationId::new(),
                            channel: bot_channel.clone(),
                            text,
                            message_ref: None,
                            interaction: None,
                        };
                        let policy = RetryPolicy::for_platform(bot_channel.platform);
                        if let Err(e) = retry_send(adapter.as_ref(), &msg, &policy).await {
                            tracing::warn!(error = %e, "Failed to send compact success message");
                        }
                    }
                    Ok(None) | Err(_) => {
                        let msg = OutboundMessage {
                            conversation_id: ConversationId::new(),
                            channel: bot_channel.clone(),
                            text: "Compaction failed. Try /new to start a fresh session."
                                .to_string(),
                            message_ref: None,
                            interaction: None,
                        };
                        let policy = RetryPolicy::for_platform(bot_channel.platform);
                        if let Err(e) = retry_send(adapter.as_ref(), &msg, &policy).await {
                            tracing::warn!(error = %e, "Failed to send compact failure message");
                        }
                    }
                }
            }
            Ok(())
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

/// Check if a process with the given PID is still alive (signal-0 probe / OpenProcess).
/// Returns `true` if the process exists (including the EPERM case where it is
/// alive but owned by another user). Returns `false` only on ESRCH (no such process).
#[cfg(unix)]
fn is_pid_alive(pid: u32) -> bool {
    unsafe {
        if libc::kill(pid as i32, 0) == 0 {
            return true;
        }
        // EPERM means the process exists but we lack permission to signal it.
        std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
    }
}

#[cfg(windows)]
fn is_pid_alive(pid: u32) -> bool {
    std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}

/// R3/R4/R5: Attempt recovery from a context window limit error.
///
/// Strategy:
/// 1. If recovery was already attempted, report failure (prevents infinite loops)
/// 2. Try sending `/compact` to reduce context, then replay the original message
/// 3. If compaction fails, start a completely new session and replay
async fn handle_context_limit_recovery(
    state: &Arc<AppState>,
    scope: &str,
    channel: &dyn crate::ports::channel_ports::ChannelPort,
    original_msg: &TextMessage,
    yolo: bool,
) -> anyhow::Result<()> {
    let already_attempted = {
        let cs = state.channel_state.read().await;
        cs.get(scope, "AUTO_COMPACT_TRIGGERED") == Some("true")
    };

    if already_attempted {
        tracing::warn!("Recovery already attempted — reporting failure to user");
        with_write(&state.channel_state, |cs| {
            cs.clear_session(scope);
            cs.set(scope, "AUTO_COMPACT_TRIGGERED", "false");
        })
        .await;

        let fallback_msg = OutboundMessage {
            conversation_id: original_msg.conversation_id.clone(),
            channel: original_msg.channel.clone(),
            text: "Context recovery failed. New session started — please resend your message."
                .to_string(),
            message_ref: None,
            interaction: None,
        };
        let policy = RetryPolicy::for_platform(original_msg.channel.platform);
        if let Err(e) = retry_send(channel, &fallback_msg, &policy).await {
            tracing::warn!(error = %e, "Failed to send recovery failure message");
        }

        return Ok(());
    }

    // Mark recovery as in-progress
    with_write(&state.channel_state, |cs| {
        cs.set(scope, "AUTO_COMPACT_TRIGGERED", "true");
    })
    .await;

    // Notify user that compaction is being attempted
    let compacting_msg = OutboundMessage {
        conversation_id: original_msg.conversation_id.clone(),
        channel: original_msg.channel.clone(),
        text: "Context limit reached. Compacting conversation...".to_string(),
        message_ref: None,
        interaction: None,
    };
    let policy = RetryPolicy::for_platform(original_msg.channel.platform);
    if let Err(e) = retry_send(channel, &compacting_msg, &policy).await {
        tracing::warn!(error = %e, "Failed to send compacting notification");
    }

    // Get current session info for resume
    let (resume_session, working_dir, model) = {
        let cs = state.channel_state.read().await;
        let session = cs.session_id(scope).map(|s| s.to_string());
        let cwd = cs.working_dir(scope).map(|s| s.to_string());
        let m = cs.model(scope).map(|s| s.to_string());
        (session, cwd, m)
    };

    let resume_session = validate_resume_session(state, scope, resume_session).await;

    // Phase 1: Try compaction
    if let Some(ref sid) = resume_session {
        let compact_result = run_compact_command(CompactParams {
            state,
            scope,
            channel,
            channel_id: &original_msg.channel,
            session_id: sid,
            working_dir: working_dir.as_deref(),
            model: model.as_deref(),
            yolo,
        })
        .await;

        match compact_result {
            Ok(Some(result)) => {
                return update_and_replay_after_compact(
                    state,
                    scope,
                    channel,
                    original_msg,
                    &result,
                    &policy,
                )
                .await;
            }
            Ok(None) => {
                tracing::warn!("Compact produced no result — falling back to new session");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Compact failed — falling back to new session");
            }
        }
    }

    // Phase 2: Fallback — new session + replay
    start_fresh_session_and_replay(state, scope, channel, original_msg).await
}

/// Update session state after a successful compaction, then replay the original
/// message in the compacted session.
async fn update_and_replay_after_compact(
    state: &Arc<AppState>,
    scope: &str,
    channel: &dyn crate::ports::channel_ports::ChannelPort,
    original_msg: &TextMessage,
    compact_result: &crate::adapters::channel::stream_handler::StreamResult,
    policy: &RetryPolicy,
) -> anyhow::Result<()> {
    let before_tokens = {
        let cs = state.channel_state.read().await;
        cs.input_tokens(scope)
    };
    let after_tokens = compact_result.input_tokens;

    let text = match (before_tokens, after_tokens) {
        (0, 0) => "Compaction complete. Retrying your message...".to_string(),
        (b, a) => format!(
            "Compaction complete ({} -> {} tokens). Retrying your message...",
            b, a
        ),
    };
    let success_msg = OutboundMessage {
        conversation_id: original_msg.conversation_id.clone(),
        channel: original_msg.channel.clone(),
        text,
        message_ref: None,
        interaction: None,
    };
    if let Err(e) = retry_send(channel, &success_msg, policy).await {
        tracing::warn!(error = %e, "Failed to send compact success notification");
    }

    if let Some(ref new_sid) = compact_result.session_id {
        // DO NOT reset AUTO_COMPACT_TRIGGERED here — keep it "true" so that if
        // the replayed message also hits the context limit, the guard in
        // handle_context_limit_recovery catches it and falls back to a new
        // session instead of looping. The flag is cleared only after a
        // successful non-recovery message completes (manual /compact or the
        // replayed message succeeding without context-limit errors).
        with_write(&state.channel_state, |cs| {
            cs.set_session_id(scope, new_sid);
            if compact_result.input_tokens > 0 || compact_result.output_tokens > 0 {
                cs.add_tokens(
                    scope,
                    compact_result.input_tokens,
                    compact_result.output_tokens,
                );
            }
        })
        .await;
    } else {
        // Compact produced no new session ID — replaying into the old
        // context-limited session would be pointless. Fall back to fresh session.
        tracing::warn!(
            "Compact succeeded but produced no session ID — falling back to new session"
        );
        return start_fresh_session_and_replay(state, scope, channel, original_msg).await;
    }

    // Box to break the recursive async cycle:
    // handle_text_message → handle_context_limit_recovery → this fn → handle_text_message
    Box::pin(handle_text_message(
        state,
        TextMessage {
            conversation_id: original_msg.conversation_id.clone(),
            channel: original_msg.channel.clone(),
            text: original_msg.text.clone(),
            reply_to_id: None,
        },
    ))
    .await
}

/// Clear the current session and replay the original message in a fresh one.
async fn start_fresh_session_and_replay(
    state: &Arc<AppState>,
    scope: &str,
    channel: &dyn crate::ports::channel_ports::ChannelPort,
    original_msg: &TextMessage,
) -> anyhow::Result<()> {
    tracing::info!("Starting new session as context limit fallback");
    with_write(&state.channel_state, |cs| {
        cs.clear_session(scope);
    })
    .await;

    let fallback_msg = OutboundMessage {
        conversation_id: original_msg.conversation_id.clone(),
        channel: original_msg.channel.clone(),
        text: "Context could not be recovered. New session started — retrying your message..."
            .to_string(),
        message_ref: None,
        interaction: None,
    };
    let policy = RetryPolicy::for_platform(original_msg.channel.platform);
    if let Err(e) = retry_send(channel, &fallback_msg, &policy).await {
        tracing::warn!(error = %e, "Failed to send new session fallback message");
    }

    // Box to break the recursive async cycle:
    // handle_text_message → handle_context_limit_recovery → this fn → handle_text_message
    Box::pin(handle_text_message(
        state,
        TextMessage {
            conversation_id: original_msg.conversation_id.clone(),
            channel: original_msg.channel.clone(),
            text: original_msg.text.clone(),
            reply_to_id: None,
        },
    ))
    .await
}

/// Parameters for launching a compaction command against an existing session.
struct CompactParams<'a> {
    state: &'a Arc<AppState>,
    scope: &'a str,
    channel: &'a dyn crate::ports::channel_ports::ChannelPort,
    channel_id: &'a ChannelIdentity,
    session_id: &'a str,
    working_dir: Option<&'a str>,
    model: Option<&'a str>,
    yolo: bool,
}

/// Run a `/compact` command against an existing session by launching a new
/// Claude process with `--resume` and sending the compact text.
async fn run_compact_command(
    params: CompactParams<'_>,
) -> anyhow::Result<Option<crate::adapters::channel::stream_handler::StreamResult>> {
    // Kill any active process for this scope first
    {
        let mut active = params.state.active_claude.lock().await;
        if let Some(pid) = active.remove(params.scope) {
            let _ = tokio::process::Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output()
                .await;
        }
    }

    let profile = params.state.channel_config.profile_for_channel(
        params.channel_id.platform.as_str(),
        &params.channel_id.channel_id,
        params.channel_id.guild_id.as_deref(),
    );
    let mode = params.state.channel_config.mode_for_channel(
        params.channel_id.platform.as_str(),
        &params.channel_id.channel_id,
        params.channel_id.guild_id.as_deref(),
    );

    let mut claude = start_claude_and_track(
        params.state,
        params.scope,
        &crate::adapters::channel::claude_process::SessionConfig {
            profile: &profile,
            mode: mode.as_deref(),
            resume_session: Some(params.session_id),
            working_dir: params.working_dir,
            model: params.model,
            yolo: params.yolo,
        },
    )
    .await?;

    // Send /compact as the prompt text
    {
        use tokio::io::AsyncWriteExt;
        let mut stdin = claude
            .take_stdin()
            .ok_or_else(|| anyhow::anyhow!("Claude stdin unavailable — cannot send /compact"))?;
        stdin.write_all(b"/compact\n").await?;
        drop(stdin);
    }

    let stdout = claude.stdout();
    let result = crate::adapters::channel::stream_handler::stream_response(
        stdout,
        params.channel,
        params.channel_id,
        "compact",
        params.state.channel_config.stream_timeout_secs,
    )
    .await;

    // Clean up PID
    params.state.active_claude.lock().await.remove(params.scope);

    match result {
        Ok(r) => Ok(Some(r)),
        Err(e) => {
            tracing::warn!(error = %e, "Compact stream failed");
            Ok(None)
        }
    }
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
