mod api;
mod blocks;
mod normalize;
pub mod socket_mode;
mod webhook;

use anyhow::Result;
use async_trait::async_trait;

use crate::domain::channel_events::{ChannelIdentity, MessageDelivery, OutboundMessage};
use crate::ports::channel_ports::ChannelPort;

pub use blocks::to_block_kit;
pub use normalize::{
    SlackEvent, SlackEventCallback, SlackInteractionPayload, SlackMessageEvent, normalize_event,
    normalize_interaction,
};
pub use webhook::{SlackChallenge, SlackEventPayload, verify_signature};

use self::api::SlackApi;

pub struct SlackAdapter {
    api: SlackApi,
}

impl SlackAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            api: SlackApi::new(bot_token),
        }
    }
}

#[async_trait]
impl ChannelPort for SlackAdapter {
    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageDelivery> {
        let blocks = msg.interaction.as_ref().map(blocks::to_block_kit);

        let response = self
            .api
            .post_message(&msg.channel.channel_id, &msg.text, blocks)
            .await?;

        let platform_message_id = format!("{}:{}", msg.channel.channel_id, response.ts);

        Ok(MessageDelivery {
            platform_message_id,
        })
    }

    async fn edit_message(&self, msg: &OutboundMessage) -> Result<()> {
        let message_ref = msg
            .message_ref
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("message_ref required for edit_message"))?;

        let (channel, ts) = parse_message_ref(message_ref)?;

        let blocks = msg.interaction.as_ref().map(blocks::to_block_kit);
        self.api
            .update_message(&channel, &ts, &msg.text, blocks)
            .await
    }

    async fn delete_message(&self, channel: &ChannelIdentity, message_ref: &str) -> Result<()> {
        let (ch, ts) = parse_message_ref(message_ref)?;
        let _ = channel; // used only for trait signature consistency
        self.api.delete_message(&ch, &ts).await
    }

    async fn ack_interaction(
        &self,
        _channel: &ChannelIdentity,
        _interaction_id: &str,
    ) -> Result<()> {
        // Slack interactive components do not require explicit REST ACK;
        // responses are sent via the interaction payload response_url.
        Ok(())
    }

    async fn send_typing(&self, _channel: &ChannelIdentity) -> Result<()> {
        // Slack typing indicators are sent via the WebSocket (Socket Mode),
        // not through the REST API. No-op here.
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Parse a message reference in the format "{channel_id}:{ts}".
fn parse_message_ref(message_ref: &str) -> Result<(String, String)> {
    let (channel, ts) = message_ref
        .rsplit_once(':')
        .ok_or_else(|| anyhow::anyhow!("invalid Slack message_ref format: expected '{{channel_id}}:{{ts}}', got '{message_ref}'"))?;

    Ok((channel.to_string(), ts.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_events::Platform;

    #[test]
    fn parse_message_ref_valid() {
        let (channel, ts) = parse_message_ref("C12345:1234567890.123456").unwrap();
        assert_eq!(channel, "C12345");
        assert_eq!(ts, "1234567890.123456");
    }

    #[test]
    fn parse_message_ref_no_colon() {
        let result = parse_message_ref("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn parse_message_ref_multiple_colons() {
        // rsplit_once splits on the last colon, so ts part must not contain colons
        let (channel, ts) = parse_message_ref("C12345:extra:1234567890.123456").unwrap();
        assert_eq!(channel, "C12345:extra");
        assert_eq!(ts, "1234567890.123456");
    }

    #[tokio::test]
    async fn ack_interaction_is_noop() {
        let adapter = SlackAdapter::new("xoxb-test-token".to_string());
        let channel = ChannelIdentity {
            platform: Platform::Slack,
            channel_id: "C123".to_string(),
            user_id: "U456".to_string(),
            thread_id: None,
            guild_id: None,
        };
        let result = adapter.ack_interaction(&channel, "interaction_123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_typing_is_noop() {
        let adapter = SlackAdapter::new("xoxb-test-token".to_string());
        let channel = ChannelIdentity {
            platform: Platform::Slack,
            channel_id: "C123".to_string(),
            user_id: "U456".to_string(),
            thread_id: None,
            guild_id: None,
        };
        let result = adapter.send_typing(&channel).await;
        assert!(result.is_ok());
    }
}
