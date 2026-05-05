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
}

impl DiscordApi {
    pub fn new(bot_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token,
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

    /// Acknowledge an interaction via REST (type 5 = DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE).
    /// Used by Gateway to respond to INTERACTION_CREATE events.
    pub async fn defer_interaction(
        secrets: &crate::config::vault::SecretVault,
        interaction_id: &str,
        token: &str,
    ) -> Result<()> {
        let bot_token = secrets
            .get("DISCORD_BOT_TOKEN")
            .cloned()
            .unwrap_or_default();
        let client = reqwest::Client::new();

        let url = format!(
            "{}/interactions/{}/{}/callback",
            API_BASE, interaction_id, token
        );
        let body = serde_json::json!({
            "type": 5,
            "data": { "flags": 64 }
        });

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bot {bot_token}"))
            .json(&body)
            .send()
            .await
            .context("defer_interaction request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(
                status = %status,
                "defer_interaction failed: {text}"
            );
        }

        Ok(())
    }

    /// Fetch the bot's application ID via `GET /oauth2/applications/@me`.
    pub async fn get_application_id(&self) -> Result<String> {
        let url = format!("{API_BASE}/oauth2/applications/@me");
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .context("discord get_application_id request failed")?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .context("discord get_application_id read body")?;

        if !status.is_success() {
            bail!("get_application_id failed: status={status} body={text}");
        }

        let app: serde_json::Value =
            serde_json::from_str(&text).context("discord get_application_id parse response")?;
        app.get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .context("discord get_application_id: missing 'id' field")
    }

    /// Register global Application Commands via `PUT /applications/{app_id}/commands`.
    pub async fn register_application_commands(
        &self,
        application_id: &str,
        commands: &[CommandDefinition],
    ) -> Result<()> {
        let url = format!("{API_BASE}/applications/{application_id}/commands");
        let body: Vec<serde_json::Value> = commands
            .iter()
            .map(|cmd| {
                let mut obj = serde_json::json!({
                    "name": cmd.name,
                    "description": cmd.description,
                });
                if !cmd.options.is_empty() {
                    obj.as_object_mut()
                        .expect("root is object")
                        .insert("options".into(), serde_json::json!(cmd.options));
                }
                obj
            })
            .collect();

        let resp = self
            .client
            .put(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&body)
            .send()
            .await
            .context("discord register_application_commands request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("register_application_commands failed: status={status} body={text}");
        }

        Ok(())
    }
}

/// A slash command definition for Discord Application Commands.
pub struct CommandDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub options: Vec<CommandOption>,
}

/// Discord Application Command option type constants.
pub mod option_kind {
    /// STRING option type (value 3).
    pub const STRING: u8 = 3;
}

/// A single option for a slash command.
pub struct CommandOption {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: u8,
    pub required: bool,
}

impl serde::Serialize for CommandOption {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("CommandOption", 4)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("description", &self.description)?;
        s.serialize_field("type", &self.kind)?;
        s.serialize_field("required", &self.required)?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constructs_api() {
        let api = DiscordApi::new("tok".into());
        assert_eq!(api.bot_token, "tok");
    }
}
