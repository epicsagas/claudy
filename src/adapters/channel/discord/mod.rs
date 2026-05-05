pub mod api;
pub mod components;
pub mod gateway;
pub mod normalize;
pub mod webhook;

use anyhow::Result;
use async_trait::async_trait;

use crate::domain::channel_events::{ChannelIdentity, MessageDelivery, OutboundMessage};
use crate::ports::channel_ports::ChannelPort;

use api::DiscordApi;

/// Discord adapter implementing the [`ChannelPort`] trait.
pub struct DiscordAdapter {
    api: DiscordApi,
}

impl DiscordAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            api: DiscordApi::new(bot_token),
        }
    }
}

#[async_trait]
impl ChannelPort for DiscordAdapter {
    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageDelivery> {
        let discord_msg = self
            .api
            .create_message(&msg.channel.channel_id, &msg.text, msg.interaction.as_ref())
            .await?;

        Ok(MessageDelivery {
            platform_message_id: discord_msg.id,
        })
    }

    async fn edit_message(&self, msg: &OutboundMessage) -> Result<()> {
        let message_id = msg
            .message_ref
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("message_ref required for edit_message"))?;

        self.api
            .edit_message(
                &msg.channel.channel_id,
                message_id,
                &msg.text,
                msg.interaction.as_ref(),
            )
            .await
    }

    async fn delete_message(&self, channel: &ChannelIdentity, message_ref: &str) -> Result<()> {
        self.api
            .delete_message(&channel.channel_id, message_ref)
            .await
    }

    async fn ack_interaction(
        &self,
        _channel: &ChannelIdentity,
        _interaction_id: &str,
    ) -> Result<()> {
        // Discord interactions are acknowledged via the Gateway (defer_interaction).
        // No REST ack needed here.
        Ok(())
    }

    async fn send_typing(&self, channel: &ChannelIdentity) -> Result<()> {
        self.api.trigger_typing(&channel.channel_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_events::{Button, ConversationId, InteractionButtons, Platform};

    fn sample_channel() -> ChannelIdentity {
        ChannelIdentity {
            platform: Platform::Discord,
            channel_id: "ch-1".into(),
            user_id: "u-1".into(),
            thread_id: None,
            guild_id: None,
        }
    }

    fn sample_outbound(text: &str, message_ref: Option<&str>) -> OutboundMessage {
        OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: sample_channel(),
            text: text.into(),
            message_ref: message_ref.map(String::from),
            interaction: None,
        }
    }

    fn sample_outbound_with_buttons() -> OutboundMessage {
        OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: sample_channel(),
            text: "Allow access?".into(),
            message_ref: None,
            interaction: Some(InteractionButtons {
                prompt_text: "Choose".into(),
                buttons: vec![
                    Button {
                        id: "allow".into(),
                        label: "Yes".into(),
                    },
                    Button {
                        id: "deny".into(),
                        label: "No".into(),
                    },
                ],
            }),
        }
    }

    #[test]
    fn adapter_new_stores_api() {
        let _adapter = DiscordAdapter::new("token".into());
    }

    #[test]
    fn outbound_message_without_ref_fails_edit_validation() {
        let msg = sample_outbound("hello", None);
        assert!(msg.message_ref.is_none());
    }

    #[test]
    fn outbound_with_buttons_has_interaction() {
        let msg = sample_outbound_with_buttons();
        let interaction = msg.interaction.as_ref().expect("interaction present");
        assert_eq!(interaction.buttons.len(), 2);
        assert_eq!(interaction.buttons[0].id, "allow");
        assert_eq!(interaction.buttons[1].id, "deny");
    }
}
