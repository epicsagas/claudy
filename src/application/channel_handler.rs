use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::channel_events::{
    IncomingEvent, InteractionEvent, OutboundMessage, PermissionDecision, Platform, TextMessage,
};
use crate::domain::channel_session::{ChannelSession, SessionStatus};
use crate::ports::channel_ports::ChannelPort;

use crate::adapters::channel::session::InMemorySessionStore;

pub struct ChannelHandler {
    channels: HashMap<Platform, Arc<dyn ChannelPort>>,
    sessions: Arc<InMemorySessionStore>,
}

impl ChannelHandler {
    pub fn new(
        channels: HashMap<Platform, Arc<dyn ChannelPort>>,
        sessions: Arc<InMemorySessionStore>,
    ) -> Self {
        Self { channels, sessions }
    }

    pub async fn handle_event(&self, event: IncomingEvent) -> anyhow::Result<()> {
        match event {
            IncomingEvent::TextMessage(msg) => self.handle_text(msg).await,
            IncomingEvent::Interaction(interaction) => self.handle_interaction(interaction).await,
            IncomingEvent::Attachment(_attachment) => {
                // Attachment support is deferred to a future phase
                Ok(())
            }
            IncomingEvent::BotCommand { .. } => {
                // Commands are handled directly in server.rs
                Ok(())
            }
        }
    }

    async fn handle_text(&self, msg: TextMessage) -> anyhow::Result<()> {
        let channel = self
            .channels
            .get(&msg.channel.platform)
            .ok_or_else(|| anyhow::anyhow!("No adapter for platform {:?}", msg.channel.platform))?;

        let _ = channel.send_typing(&msg.channel).await;

        let session = self.sessions.get(&msg.conversation_id).await?;
        if session.is_none() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let session = ChannelSession {
                conversation_id: msg.conversation_id.clone(),
                channel: msg.channel.clone(),
                profile: String::new(),
                status: SessionStatus::Starting,
                claude_process_id: None,
                created_at: now,
            };
            self.sessions.create(session).await?;
        }

        self.sessions
            .update_status(&msg.conversation_id, SessionStatus::Running)
            .await?;

        let outbound = OutboundMessage {
            conversation_id: msg.conversation_id.clone(),
            channel: msg.channel.clone(),
            text: format!("Processing: {}", &msg.text[..msg.text.len().min(100)]),
            message_ref: None,
            interaction: None,
        };

        let delivery = channel.send_message(&outbound).await?;

        tracing::info!(
            conversation = %msg.conversation_id.0,
            message_id = %delivery.platform_message_id,
            "Message sent"
        );

        Ok(())
    }

    async fn handle_interaction(&self, interaction: InteractionEvent) -> anyhow::Result<()> {
        let channel = self
            .channels
            .get(&interaction.channel.platform)
            .ok_or_else(|| {
                anyhow::anyhow!("No adapter for platform {:?}", interaction.channel.platform)
            })?;

        channel
            .ack_interaction(
                &interaction.channel,
                interaction
                    .callback_query_id
                    .as_deref()
                    .unwrap_or(&interaction.action_id),
            )
            .await?;

        let decision = match interaction.action_id.as_str() {
            "allow" => PermissionDecision::Allow,
            "deny" => PermissionDecision::Deny,
            _ => {
                tracing::warn!(action = %interaction.action_id, "Unknown interaction action");
                return Ok(());
            }
        };

        self.sessions
            .update_status(&interaction.conversation_id, SessionStatus::Running)
            .await?;

        tracing::info!(
            conversation = %interaction.conversation_id.0,
            decision = ?decision,
            "Permission decision recorded"
        );

        Ok(())
    }
}
