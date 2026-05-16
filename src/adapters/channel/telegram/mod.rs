pub mod api;
pub mod buttons;
pub mod normalize;

use anyhow::Result;
use async_trait::async_trait;

use self::api::TelegramApi;
use crate::domain::channel_events::{ChannelIdentity, MessageDelivery, OutboundMessage};
use crate::ports::channel_ports::ChannelPort;

pub struct TelegramAdapter {
    api: TelegramApi,
}

impl TelegramAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            api: TelegramApi::new(bot_token),
        }
    }

    pub async fn send_reply_keyboard(&self, chat_id: &str) -> anyhow::Result<()> {
        self.api.send_reply_keyboard(chat_id).await
    }
}

#[async_trait]
impl ChannelPort for TelegramAdapter {
    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageDelivery> {
        let chat_id = &msg.channel.channel_id;
        let tg_msg = self
            .api
            .send_message(chat_id, &msg.text, msg.interaction.as_ref())
            .await?;

        Ok(MessageDelivery {
            platform_message_id: tg_msg.message_id.to_string(),
        })
    }

    async fn edit_message(&self, msg: &OutboundMessage) -> Result<()> {
        let chat_id = &msg.channel.channel_id;
        let ref_str = msg
            .message_ref
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("message_ref required for telegram edit"))?;
        let message_id: i64 = ref_str
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid telegram message_ref for edit"))?;

        self.api
            .edit_message_text(chat_id, message_id, &msg.text, msg.interaction.as_ref())
            .await
    }

    async fn delete_message(&self, channel: &ChannelIdentity, message_ref: &str) -> Result<()> {
        let chat_id = &channel.channel_id;
        let message_id: i64 = message_ref
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid telegram message_ref for delete"))?;

        self.api.delete_message(chat_id, message_id).await
    }

    async fn ack_interaction(
        &self,
        _channel: &ChannelIdentity,
        interaction_id: &str,
    ) -> Result<()> {
        self.api.answer_callback_query(interaction_id).await
    }

    async fn send_typing(&self, channel: &ChannelIdentity) -> Result<()> {
        self.api
            .send_chat_action(&channel.channel_id, "typing")
            .await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_events::{Button, ConversationId, InteractionButtons, Platform};

    fn make_outbound(text: &str, message_ref: Option<&str>) -> OutboundMessage {
        OutboundMessage {
            conversation_id: ConversationId::from_platform(Platform::Telegram, "123"),
            channel: ChannelIdentity {
                platform: Platform::Telegram,
                channel_id: "123".to_string(),
                user_id: "456".to_string(),
                thread_id: None,
                guild_id: None,
            },
            text: text.to_string(),
            message_ref: message_ref.map(String::from),
            interaction: None,
        }
    }

    fn make_outbound_with_buttons() -> OutboundMessage {
        OutboundMessage {
            conversation_id: ConversationId::from_platform(Platform::Telegram, "123"),
            channel: ChannelIdentity {
                platform: Platform::Telegram,
                channel_id: "123".to_string(),
                user_id: "456".to_string(),
                thread_id: None,
                guild_id: None,
            },
            text: "Choose an option".to_string(),
            message_ref: None,
            interaction: Some(InteractionButtons {
                prompt_text: "Pick one".to_string(),
                buttons: vec![
                    Button {
                        id: "allow:1".to_string(),
                        label: "Allow".to_string(),
                    },
                    Button {
                        id: "deny:1".to_string(),
                        label: "Deny".to_string(),
                    },
                ],
            }),
        }
    }

    #[test]
    fn adapter_new_stores_token() {
        let adapter = TelegramAdapter::new("123456:TOKEN".to_string());
        assert_eq!(adapter.api.bot_token, "123456:TOKEN");
    }

    #[test]
    fn make_outbound_has_correct_platform() {
        let msg = make_outbound("test", None);
        assert_eq!(msg.channel.platform, Platform::Telegram);
        assert_eq!(msg.channel.channel_id, "123");
    }

    #[test]
    fn make_outbound_with_buttons_builds_correctly() {
        let msg = make_outbound_with_buttons();
        let interaction = msg.interaction.as_ref().unwrap();
        assert_eq!(interaction.buttons.len(), 2);
        assert_eq!(interaction.buttons[0].id, "allow:1");
        assert_eq!(interaction.buttons[1].label, "Deny");
    }
}
