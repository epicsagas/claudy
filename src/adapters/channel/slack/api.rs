use anyhow::{Context, Result};
use serde::Deserialize;

pub struct SlackApi {
    client: reqwest::Client,
    bot_token: String,
}

#[derive(Debug, Deserialize)]
struct SlackResponse {
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackPostResponse {
    pub ts: String,
}

impl SlackApi {
    const BASE_URL: &'static str = "https://slack.com/api";

    pub fn new(bot_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token,
        }
    }

    async fn post_json<T: serde::Serialize>(
        &self,
        method: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        let url = format!("{}/{}", Self::BASE_URL, method);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json; charset=utf-8")
            .json(body)
            .send()
            .await
            .with_context(|| format!("failed to send request to Slack {method}"))?;

        Ok(response)
    }

    fn check_response(resp: &SlackResponse, method: &str) -> Result<()> {
        if resp.ok {
            return Ok(());
        }
        let error = resp.error.as_deref().unwrap_or("unknown_error");
        anyhow::bail!("Slack {method} failed: {error}")
    }

    pub async fn post_message(
        &self,
        channel: &str,
        text: &str,
        blocks: Option<serde_json::Value>,
    ) -> Result<SlackPostResponse> {
        let mut body = serde_json::json!({
            "channel": channel,
            "text": text,
        });

        if let Some(blocks) = blocks {
            body["blocks"] = blocks;
        }

        let response = self.post_json("chat.postMessage", &body).await?;
        let parsed: SlackPostResponse = self.parse_response("chat.postMessage", response).await?;

        Ok(parsed)
    }

    pub async fn update_message(
        &self,
        channel: &str,
        ts: &str,
        text: &str,
        blocks: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut body = serde_json::json!({
            "channel": channel,
            "ts": ts,
            "text": text,
        });

        if let Some(blocks) = blocks {
            body["blocks"] = blocks;
        } else {
            body["blocks"] = serde_json::json!([]);
        }

        let response = self.post_json("chat.update", &body).await?;
        self.parse_unit_response("chat.update", response).await
    }

    pub async fn delete_message(&self, channel: &str, ts: &str) -> Result<()> {
        let body = serde_json::json!({
            "channel": channel,
            "ts": ts,
        });

        let response = self.post_json("chat.delete", &body).await?;
        self.parse_unit_response("chat.delete", response).await
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        response: reqwest::Response,
    ) -> Result<T> {
        let text = response
            .text()
            .await
            .context("failed to read Slack response body")?;

        // Check the envelope for errors first.
        let envelope: serde_json::Value =
            serde_json::from_str(&text).context("failed to parse Slack response as JSON")?;

        if envelope["ok"].as_bool() != Some(true) {
            let error = envelope["error"].as_str().unwrap_or("unknown_error");
            anyhow::bail!("Slack {method} failed: {error}");
        }

        serde_json::from_str::<T>(&text).context("failed to deserialize Slack response")
    }

    async fn parse_unit_response(&self, method: &str, response: reqwest::Response) -> Result<()> {
        let text = response
            .text()
            .await
            .context("failed to read Slack response body")?;

        let parsed: SlackResponse =
            serde_json::from_str(&text).context("failed to parse Slack response")?;

        Self::check_response(&parsed, method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_api_with_token() {
        let api = SlackApi::new("xoxb-test-token".to_string());
        assert_eq!(api.bot_token, "xoxb-test-token");
    }

    #[test]
    fn check_response_ok() {
        let resp = SlackResponse {
            ok: true,
            error: None,
        };
        assert!(SlackApi::check_response(&resp, "test").is_ok());
    }

    #[test]
    fn check_response_error() {
        let resp = SlackResponse {
            ok: false,
            error: Some("channel_not_found".to_string()),
        };
        let result = SlackApi::check_response(&resp, "chat.postMessage");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("channel_not_found"));
    }

    #[test]
    fn check_response_error_without_message() {
        let resp = SlackResponse {
            ok: false,
            error: None,
        };
        let result = SlackApi::check_response(&resp, "chat.postMessage");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown_error"));
    }
}
