use anyhow::Result;
use async_trait::async_trait;

use crate::domain::channel_events::{
    ChannelIdentity, ConversationId, MessageDelivery, OutboundMessage,
};
use crate::domain::channel_session::{ChannelSession, DeliveryAttempt, SessionStatus};

#[async_trait]
pub trait ChannelPort: Send + Sync {
    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageDelivery>;
    async fn edit_message(&self, msg: &OutboundMessage) -> Result<()>;
    async fn delete_message(&self, channel: &ChannelIdentity, message_ref: &str) -> Result<()>;
    async fn ack_interaction(&self, channel: &ChannelIdentity, interaction_id: &str) -> Result<()>;
    async fn send_typing(&self, channel: &ChannelIdentity) -> Result<()>;

    fn as_any(&self) -> &dyn std::any::Any;
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: ChannelSession) -> Result<()>;
    async fn get(&self, id: &ConversationId) -> Result<Option<ChannelSession>>;
    async fn update_status(&self, id: &ConversationId, status: SessionStatus) -> Result<()>;
    async fn remove(&self, id: &ConversationId) -> Result<()>;
    async fn list_active(&self) -> Result<Vec<ChannelSession>>;
}

#[async_trait]
pub trait DeliveryStore: Send + Sync {
    async fn record_attempt(&self, attempt: DeliveryAttempt) -> Result<()>;
    async fn is_duplicate(&self, idempotency_key: &str) -> Result<bool>;
}
