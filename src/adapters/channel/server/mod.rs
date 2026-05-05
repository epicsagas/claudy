mod event_dispatch;
mod polling;
mod webhook_handlers;

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::layout::AppPaths;
use crate::config::registry::{AppRegistry, BridgeSettings};
use crate::config::vault::SecretVault;
use crate::domain::channel_events::{ChannelIdentity, IncomingEvent, Platform};
use crate::domain::context::Context;
use crate::ports::channel_ports::ChannelPort;
use crate::providers::index::ProviderIndex;

use super::pid;
use super::session::InMemorySessionStore;
use super::state::ChannelState;

/// Periodically sends typing indicators while Claude is processing.
/// Drop to cancel. Telegram typing expires after ~5s, Discord after ~10s.
pub(super) struct TypingGuard {
    tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl TypingGuard {
    pub(super) fn start(channel: Arc<dyn ChannelPort>, channel_id: ChannelIdentity) -> Self {
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let mut rx = std::pin::pin!(rx);
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(4)) => {
                        let _ = channel.send_typing(&channel_id).await;
                    }
                    _ = &mut rx => break,
                }
            }
        });
        Self { tx: Some(tx) }
    }
}

impl Drop for TypingGuard {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(());
        }
    }
}

pub type ChannelRegistry = HashMap<Platform, Arc<dyn ChannelPort>>;

pub(super) const THINKING_MESSAGES: &[&str] = &[
    "Thinking...",
    "Let me work on that...",
    "Processing...",
    "One moment...",
    "On it...",
    "Give me a second...",
    "Analyzing...",
    "Looking into it...",
    "Crunching...",
    "Almost there...",
];

pub struct AppState {
    pub channels: ChannelRegistry,
    pub sessions: Arc<InMemorySessionStore>,
    pub channel_config: BridgeSettings,
    pub paths: AppPaths,
    pub config: AppRegistry,
    pub secrets: SecretVault,
    pub catalog: ProviderIndex,
    pub channel_state: Arc<RwLock<ChannelState>>,
    /// Active Claude child PIDs keyed by scope for per-channel cancellation.
    pub active_claude: Arc<tokio::sync::Mutex<HashMap<String, u32>>>,
    /// Auto-continue timer handles (YOLO mode) keyed by scope.
    pub auto_continue: Arc<tokio::sync::Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

fn init_tracing(logs_dir: &str) {
    let log_path = std::path::Path::new(logs_dir);
    let _ = std::fs::create_dir_all(log_path);

    let file_appender = tracing_appender::rolling::daily(log_path, "server.log");
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false);

    let stdout_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();
}

