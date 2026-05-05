use crate::domain::channel_events::{
    ChannelIdentity, ConversationId, IncomingEvent, InteractionEvent, Platform, TextMessage,
};

use super::super::server::is_bot_command;
use super::webhook::{DiscordInteraction, DiscordInteractionType};

/// Convert a raw Discord interaction into a domain [`IncomingEvent`].
///
/// Returns `None` for interaction types we do not handle (e.g. Ping).
pub fn normalize_interaction(interaction: &DiscordInteraction) -> Option<IncomingEvent> {
    let channel_id = interaction.channel_id.as_deref().unwrap_or_default();
    let user_id = interaction.user_id.as_deref().unwrap_or_default();
    let conversation_id = ConversationId::from_platform(Platform::Discord, channel_id);

    let channel = ChannelIdentity::new(
        Platform::Discord,
        channel_id.to_string(),
        user_id.to_string(),
        None,
        None,
    );

    match interaction.interaction_type {
        DiscordInteractionType::Ping => None,

        DiscordInteractionType::ApplicationCommand => {
            let data = interaction.data.as_ref();
            let cmd_name = data.and_then(|d| d.name.as_deref()).unwrap_or("");

            if is_bot_command(cmd_name) {
                let args = extract_command_args(interaction);
                Some(IncomingEvent::BotCommand {
                    command: format!("/{cmd_name}"),
                    args,
                    channel,
                    conversation_id,
                })
            } else {
                let text = extract_command_text(interaction)?;
                Some(IncomingEvent::TextMessage(TextMessage {
                    conversation_id,
                    channel,
                    text,
                    reply_to_id: None,
                }))
            }
        }

        DiscordInteractionType::MessageComponent => {
            let data = interaction.data.as_ref()?;
            let action_id = data.custom_id.clone().unwrap_or_default();
            let message_ref = interaction.id.clone();
            Some(IncomingEvent::Interaction(InteractionEvent {
                conversation_id,
                channel,
                action_id,
                message_ref,
                callback_message_id: None,
                callback_query_id: None,
            }))
        }
    }
}

