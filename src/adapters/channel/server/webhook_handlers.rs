use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;

use crate::domain::channel_events::Platform;

use super::{AppState, authorize_and_spawn, constant_time_eq};

pub(super) async fn telegram_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> StatusCode {
    // Verify Telegram secret token header.
    // The secret token is configured via setWebhook's secret_token parameter
    // and stored as TELEGRAM_WEBHOOK_SECRET in the secrets vault.
    let expected_secret = state.secrets.get("TELEGRAM_WEBHOOK_SECRET");
    if let Some(secret) = expected_secret
        && !secret.is_empty()
    {
        let provided = headers
            .get("X-Telegram-Bot-Api-Secret-Token")
            .and_then(|v| v.to_str().ok());
        match provided {
            Some(token) if constant_time_eq(token.as_bytes(), secret.as_bytes()) => {}
            _ => {
                tracing::warn!("Telegram webhook rejected: invalid or missing secret token");
                return StatusCode::UNAUTHORIZED;
            }
        }
    }

    use crate::adapters::channel::telegram::normalize::{TelegramUpdate, normalize_update};

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse Telegram update");
            return StatusCode::BAD_REQUEST;
        }
    };

    let update: TelegramUpdate = match serde_json::from_value(payload) {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse Telegram update");
            return StatusCode::BAD_REQUEST;
        }
    };

    let event = match normalize_update(update) {
        Some(e) => e,
        None => return StatusCode::OK,
    };

    authorize_and_spawn(&state, Platform::Telegram, event);

    StatusCode::OK
}

pub(super) async fn slack_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> impl IntoResponse {
    // Verify Slack request signature using HMAC-SHA256.
    let signing_secret = state.secrets.get("SLACK_SIGNING_SECRET");
    match signing_secret {
        Some(secret) if !secret.is_empty() => {
            let timestamp = headers
                .get("X-Slack-Request-Timestamp")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let signature = headers
                .get("X-Slack-Signature")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            // Validate timestamp freshness to prevent replay attacks
            let timestamp_secs: i64 = match timestamp.parse() {
                Ok(t) => t,
                Err(_) => {
                    tracing::warn!("Slack webhook rejected: invalid timestamp");
                    return StatusCode::UNAUTHORIZED.into_response();
                }
            };
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            if (now_secs - timestamp_secs).unsigned_abs() > 300 {
                tracing::warn!("Slack webhook rejected: timestamp too old or too far in future");
                return StatusCode::UNAUTHORIZED.into_response();
            }

            if !crate::adapters::channel::slack::verify_signature(
                secret, &body, timestamp, signature,
            ) {
                tracing::warn!("Slack webhook rejected: invalid signature");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        }
        _ => {
            tracing::warn!("Slack webhook rejected: SLACK_SIGNING_SECRET not configured");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Slack interactive payloads are sent as application/x-www-form-urlencoded
    // with a `payload` field containing JSON.
    if content_type.contains("application/x-www-form-urlencoded") {
        let body_str = String::from_utf8_lossy(&body);
        let parsed = serde_urlencoded::from_str::<Vec<(String, String)>>(&body_str);
        if let Ok(params) = parsed
            && let Some(payload_str) = params
                .iter()
                .find(|(k, _)| k == "payload")
                .map(|(_, v)| v.clone())
        {
            match serde_json::from_str::<crate::adapters::channel::slack::SlackInteractionPayload>(
                &payload_str,
            ) {
                Ok(interaction_payload) => {
                    if let Some(event) =
                        crate::adapters::channel::slack::normalize_interaction(&interaction_payload)
                    {
                        authorize_and_spawn(&state, Platform::Slack, event);
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to parse Slack interaction payload");
                }
            }
        }
        return StatusCode::OK.into_response();
    }

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse Slack payload");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // Handle Slack URL verification challenge.
    if payload.get("type").and_then(|v| v.as_str()) == Some("url_verification") {
        if let Some(challenge) = payload.get("challenge").and_then(|v| v.as_str()) {
            return Json(serde_json::json!({ "challenge": challenge })).into_response();
        }
        return StatusCode::BAD_REQUEST.into_response();
    }

    // Handle Slack event callbacks.
    if payload.get("type").and_then(|v| v.as_str()) == Some("event_callback") {
        match serde_json::from_value::<crate::adapters::channel::slack::SlackEventCallback>(payload)
        {
            Ok(callback) => {
                if let Some(event) = crate::adapters::channel::slack::normalize_event(&callback) {
                    authorize_and_spawn(&state, Platform::Slack, event);
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to parse Slack event callback");
            }
        }
    }

    StatusCode::OK.into_response()
}

pub(super) async fn discord_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> impl IntoResponse {
    // Verify Discord Ed25519 signature.
    let public_key = state.secrets.get("DISCORD_PUBLIC_KEY");
    match public_key {
        Some(key) if !key.is_empty() => {
            let signature = headers
                .get("X-Signature-Ed25519")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let timestamp = headers
                .get("X-Signature-Timestamp")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            let sig_bytes = match hex::decode(signature) {
                Ok(b) => b,
                Err(_) => {
                    tracing::warn!("Discord webhook rejected: malformed signature hex");
                    return StatusCode::UNAUTHORIZED.into_response();
                }
            };

            if !crate::adapters::channel::discord::webhook::verify_discord_signature(
                key.as_bytes(),
                &body,
                &sig_bytes,
                timestamp.as_bytes(),
            ) {
                tracing::warn!("Discord webhook rejected: invalid signature");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        }
        _ => {
            tracing::warn!("Discord webhook rejected: DISCORD_PUBLIC_KEY not configured");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }

    let interaction: crate::adapters::channel::discord::webhook::DiscordInteraction =
        match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to parse Discord interaction");
                return StatusCode::BAD_REQUEST.into_response();
            }
        };

    // Handle Discord Ping interaction (type=1) by responding with Pong.
    if interaction.interaction_type
        == crate::adapters::channel::discord::webhook::DiscordInteractionType::Ping
    {
        return Json(serde_json::json!({ "type": 1 })).into_response();
    }

    // Normalize and dispatch Discord interactions.
    if let Some(event) =
        crate::adapters::channel::discord::normalize::normalize_interaction(&interaction)
    {
        authorize_and_spawn(&state, Platform::Discord, event);
    }

    StatusCode::OK.into_response()
}
