use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Telegram,
    Slack,
    Discord,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::Slack => "slack",
            Self::Discord => "discord",
        }
    }

    pub fn max_message_length(&self) -> usize {
        match self {
            Self::Telegram => 4096,
            Self::Discord => 2000,
            Self::Slack => 40000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelIdentity {
    pub platform: Platform,
    pub channel_id: String,
    pub user_id: String,
    pub thread_id: Option<String>,
    /// Discord guild (server) ID, or Slack team/workspace ID.
    pub guild_id: Option<String>,
}

impl ChannelIdentity {
    pub fn new(
        platform: Platform,
        channel_id: impl Into<String>,
        user_id: impl Into<String>,
        thread_id: Option<String>,
        guild_id: Option<String>,
    ) -> Self {
        Self {
            platform,
            channel_id: channel_id.into(),
            user_id: user_id.into(),
            thread_id,
            guild_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationId(pub String);

impl Default for ConversationId {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_platform(platform: Platform, id: &str) -> Self {
        Self(format!("{}-{}", platform.as_str(), id))
    }
}

#[derive(Debug, Clone)]
pub enum IncomingEvent {
    TextMessage(TextMessage),
    BotCommand {
        command: String,
        args: String,
        channel: ChannelIdentity,
        conversation_id: ConversationId,
    },
    Interaction(InteractionEvent),
    Attachment(AttachmentEvent),
}

#[derive(Debug, Clone)]
pub struct TextMessage {
    pub conversation_id: ConversationId,
    pub channel: ChannelIdentity,
    pub text: String,
    pub reply_to_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InteractionEvent {
    pub conversation_id: ConversationId,
    pub channel: ChannelIdentity,
    /// The callback action parsed from data (e.g. "sess", "proj", "model").
    pub action_id: String,
    /// The payload after the action prefix (e.g. session ID prefix, encoded dir, model name).
    pub message_ref: String,
    /// Platform-specific message reference for the message containing the button.
    /// Used to dismiss the keyboard by editing the original message.
    /// - Telegram: message_id as string (e.g. "200")
    /// - Discord: message snowflake (e.g. "123456789012345678")
    /// - Slack: composite "channel:ts" (e.g. "C12345:1234567890.654321")
    pub callback_message_id: Option<String>,
    /// The platform's callback query ID (e.g. Telegram callback_query ID).
    /// Used for answerCallbackQuery to dismiss the loading spinner.
    pub callback_query_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttachmentEvent {
    pub conversation_id: ConversationId,
    pub channel: ChannelIdentity,
    pub file_name: String,
    pub mime_type: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub conversation_id: ConversationId,
    pub channel: ChannelIdentity,
    pub text: String,
    pub message_ref: Option<String>,
    pub interaction: Option<InteractionButtons>,
}

#[derive(Debug, Clone)]
pub struct InteractionButtons {
    pub prompt_text: String,
    pub buttons: Vec<Button>,
}

#[derive(Debug, Clone)]
pub struct Button {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny,
}

#[derive(Debug, Clone)]
pub struct MessageDelivery {
    pub platform_message_id: String,
}
