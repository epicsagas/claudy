use anyhow::{Context, Result};
use serde::Deserialize;

use super::buttons::to_inline_keyboard;
use super::normalize::TelegramUpdate;
use crate::domain::channel_events::InteractionButtons;

#[derive(Debug, Deserialize)]
struct TelegramResponse {
    ok: bool,
    result: Option<serde_json::Value>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
}

pub struct TelegramApi {
    client: reqwest::Client,
    pub(crate) bot_token: String,
}

impl TelegramApi {
    pub fn new(bot_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token,
        }
    }

    fn url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{method}", self.bot_token)
    }

    pub async fn send_message(
        &self,
        chat_id: &str,
        text: &str,
        interaction: Option<&InteractionButtons>,
    ) -> Result<TelegramMessage> {
        let mut body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });

        if let Some(buttons) = interaction {
            body["reply_markup"] = to_inline_keyboard(buttons);
        }

        let resp: TelegramResponse = self
            .client
            .post(self.url("sendMessage"))
            .json(&body)
            .send()
            .await
            .context("telegram sendMessage request failed")?
            .json()
            .await
            .context("telegram sendMessage response parse failed")?;

        if !resp.ok {
            let desc = resp
                .description
                .unwrap_or_else(|| "unknown error".to_string());
            anyhow::bail!("telegram sendMessage failed: {desc}");
        }

        let result = resp.result.context("telegram sendMessage missing result")?;
        let message: TelegramMessage =
            serde_json::from_value(result).context("telegram sendMessage result parse failed")?;

        Ok(message)
    }

    pub async fn edit_message_text(
        &self,
        chat_id: &str,
        message_id: i64,
        text: &str,
        interaction: Option<&InteractionButtons>,
    ) -> Result<()> {
        let mut body = serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "text": text,
        });

        match interaction {
            Some(buttons) => {
                body["reply_markup"] = to_inline_keyboard(buttons);
            }
            None => {
                body["reply_markup"] = serde_json::json!({"inline_keyboard": []});
            }
        }

        let resp: TelegramResponse = self
            .client
            .post(self.url("editMessageText"))
            .json(&body)
            .send()
            .await
            .context("telegram editMessageText request failed")?
            .json()
            .await
            .context("telegram editMessageText response parse failed")?;

        if !resp.ok {
            let desc = resp
                .description
                .unwrap_or_else(|| "unknown error".to_string());
            anyhow::bail!("telegram editMessageText failed: {desc}");
        }

        Ok(())
    }

    pub async fn delete_message(&self, chat_id: &str, message_id: i64) -> Result<()> {
        let body = serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
        });

        let resp: TelegramResponse = self
            .client
            .post(self.url("deleteMessage"))
            .json(&body)
            .send()
            .await
            .context("telegram deleteMessage request failed")?
            .json()
            .await
            .context("telegram deleteMessage response parse failed")?;

        if !resp.ok {
            let desc = resp
                .description
                .unwrap_or_else(|| "unknown error".to_string());
            anyhow::bail!("telegram deleteMessage failed: {desc}");
        }

        Ok(())
    }

    pub async fn answer_callback_query(&self, callback_query_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "callback_query_id": callback_query_id,
        });

        let resp: TelegramResponse = self
            .client
            .post(self.url("answerCallbackQuery"))
            .json(&body)
            .send()
            .await
            .context("telegram answerCallbackQuery request failed")?
            .json()
            .await
            .context("telegram answerCallbackQuery response parse failed")?;

        if !resp.ok {
            let desc = resp
                .description
                .unwrap_or_else(|| "unknown error".to_string());
            anyhow::bail!("telegram answerCallbackQuery failed: {desc}");
        }

        Ok(())
    }

    pub async fn send_chat_action(&self, chat_id: &str, action: &str) -> Result<()> {
        let body = serde_json::json!({
            "chat_id": chat_id,
            "action": action,
        });

        let resp: TelegramResponse = self
            .client
            .post(self.url("sendChatAction"))
            .json(&body)
            .send()
            .await
            .context("telegram sendChatAction request failed")?
            .json()
            .await
            .context("telegram sendChatAction response parse failed")?;

        if !resp.ok {
            let desc = resp
                .description
                .unwrap_or_else(|| "unknown error".to_string());
            anyhow::bail!("telegram sendChatAction failed: {desc}");
        }

        Ok(())
    }

    pub async fn get_updates(
        &self,
        offset: Option<i64>,
        timeout: u32,
    ) -> Result<Vec<TelegramUpdate>> {
        let mut body = serde_json::json!({
            "timeout": timeout,
            "allowed_updates": ["message", "callback_query"],
        });
        if let Some(o) = offset {
            body["offset"] = serde_json::json!(o);
        }

        let resp: TelegramResponse = self
            .client
            .post(self.url("getUpdates"))
            .json(&body)
            .send()
            .await
            .context("telegram getUpdates request failed")?
            .json()
            .await
            .context("telegram getUpdates response parse failed")?;

        if !resp.ok {
            let desc = resp
                .description
                .unwrap_or_else(|| "unknown error".to_string());
            anyhow::bail!("telegram getUpdates failed: {desc}");
        }

        let result = resp.result.unwrap_or(serde_json::Value::Null);
        let updates: Vec<TelegramUpdate> =
            serde_json::from_value(result).context("telegram getUpdates result parse failed")?;

        Ok(updates)
    }

    pub async fn set_my_commands(&self, commands: &[(&str, &str)]) -> Result<()> {
        let cmds: Vec<serde_json::Value> = commands
            .iter()
            .map(|(cmd, desc)| serde_json::json!({"command": cmd, "description": desc}))
            .collect();
        let body = serde_json::json!({"commands": cmds});

        let resp: TelegramResponse = self
            .client
            .post(self.url("setMyCommands"))
            .json(&body)
            .send()
            .await
            .context("telegram setMyCommands request failed")?
            .json()
            .await
            .context("telegram setMyCommands response parse failed")?;

        if !resp.ok {
            let desc = resp
                .description
                .unwrap_or_else(|| "unknown error".to_string());
            anyhow::bail!("telegram setMyCommands failed: {desc}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_api_with_token() {
        let api = TelegramApi::new("123456:ABC-DEF".to_string());
        assert_eq!(api.bot_token, "123456:ABC-DEF");
    }

    #[test]
    fn url_formats_correctly() {
        let api = TelegramApi::new("123456:ABC-DEF".to_string());
        assert_eq!(
            api.url("sendMessage"),
            "https://api.telegram.org/bot123456:ABC-DEF/sendMessage"
        );
        assert_eq!(
            api.url("getMe"),
            "https://api.telegram.org/bot123456:ABC-DEF/getMe"
        );
    }
}