pub async fn run(ctx: &Context, listen_addr: &str) -> anyhow::Result<i32> {
    init_tracing(&ctx.paths.channel_logs_dir);

    let sessions = Arc::new(InMemorySessionStore::new());
    let state_path = format!("{}/state", ctx.paths.channel_dir);
    let channel_state = Arc::new(RwLock::new(ChannelState::load(&state_path)));

    let mut channels = ChannelRegistry::new();
    if let Some(token) = ctx.secrets.get("TELEGRAM_BOT_TOKEN")
        && !token.is_empty()
    {
        tracing::info!("Registering Telegram adapter");
        channels.insert(
            Platform::Telegram,
            Arc::new(super::telegram::TelegramAdapter::new(token.clone())),
        );
    }

    if let Some(token) = ctx.secrets.get("SLACK_BOT_TOKEN")
        && !token.is_empty()
    {
        tracing::info!("Registering Slack adapter");
        channels.insert(
            Platform::Slack,
            Arc::new(super::slack::SlackAdapter::new(token.clone())),
        );
    }

    if let Some(token) = ctx.secrets.get("DISCORD_BOT_TOKEN")
        && !token.is_empty()
    {
        tracing::info!("Registering Discord adapter");
        channels.insert(
            Platform::Discord,
            Arc::new(super::discord::DiscordAdapter::new(token.clone())),
        );
    }

    let state = Arc::new(AppState {
        channels,
        sessions,
        channel_config: ctx.config.channel.clone(),
        paths: ctx.paths.clone(),
        config: ctx.config.clone(),
        secrets: ctx.secrets.clone(),
        catalog: ctx.catalog.clone(),
        channel_state,
        active_claude: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        auto_continue: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route(
            "/webhook/telegram",
            post(webhook_handlers::telegram_webhook),
        )
        .route("/webhook/slack", post(webhook_handlers::slack_webhook))
        .route("/webhook/discord", post(webhook_handlers::discord_webhook))
        .with_state(state.clone());

    if let Some(token) = ctx.secrets.get("TELEGRAM_BOT_TOKEN")
        && !token.is_empty()
    {
        {
            let api = super::telegram::api::TelegramApi::new(token.clone());
            if let Err(e) = api
                .set_my_commands(&[
                    ("help", "Show available commands"),
                    ("cancel", "Cancel current task"),
                    ("model", "Change Claude model"),
                    ("yolo", "Toggle auto-allow permissions"),
                    ("status", "Show session status"),
                    ("sessions", "List recent sessions"),
                    ("projects", "List projects"),
                    ("new", "Start new session"),
                    ("history", "Show session history"),
                ])
                .await
            {
                tracing::warn!(error = %e, "Failed to register bot commands");
            }
        }

        let poll_state = state.clone();
        let poll_token = token.clone();
        tokio::spawn(async move {
            polling::start_telegram_polling(poll_state, poll_token).await;
        });
    }

    if let Some(token) = ctx.secrets.get("DISCORD_BOT_TOKEN")
        && !token.is_empty()
    {
        {
            use super::discord::api::{CommandDefinition, CommandOption, DiscordApi};
            let api = DiscordApi::new(token.clone());
            match api.get_application_id().await {
                Ok(app_id) => {
                    let commands = &[
                        CommandDefinition {
                            name: "help",
                            description: "Show available commands",
                            options: vec![],
                        },
                        CommandDefinition {
                            name: "cancel",
                            description: "Cancel current task",
                            options: vec![],
                        },
                        CommandDefinition {
                            name: "model",
                            description: "Change Claude model",
                            options: vec![CommandOption {
                                name: "name",
                                description: "Model name",
                                kind: 3,
                                required: false,
                            }],
                        },
                        CommandDefinition {
                            name: "yolo",
                            description: "Toggle auto-allow permissions",
                            options: vec![],
                        },
                        CommandDefinition {
                            name: "status",
                            description: "Show session status",
                            options: vec![],
                        },
                        CommandDefinition {
                            name: "sessions",
                            description: "List recent sessions",
                            options: vec![],
                        },
                        CommandDefinition {
                            name: "projects",
                            description: "List projects",
                            options: vec![],
                        },
                        CommandDefinition {
                            name: "new",
                            description: "Start new session",
                            options: vec![],
                        },
                        CommandDefinition {
                            name: "history",
                            description: "Show session history",
                            options: vec![],
                        },
                    ];
                    if let Err(e) = api.register_application_commands(&app_id, commands).await {
                        tracing::warn!(error = %e, "Failed to register Discord slash commands");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to get Discord application ID; skipping command registration");
                }
            }
        }

        let gw_state = state.clone();
        let gw_token = token.clone();
        tokio::spawn(async move {
            super::discord::gateway::start_discord_gateway(gw_state, gw_token).await;
        });
    }

    if let Some(bot_token) = ctx.secrets.get("SLACK_BOT_TOKEN")
        && !bot_token.is_empty()
    {
        if let Some(app_token) = ctx.secrets.get("SLACK_APP_TOKEN")
            && !app_token.is_empty()
        {
            let sm_state = state.clone();
            let sm_bot_token = bot_token.clone();
            let sm_app_token = app_token.clone();
            tokio::spawn(async move {
                super::slack::socket_mode::start_slack_socket_mode(
                    sm_state,
                    sm_bot_token,
                    sm_app_token,
                )
                .await;
            });
        } else {
            tracing::warn!(
                "SLACK_BOT_TOKEN set but SLACK_APP_TOKEN missing; skipping Slack Socket Mode"
            );
        }
    }

    ctx.paths.ensure_base_dirs()?;

    tracing::info!(addr = listen_addr, "Channel server starting");

    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    // Write PID after successful bind to avoid stale PID on bind failure
    pid::write(&ctx.paths.channel_pid_file)?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    pid::remove(&ctx.paths.channel_pid_file);
    Ok(0)
}

async fn health_handler() -> &'static str {
    "ok"
}

/// Spawn a synthetic event for processing (used by auto-continue and button callbacks).
pub fn spawn_process_event(state: Arc<AppState>, event: IncomingEvent) {
    tokio::spawn(async move {
        if let Err(e) = process_event(&state, event).await {
            tracing::error!(error = %e, "Synthetic event processing failed");
        }
    });
}

/// Authorize the event's user for the given platform, then spawn async processing.
///
/// Extracts the `user_id` from known event variants (`TextMessage`, `Interaction`,
/// `BotCommand`). Silently ignores variants that carry no user identity (e.g.
/// `Attachment`). Logs a warning and skips processing for unauthorized users.
pub(super) fn authorize_and_spawn(state: &Arc<AppState>, platform: Platform, event: IncomingEvent) {
    let user_id = match &event {
        IncomingEvent::TextMessage(msg) => msg.channel.user_id.as_str(),
        IncomingEvent::Interaction(inter) => inter.channel.user_id.as_str(),
        IncomingEvent::BotCommand { channel, .. } => channel.user_id.as_str(),
        _ => return,
    };
    if !is_authorized(state, platform, user_id) {
        tracing::warn!(user_id, platform = platform.as_str(), "Unauthorized user");
        return;
    }
    spawn_process_event(state.clone(), event);
}

pub async fn process_event(state: &Arc<AppState>, event: IncomingEvent) -> anyhow::Result<()> {
    match event {
        IncomingEvent::TextMessage(msg) => event_dispatch::handle_text_message(state, msg).await,
        IncomingEvent::Interaction(inter) => event_dispatch::handle_interaction(state, inter).await,
        IncomingEvent::BotCommand {
            command,
            args,
            channel,
            ..
        } => event_dispatch::handle_bot_command(state, &command, &args, channel).await,
        IncomingEvent::Attachment(_) => Ok(()),
    }
}

/// Returns `true` when `user_id` is allowed to interact on the given platform.
///
/// Checks `platform_allowed_users` first, falls back to `allowed_users`.
/// If neither list is set for the platform, everyone is permitted.
pub(super) fn is_authorized(state: &AppState, platform: Platform, user_id: &str) -> bool {
    let users = state.channel_config.allowed_users_for(platform.as_str());
    if users.is_empty() {
        return true;
    }
    users.iter().any(|u| u == user_id)
}

/// Constant-time comparison to prevent timing attacks on secret tokens.
pub(super) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut result: u8 = (a.len() ^ b.len()) as u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Shutdown signal received (ctrl+c)");
        }
        _ = terminate => {
            tracing::info!("Shutdown signal received (SIGTERM)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_equal() {
        assert!(constant_time_eq(b"hello", b"hello"));
    }

    #[test]
    fn constant_time_eq_not_equal() {
        assert!(!constant_time_eq(b"hello", b"world"));
    }

    #[test]
    fn constant_time_eq_different_lengths() {
        assert!(!constant_time_eq(b"short", b"longer"));
    }

    #[test]
    fn constant_time_eq_empty() {
        assert!(constant_time_eq(b"", b""));
    }

    #[tokio::test]
    async fn test_shutdown_signal_does_not_panic_on_cancel() {
        let handle = tokio::spawn(async {
            shutdown_signal().await;
        });

        handle.abort();
        tokio::task::yield_now().await;
    }
}
