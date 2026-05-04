use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::domain::channel_events::ConversationId;
use crate::domain::channel_session::{ChannelSession, SessionStatus};

pub struct InMemorySessionStore {
    sessions: RwLock<HashMap<String, ChannelSession>>,
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create(&self, session: ChannelSession) -> anyhow::Result<()> {
        self.sessions
            .write()
            .await
            .insert(session.conversation_id.0.clone(), session);
        Ok(())
    }

    pub async fn get(&self, id: &ConversationId) -> anyhow::Result<Option<ChannelSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(&id.0).cloned())
    }

    pub async fn update_status(
        &self,
        id: &ConversationId,
        status: SessionStatus,
    ) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&id.0) {
            session.status = status;
        }
        Ok(())
    }

    pub async fn remove(&self, id: &ConversationId) -> anyhow::Result<()> {
        self.sessions.write().await.remove(&id.0);
        Ok(())
    }

    pub async fn list_active(&self) -> anyhow::Result<Vec<ChannelSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .filter(|s| s.status != SessionStatus::Stopped)
            .cloned()
            .collect())
    }
}
