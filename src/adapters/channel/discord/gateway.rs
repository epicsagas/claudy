use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::adapters::channel::server::{AppState, authorize_and_spawn};
use crate::domain::channel_events::Platform;

const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";

// Intents: GUILDS | GUILD_MESSAGES | DIRECT_MESSAGES | MESSAGE_CONTENT
const INTENTS: u64 = 1 | 512 | 4096 | 32768;

#[derive(Serialize)]
struct GatewayPayload {
    op: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    d: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    t: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GatewayEvent {
    op: u64,
    #[serde(default)]
    d: serde_json::Value,
    #[serde(default)]
    s: Option<u64>,
    #[serde(default)]
    t: Option<String>,
}

#[derive(Deserialize, Debug)]
struct HelloData {
    heartbeat_interval: u64,
}

#[derive(Deserialize, Debug)]
struct ReadyData {
    session_id: String,
    #[serde(default)]
    resume_gateway_url: Option<String>,
}

pub async fn start_discord_gateway(state: Arc<AppState>, token: String) {
    let mut session_id: Option<String> = None;
    let mut resume_url: Option<String> = None;
    let mut last_seq: Option<u64> = None;

    loop {
        let url = resume_url.as_deref().unwrap_or(GATEWAY_URL);
        let result =
            run_gateway_session(&state, &token, url, session_id.as_deref(), &mut last_seq).await;

        match result {
            Ok(Some(ready)) => {
                session_id = Some(ready.session_id.clone());
                resume_url = ready.resume_gateway_url.clone();
                tracing::info!(
                    session_id = %ready.session_id,
                    "Discord gateway session established"
                );
            }
            Ok(None) => {
                // Clean shutdown — clear session for fresh identify
                session_id = None;
                resume_url = None;
                last_seq = None;
            }
            Err(e) => {
                tracing::error!(error = %e, "Discord gateway error, reconnecting in 5s");
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

/// Run a single gateway session. Returns ReadyData on successful identify/resume,
/// or None if the connection closed cleanly.
async fn run_gateway_session(
    state: &Arc<AppState>,
    token: &str,
    url: &str,
    session_id: Option<&str>,
    last_seq: &mut Option<u64>,
) -> anyhow::Result<Option<ReadyData>> {
    let (mut ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|e| anyhow::anyhow!("WebSocket connect failed: {}", e))?;

    tracing::info!("Discord gateway WebSocket connected");

    // Wait for Hello (op 10)
    let heartbeat_interval = {
        let msg = ws
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("Connection closed before Hello"))?
            .map_err(|e| anyhow::anyhow!("WebSocket error: {}", e))?;
        let event: GatewayEvent = parse_message(&msg)?;
        if event.op != 10 {
            anyhow::bail!("Expected Hello (op 10), got op {}", event.op);
        }
        let hello: HelloData = serde_json::from_value(event.d)
            .map_err(|e| anyhow::anyhow!("Invalid Hello data: {}", e))?;
        hello.heartbeat_interval
    };

    // Identify or Resume
    if let Some(sid) = session_id {
        let resume = GatewayPayload {
            op: 6,
            d: Some(serde_json::json!({
                "token": token,
                "session_id": sid,
                "seq": *last_seq,
            })),
            s: None,
            t: None,
        };
        send_payload(&mut ws, &resume).await?;
        tracing::info!(session_id = %sid, "Discord gateway resuming session");
    } else {
        let identify = GatewayPayload {
            op: 2,
            d: Some(serde_json::json!({
                "token": token,
                "intents": INTENTS,
                "properties": {
                    "os": "macos",
                    "browser": "claudy",
                    "device": "claudy",
                },
            })),
            s: None,
            t: None,
        };
        send_payload(&mut ws, &identify).await?;
        tracing::info!("Discord gateway identifying");
    }

    // Main event loop with heartbeat
    let mut heartbeat_ack = true;
    let mut ready_data: Option<ReadyData> = None;
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(heartbeat_interval));
    // First heartbeat after jitter (random 0-1 * interval)
    let jitter = (rand::random::<f64>() * heartbeat_interval as f64) as u64;
    tokio::time::sleep(std::time::Duration::from_millis(jitter)).await;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if !heartbeat_ack {
                    tracing::warn!("Discord gateway: no heartbeat ACK, reconnecting");
                    anyhow::bail!("Heartbeat timeout");
                }
                let hb = GatewayPayload {
                    op: 1,
                    d: Some(serde_json::json!(*last_seq)),
                    s: None,
                    t: None,
                };
                send_payload(&mut ws, &hb).await?;
                heartbeat_ack = false;
            }
            msg = ws.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        let event = match parse_message(&msg) {
                            Ok(e) => e,
                            Err(e) => {
                                tracing::warn!(error = %e, "Failed to parse gateway message");
                                continue;
                            }
                        };

                        if event.s.is_some() {
                            *last_seq = event.s;
                        }

                        match event.op {
                            0 => {
                                // Dispatch
                                let event_type = event.t.as_deref().unwrap_or("");
                                match event_type {
                                    "READY" => {
                                        if let Ok(ready) = serde_json::from_value::<ReadyData>(event.d) {
                                            ready_data = Some(ReadyData {
                                                session_id: ready.session_id,
                                                resume_gateway_url: ready.resume_gateway_url,
                                            });
                                        }
                                        tracing::info!("Discord gateway ready");
                                    }
                                    "RESUMED" => {
                                        tracing::info!("Discord gateway resumed");
                                    }
                                    "MESSAGE_CREATE" => {
                                        handle_message_create(state, &event.d).await;
                                    }
                                    "INTERACTION_CREATE" => {
                                        handle_interaction_create(state, &event.d).await;
                                    }
                                    _ => {}
                                }
                            }
                            1 => {
                                // Heartbeat request — respond immediately
                                let hb = GatewayPayload {
                                    op: 1,
                                    d: Some(serde_json::json!(*last_seq)),
                                    s: None,
                                    t: None,
                                };
                                send_payload(&mut ws, &hb).await?;
                            }
                            7 => {
                                // Reconnect
                                tracing::info!("Discord gateway reconnect requested");
                                return Ok(ready_data);
                            }
                            9 => {
                                // Invalid session
                                let can_resume = event.d.as_bool().unwrap_or(false);
                                if !can_resume {
                                    *last_seq = None;
                                    return Ok(None);
                                }
                                // Will retry with resume
                                return Ok(ready_data);
                            }
                            11 => {
                                heartbeat_ack = true;
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!(error = %e, "Discord gateway WebSocket error");
                        anyhow::bail!("WebSocket error: {}", e);
                    }
                    None => {
                        tracing::info!("Discord gateway WebSocket closed");
                        return Ok(ready_data);
                    }
                }
            }
        }
    }
}

