use anyhow::{Context, Result, bail};
use serde::Deserialize;

use super::components::to_action_row;
use crate::domain::channel_events::InteractionButtons;

const API_BASE: &str = "https://discord.com/api/v10";

/// Minimal response shape we need from the Discord messages API.
#[derive(Debug, Deserialize)]
pub struct DiscordMessage {
    pub id: String,
}

/// Structured Discord API error (best-effort extraction).
#[derive(Debug, Deserialize)]
struct DiscordApiError {
    message: Option<String>,
    code: Option<i64>,
}

/// Low-level Discord REST client for the messages resource.
pub struct DiscordApi {
    client: reqwest::Client,
    bot_token: String,
    application_id: String,
}

impl DiscordApi {
    pub fn new(bot_token: String, application_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token,
            application_id,
        }
    }

    /// POST `/channels/{channel_id}/messages`
    pub async fn create_message(
        &self,
        channel_id: &str,
        content: &str,
        interaction: Option<&InteractionButtons>,
    ) -> Result<DiscordMessage> {
        let mut body = serde_json::json!({ "content": content });

        if let Some(buttons) = interaction {
            body.as_object_mut()
                .expect("body is always an object")
                .insert("components".into(), to_action_row(buttons));
        }

        let url = format!("{API_BASE}/channels/{channel_id}/messages");
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&body)
            .send()
            .await
            .context("discord create_message request failed")?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .context("discord create_message read body")?;

        if !status.is_success() {
            let msg = serde_json::from_str::<DiscordApiError>(&text)
                .map(|e| format!("discord error code={:?} message={:?}", e.code, e.message))
                .unwrap_or_else(|_| format!("discord error: status={status} body={text}"));
            bail!("create_message failed: {msg}");
        }

        serde_json::from_str::<DiscordMessage>(&text)
            .context("discord create_message parse response")
    }

    /// PATCH `/channels/{channel_id}/messages/{message_id}`
    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        content: &str,
        interaction: Option<&InteractionButtons>,
    ) -> Result<()> {
        let mut body = serde_json::json!({ "content": content });

        match interaction {
            Some(buttons) => {
                body.as_object_mut()
                    .expect("body is always an object")
                    .insert("components".into(), to_action_row(buttons));
            }
            None => {
                body.as_object_mut()
                    .expect("body is always an object")
                    .insert("components".into(), serde_json::json!([]));
            }
        }

        let url = format!("{API_BASE}/channels/{channel_id}/messages/{message_id}");
        let resp = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&body)
            .send()
            .await
            .context("discord edit_message request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("edit_message failed: status={status} body={text}");
        }

        Ok(())
    }

    /// DELETE `/channels/{channel_id}/messages/{message_id}`
    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let url = format!("{API_BASE}/channels/{channel_id}/messages/{message_id}");
        let resp = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .context("discord delete_message request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("delete_message failed: status={status} body={text}");
        }

        Ok(())
    }

    /// POST `/channels/{channel_id}/typing`
    pub async fn trigger_typing(&self, channel_id: &str) -> Result<()> {
        let url = format!("{API_BASE}/channels/{channel_id}/typing");
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .context("discord trigger_typing request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("trigger_typing failed: status={status} body={text}");
        }

        Ok(())
    }

    /// Edit the original interaction response via the webhooks endpoint.
    ///
    /// Discord requires acknowledging interactions through its interaction
    /// response API. We use the "edit original response" route for this,
    /// which works after the initial `DEFERRED_UPDATE_MESSAGE` or similar
    /// acknowledgement has been sent by the gateway/webhook handler.
    pub async fn edit_original_interaction_response(
        &self,
        interaction_token: &str,
        content: &str,
    ) -> Result<()> {
        let url = format!(
            "{API_BASE}/webhooks/{app_id}/{token}/messages/@original",
            app_id = self.application_id,
            token = interaction_token,
        );
        let resp = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&serde_json::json!({ "content": content }))
            .send()
            .await
            .context("discord edit_original_interaction_response request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("edit_original_interaction_response failed: status={status} body={text}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constructs_api() {
        let api = DiscordApi::new("tok".into(), "app".into());
        assert_eq!(api.bot_token, "tok");
        assert_eq!(api.application_id, "app");
    }
}
