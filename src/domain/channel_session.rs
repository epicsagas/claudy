use serde::{Deserialize, Serialize};

use super::channel_events::{ChannelIdentity, ConversationId, Platform};

#[derive(Debug, Clone)]
pub struct ChannelSession {
    pub conversation_id: ConversationId,
    pub channel: ChannelIdentity,
    pub profile: String,
    pub status: SessionStatus,
    pub claude_process_id: Option<u32>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Starting,
    Running,
    WaitingPermission,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSession {
    pub conversation_id: String,
    pub platform: String,
    pub channel_id: String,
    pub user_id: String,
    pub thread_id: Option<String>,
    pub profile: String,
    pub status: SessionStatus,
    pub created_at: u64,
}

impl From<&ChannelSession> for PersistedSession {
    fn from(s: &ChannelSession) -> Self {
        Self {
            conversation_id: s.conversation_id.0.clone(),
            platform: s.channel.platform.as_str().to_string(),
            channel_id: s.channel.channel_id.clone(),
            user_id: s.channel.user_id.clone(),
            thread_id: s.channel.thread_id.clone(),
            profile: s.profile.clone(),
            status: s.status,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeliveryAttempt {
    pub idempotency_key: String,
    pub conversation_id: ConversationId,
    pub platform: Platform,
    pub attempt_no: u32,
    pub status_code: u16,
    pub error_message: Option<String>,
    pub timestamp: u64,
}
