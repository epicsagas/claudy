use std::collections::HashMap;
use std::sync::Arc;

use crate::config::registry::BridgeSettings;
use crate::domain::channel_events::ChannelIdentity;
use crate::ports::channel_ports::ChannelPort;

use super::super::state::{ChannelState, with_write};
use super::formatting::*;
use super::session_io::*;

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
    scope: &str,
    active_claude: &Arc<tokio::sync::Mutex<HashMap<String, u32>>>,
) -> anyhow::Result<()> {
    let killed = {
        let mut active = active_claude.lock().await;
        match active.remove(scope) {
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
    state: &Arc<tokio::sync::RwLock<ChannelState>>,
) -> anyhow::Result<()> {
    let label = with_write(state, |cs| {
        let next = cs.toggle_yolo(scope);
        if next {
            "Yolo: ON (auto-allow all permissions)"
        } else {
            "Yolo: OFF (ask permission per tool)"
        }
    })
    .await;
    reply(channel, channel_id, label).await
}

pub async fn handle_model(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    args: &str,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<ChannelState>>,
) -> anyhow::Result<()> {
    use crate::domain::channel_events::Button;

    if args.is_empty() {
        reply_with_buttons(
            channel,
            channel_id,
            "Choose a model:".to_string(),
            "Model selection",
            vec![
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
        )
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
            with_write(state, |cs| cs.set_model(scope, args)).await;
        }
        reply(channel, channel_id, &format!("Model set to: {}", args)).await?;
    }
    Ok(())
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
    let Some(projects_dir) = super::super::sessions::claude_projects_dir() else {
        return reply(channel, channel_id, "No Claude projects found.").await;
    };

    let sessions = super::super::sessions::discover_sessions(&projects_dir, 5);
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
    reply_with_buttons(channel, channel_id, text, "Select a session", buttons).await
}

pub async fn handle_projects(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
) -> anyhow::Result<()> {
    let Some(projects_dir) = super::super::sessions::claude_projects_dir() else {
        return reply(channel, channel_id, "No Claude projects found.").await;
    };

    let projects = super::super::sessions::discover_projects(&projects_dir);
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
    reply_with_buttons(channel, channel_id, text, "Select a project", buttons).await
}

pub async fn handle_project_sessions(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    encoded_dir: &str,
) -> anyhow::Result<()> {
    let Some(projects_dir) = super::super::sessions::claude_projects_dir() else {
        return reply(channel, channel_id, "No Claude projects found.").await;
    };

    let sessions = super::super::sessions::discover_project_sessions(&projects_dir, encoded_dir, 5);
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
    reply_with_buttons(channel, channel_id, text, "Select a session", buttons).await
}

pub async fn handle_new(
    channel: &dyn ChannelPort,
    channel_id: &ChannelIdentity,
    scope: &str,
    state: &Arc<tokio::sync::RwLock<ChannelState>>,
) -> anyhow::Result<()> {
    let cwd_info = with_write(state, |cs| {
        cs.clear_session(scope);
        cs.working_dir(scope)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default workspace".to_string())
    })
    .await;

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
    state: &Arc<tokio::sync::RwLock<ChannelState>>,
) -> anyhow::Result<()> {
    let session_id = {
        let cs = state.read().await;
        match cs.session_id(scope) {
            Some(id) => id.to_string(),
            None => return reply(channel, channel_id, "No active session.").await,
        }
    };

    let Some(projects_dir) = super::super::sessions::claude_projects_dir() else {
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
