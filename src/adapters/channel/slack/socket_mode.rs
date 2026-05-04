use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio_tungstenite::tungstenite::Message;

use crate::domain::channel_events::Platform;
use crate::adapters::channel::server::{AppState, is_authorized, spawn_process_event};

#[derive(Deserialize, Debug)]
struct SocketModeMessage {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    envelope_id: Option<String>,
    payload: Option<serde_json::Value>,
    #[serde(default)]
    #[expect(dead_code, reason = "deserialized from Slack envelope but not needed for ack")]
    accepts_response_payload: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct ConnectionsOpenResponse {
    ok: bool,
    url: Option<String>,
    error: Option<String>,
}

pub async fn start_slack_socket_mode(
    state: Arc<AppState>,
    _bot_token: String,
    app_token: String,
) {
    loop {
        if let Err(e) = run_socket_session(&state, &app_token).await {
            tracing::error!(error = %e, "Slack Socket Mode error, reconnecting in 5s");
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn run_socket_session(
    state: &Arc<AppState>,
    app_token: &str,
) -> anyhow::Result<()> {
    // Get WebSocket URL from Slack
    let ws_url = get_socket_url(app_token).await?;
    tracing::info!("Slack Socket Mode connecting");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url).await
        .map_err(|e| anyhow::anyhow!("WebSocket connect failed: {}", e))?;

    tracing::info!("Slack Socket Mode WebSocket connected");

    while let Some(msg) = ws.next().await {
        let msg = msg.map_err(|e| anyhow::anyhow!("WebSocket error: {}", e))?;
        let text = match &msg {
            Message::Text(t) => t.as_ref(),
            Message::Close(_) => {
                tracing::info!("Slack Socket Mode WebSocket closed");
                return Ok(());
            }
            Message::Ping(data) => {
                let _ = ws.send(Message::Pong(data.clone())).await;
                continue;
            }
            _ => continue,
        };

        let envelope: SocketModeMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to parse Socket Mode message");
                continue;
            }
        };

        // Ack the envelope
        if let Some(ref eid) = envelope.envelope_id {
            let ack = serde_json::json!({ "envelope_id": eid });
            let ack_text = serde_json::to_string(&ack).unwrap_or_default();
            let _ = ws.send(Message::Text(ack_text.into())).await;
        }

        let msg_type = envelope.msg_type.as_deref().unwrap_or("");

        match msg_type {
            "hello" => {
                tracing::info!("Slack Socket Mode hello received");
            }
            "events_api" => {
                if let Some(payload) = envelope.payload {
                    handle_events_api(state, &payload).await;
                }
            }
            "interactive" => {
                if let Some(payload) = envelope.payload {
                    handle_interactive(state, &payload).await;
                }
            }
            "disconnect" => {
                tracing::info!("Slack Socket Mode disconnect requested");
                return Ok(());
            }
            _ => {
                tracing::debug!(msg_type, "Unhandled Socket Mode message type");
            }
        }
    }

    Ok(())
}

async fn get_socket_url(app_token: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let resp: ConnectionsOpenResponse = client
        .post("https://slack.com/api/apps.connections.open")
        .header("Authorization", format!("Bearer {}", app_token))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("")
        .send()
        .await?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse connections.open response: {}", e))?;

    if !resp.ok {
        anyhow::bail!(
            "Slack apps.connections.open failed: {}",
            resp.error.unwrap_or_else(|| "unknown error".to_string())
        );
    }

    resp.url.ok_or_else(|| anyhow::anyhow!("No URL in connections.open response"))
}

async fn handle_events_api(state: &Arc<AppState>, payload: &serde_json::Value) {
    // The payload from Socket Mode wraps the event in an "event" field
    // that matches the SlackEventCallback structure
    let callback: super::normalize::SlackEventCallback = match serde_json::from_value(payload.clone()) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse Slack event callback");
            return;
        }
    };

    if let Some(event) = super::normalize::normalize_event(&callback) {
        let user_id = match &event {
            crate::domain::channel_events::IncomingEvent::TextMessage(msg) => &msg.channel.user_id,
            _ => return,
        };
        if !is_authorized(state, Platform::Slack, user_id) {
            tracing::warn!(user_id, "Unauthorized Slack user");
            return;
        }
        spawn_process_event(state.clone(), event);
    }
}

async fn handle_interactive(state: &Arc<AppState>, payload: &serde_json::Value) {
    let interaction: super::normalize::SlackInteractionPayload =
        match serde_json::from_value(payload.clone()) {
            Ok(i) => i,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to parse Slack interaction payload");
                return;
            }
        };

    if let Some(event) = super::normalize::normalize_interaction(&interaction) {
        let user_id = match &event {
            crate::domain::channel_events::IncomingEvent::Interaction(inter) => &inter.channel.user_id,
            _ => return,
        };
        if !is_authorized(state, Platform::Slack, user_id) {
            tracing::warn!(user_id, "Unauthorized Slack user");
            return;
        }
        spawn_process_event(state.clone(), event);
    }
}