async fn handle_message_create(state: &Arc<AppState>, data: &serde_json::Value) {
    let author = data.get("author");
    let is_bot = author
        .and_then(|a| a.get("bot"))
        .and_then(|b| b.as_bool())
        .unwrap_or(false);
    if is_bot {
        return;
    }

    if let Some(event) = super::normalize::normalize_gateway_message(data) {
        authorize_and_spawn(state, Platform::Discord, event);
    }
}

async fn handle_interaction_create(state: &Arc<AppState>, data: &serde_json::Value) {
    let interaction = match super::webhook::DiscordInteraction::from_gateway_event(data) {
        Some(i) => i,
        None => return,
    };

    // Ack the interaction via REST (type 5 = DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE)
    {
        let id = &interaction.id;
        let token = &interaction.token;
        let _ = super::api::DiscordApi::defer_interaction(&state.secrets, id, token).await;
    }

    if let Some(event) = super::normalize::normalize_interaction(&interaction) {
        authorize_and_spawn(state, Platform::Discord, event);
    }
}

fn parse_message(msg: &Message) -> anyhow::Result<GatewayEvent> {
    let text = match msg {
        Message::Text(t) => t.as_ref(),
        Message::Close(c) => {
            anyhow::bail!("WebSocket closed: {:?}", c);
        }
        _ => {
            return Ok(GatewayEvent {
                op: 255,
                d: serde_json::Value::Null,
                s: None,
                t: None,
            });
        }
    };
    serde_json::from_str(text).map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))
}

async fn send_payload(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    payload: &GatewayPayload,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(payload)?;
    ws.send(Message::Text(json.into()))
        .await
        .map_err(|e| anyhow::anyhow!("WebSocket send error: {}", e))
}