/// Normalize a MESSAGE_CREATE gateway event into a domain [`IncomingEvent`].
pub fn normalize_gateway_message(data: &serde_json::Value) -> Option<IncomingEvent> {
    let content = data.get("content").and_then(|c| c.as_str()).unwrap_or("");
    if content.is_empty() {
        return None;
    }

    let channel_id = data
        .get("channel_id")
        .and_then(|c| c.as_str())
        .unwrap_or("");
    let user_id = data
        .get("author")
        .and_then(|a| a.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("");

    let conversation_id = ConversationId::from_platform(Platform::Discord, channel_id);
    let guild_id = data
        .get("guild_id")
        .and_then(|g| g.as_str())
        .map(String::from);
    let channel = ChannelIdentity::new(
        Platform::Discord,
        channel_id.to_string(),
        user_id.to_string(),
        None,
        guild_id,
    );

    Some(IncomingEvent::TextMessage(TextMessage {
        conversation_id,
        channel,
        text: content.to_string(),
        reply_to_id: None,
    }))
}

/// Extract the user's text from the first option value of a slash command.
fn extract_command_text(interaction: &DiscordInteraction) -> Option<String> {
    interaction
        .data
        .as_ref()?
        .options
        .as_ref()?
        .first()?
        .value
        .clone()
}

/// Extract the first option value as command args (for bot commands like /model).
fn extract_command_args(interaction: &DiscordInteraction) -> String {
    interaction
        .data
        .as_ref()
        .and_then(|d| d.options.as_ref())
        .and_then(|opts| opts.first())
        .and_then(|opt| opt.value.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::channel::discord::webhook::{DiscordInteractionData, DiscordOption};

    fn base_interaction(itype: DiscordInteractionType) -> DiscordInteraction {
        DiscordInteraction {
            interaction_type: itype,
            id: "interaction-1".into(),
            token: "tok".into(),
            channel_id: Some("channel-42".into()),
            user_id: Some("user-7".into()),
            data: None,
        }
    }

    #[test]
    fn ping_returns_none() {
        let interaction = base_interaction(DiscordInteractionType::Ping);
        assert!(normalize_interaction(&interaction).is_none());
    }

    #[test]
    fn known_command_produces_bot_command() {
        let mut interaction = base_interaction(DiscordInteractionType::ApplicationCommand);
        interaction.data = Some(DiscordInteractionData {
            name: Some("cancel".into()),
            options: None,
            custom_id: None,
            component_type: None,
        });

        let event = normalize_interaction(&interaction).expect("some event");
        match event {
            IncomingEvent::BotCommand { command, args, .. } => {
                assert_eq!(command, "/cancel");
                assert_eq!(args, "");
            }
            other => panic!("expected BotCommand, got {:?}", other),
        }
    }

    #[test]
    fn known_command_with_args_produces_bot_command() {
        let mut interaction = base_interaction(DiscordInteractionType::ApplicationCommand);
        interaction.data = Some(DiscordInteractionData {
            name: Some("model".into()),
            options: Some(vec![DiscordOption {
                name: "name".into(),
                value: Some("sonnet".into()),
            }]),
            custom_id: None,
            component_type: None,
        });

        let event = normalize_interaction(&interaction).expect("some event");
        match event {
            IncomingEvent::BotCommand { command, args, .. } => {
                assert_eq!(command, "/model");
                assert_eq!(args, "sonnet");
            }
            other => panic!("expected BotCommand, got {:?}", other),
        }
    }

    #[test]
    fn unknown_command_produces_text_message() {
        let mut interaction = base_interaction(DiscordInteractionType::ApplicationCommand);
        interaction.data = Some(DiscordInteractionData {
            name: Some("ask".into()),
            options: Some(vec![DiscordOption {
                name: "prompt".into(),
                value: Some("hello world".into()),
            }]),
            custom_id: None,
            component_type: None,
        });

        let event = normalize_interaction(&interaction).expect("some event");
        match event {
            IncomingEvent::TextMessage(msg) => {
                assert_eq!(msg.text, "hello world");
                assert_eq!(msg.channel.platform, Platform::Discord);
                assert_eq!(msg.channel.channel_id, "channel-42");
                assert_eq!(
                    msg.conversation_id,
                    ConversationId::from_platform(Platform::Discord, "channel-42")
                );
                assert!(msg.reply_to_id.is_none());
            }
            other => panic!("expected TextMessage, got {:?}", other),
        }
    }

    #[test]
    fn unknown_command_without_options_returns_none() {
        let mut interaction = base_interaction(DiscordInteractionType::ApplicationCommand);
        interaction.data = Some(DiscordInteractionData {
            name: Some("ask".into()),
            options: Some(vec![]),
            custom_id: None,
            component_type: None,
        });
        assert!(normalize_interaction(&interaction).is_none());
    }

    #[test]
    fn application_command_without_name_returns_none() {
        let mut interaction = base_interaction(DiscordInteractionType::ApplicationCommand);
        interaction.data = Some(DiscordInteractionData {
            name: None,
            options: Some(vec![]),
            custom_id: None,
            component_type: None,
        });
        assert!(normalize_interaction(&interaction).is_none());
    }

    #[test]
    fn message_component_produces_interaction_event() {
        let mut interaction = base_interaction(DiscordInteractionType::MessageComponent);
        interaction.data = Some(DiscordInteractionData {
            name: None,
            options: None,
            custom_id: Some("allow".into()),
            component_type: Some(2),
        });

        let event = normalize_interaction(&interaction).expect("some event");
        match event {
            IncomingEvent::Interaction(evt) => {
                assert_eq!(evt.action_id, "allow");
                assert_eq!(evt.message_ref, "interaction-1");
                assert_eq!(evt.channel.platform, Platform::Discord);
            }
            other => panic!("expected Interaction, got {:?}", other),
        }
    }

    #[test]
    fn message_component_missing_custom_id_uses_default() {
        let mut interaction = base_interaction(DiscordInteractionType::MessageComponent);
        interaction.data = Some(DiscordInteractionData {
            name: None,
            options: None,
            custom_id: None,
            component_type: Some(2),
        });

        let event = normalize_interaction(&interaction).expect("some event");
        match event {
            IncomingEvent::Interaction(evt) => {
                assert_eq!(evt.action_id, "");
            }
            other => panic!("expected Interaction, got {:?}", other),
        }
    }
}
