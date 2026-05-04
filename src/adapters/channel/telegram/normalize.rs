use serde::Deserialize;

use crate::domain::channel_events::{
    ChannelIdentity, ConversationId, IncomingEvent, InteractionEvent, Platform, TextMessage,
};

/// Telegram API types for deserializing webhook updates.
#[derive(Debug, Deserialize)]
pub struct TelegramUpdate {
    #[serde(rename = "update_id")]
    pub _update_id: i64,
    pub message: Option<TelegramMessage>,
    pub callback_query: Option<TelegramCallbackQuery>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
    #[serde(rename = "message_id")]
    pub _message_id: i64,
    pub chat: TelegramChat,
    pub from: Option<TelegramFrom>,
    pub text: Option<String>,
    pub reply_to_message: Option<Box<TelegramMessage>>,
    pub entities: Option<Vec<TelegramEntity>>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct TelegramFrom {
    pub id: i64,
    #[serde(rename = "first_name")]
    pub _first_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramCallbackQuery {
    pub id: String,
    pub from: TelegramFrom,
    pub message: Option<TelegramMessage>,
    pub data: Option<String>,
}

/// Convert a raw Telegram update into a domain `IncomingEvent`.
///
/// Returns `None` for updates that are not text messages or callback interactions.
pub fn normalize_update(update: TelegramUpdate) -> Option<IncomingEvent> {
    if let Some(message) = update.message {
        return normalize_text_message(message);
    }

    if let Some(callback) = update.callback_query {
        return normalize_callback(callback);
    }

    None
}

fn normalize_text_message(message: TelegramMessage) -> Option<IncomingEvent> {
    let text = message.text.clone()?;

    let chat_id = message.chat.id.to_string();
    let user_id = message
        .from
        .as_ref()
        .map(|f| f.id.to_string())
        .unwrap_or_default();

    let conversation_id = ConversationId::from_platform(Platform::Telegram, &chat_id);

    let channel = ChannelIdentity {
        platform: Platform::Telegram,
        channel_id: chat_id,
        user_id,
        thread_id: None,
        guild_id: None,
    };

    if let Some((command, args)) = extract_command(&message) {
        return Some(IncomingEvent::BotCommand {
            command,
            args,
            channel,
            conversation_id,
        });
    }

    let reply_to_id = message
        .reply_to_message
        .map(|reply| reply._message_id.to_string());

    Some(IncomingEvent::TextMessage(TextMessage {
        conversation_id,
        channel,
        text,
        reply_to_id,
    }))
}

pub fn extract_command(msg: &TelegramMessage) -> Option<(String, String)> {
    let text = msg.text.as_ref()?;
    let entity = msg
        .entities
        .as_ref()?
        .iter()
        .find(|e| e.entity_type == "bot_command")?;
    let cmd = text.get(entity.offset..entity.offset + entity.length)?;
    let args = text
        .get(entity.offset + entity.length..)
        .unwrap_or("")
        .trim()
        .to_string();
    Some((cmd.to_string(), args))
}

fn normalize_callback(callback: TelegramCallbackQuery) -> Option<IncomingEvent> {
    let data = callback.data?;
    let message = callback.message?;

    let chat_id = message.chat.id.to_string();
    let user_id = callback.from.id.to_string();

    let conversation_id = ConversationId::from_platform(Platform::Telegram, &chat_id);

    // Parse callback_data in the form "action:_message_id" (e.g. "allow:msg123")
    let (action_id, message_ref) = match data.split_once(':') {
        Some((action, ref_id)) => (action.to_string(), ref_id.to_string()),
        None => (data, String::new()),
    };

    Some(IncomingEvent::Interaction(InteractionEvent {
        conversation_id,
        channel: ChannelIdentity {
            platform: Platform::Telegram,
            channel_id: chat_id,
            user_id,
            thread_id: None,
            guild_id: None,
        },
        action_id,
        message_ref,
        callback_message_id: Some(message._message_id),
        callback_query_id: Some(callback.id),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_text_update(chat_id: i64, user_id: i64, text: &str) -> TelegramUpdate {
        TelegramUpdate {
            _update_id: 1,
            message: Some(TelegramMessage {
                _message_id: 100,
                chat: TelegramChat { id: chat_id },
                from: Some(TelegramFrom {
                    id: user_id,
                    _first_name: Some("Test".to_string()),
                }),
                text: Some(text.to_string()),
                reply_to_message: None,
                entities: None,
            }),
            callback_query: None,
        }
    }

    fn make_callback_update(chat_id: i64, user_id: i64, data: &str) -> TelegramUpdate {
        TelegramUpdate {
            _update_id: 2,
            message: None,
            callback_query: Some(TelegramCallbackQuery {
                id: "cb123".to_string(),
                from: TelegramFrom {
                    id: user_id,
                    _first_name: Some("Test".to_string()),
                },
                message: Some(TelegramMessage {
                    _message_id: 200,
                    chat: TelegramChat { id: chat_id },
                    from: None,
                    text: None,
                    reply_to_message: None,
                    entities: None,
                }),
                data: Some(data.to_string()),
            }),
        }
    }

    #[test]
    fn normalize_text_message_returns_text_event() {
        let update = make_text_update(42, 99, "hello world");
        let event = normalize_update(update).unwrap();

        match event {
            IncomingEvent::TextMessage(msg) => {
                assert_eq!(msg.text, "hello world");
                assert_eq!(msg.conversation_id.0, "telegram-42");
                assert_eq!(msg.channel.channel_id, "42");
                assert_eq!(msg.channel.user_id, "99");
                assert_eq!(msg.channel.platform, Platform::Telegram);
                assert!(msg.reply_to_id.is_none());
            }
            _ => panic!("expected TextMessage variant"),
        }
    }

    #[test]
    fn normalize_text_message_without_from_uses_default_user_id() {
        let mut update = make_text_update(42, 99, "hi");
        update.message.as_mut().unwrap().from = None;

        let event = normalize_update(update).unwrap();
        match event {
            IncomingEvent::TextMessage(msg) => {
                assert_eq!(msg.channel.user_id, "");
            }
            _ => panic!("expected TextMessage variant"),
        }
    }

    #[test]
    fn normalize_text_message_with_reply_to() {
        let mut update = make_text_update(42, 99, "reply");
        update.message.as_mut().unwrap().reply_to_message = Some(Box::new(TelegramMessage {
            _message_id: 50,
            chat: TelegramChat { id: 42 },
            from: None,
            text: Some("original".to_string()),
            reply_to_message: None,
            entities: None,
        }));

        let event = normalize_update(update).unwrap();
        match event {
            IncomingEvent::TextMessage(msg) => {
                assert_eq!(msg.reply_to_id.as_deref(), Some("50"));
            }
            _ => panic!("expected TextMessage variant"),
        }
    }

    #[test]
    fn normalize_callback_returns_interaction_event() {
        let update = make_callback_update(42, 99, "allow:msg123");
        let event = normalize_update(update).unwrap();

        match event {
            IncomingEvent::Interaction(inter) => {
                assert_eq!(inter.action_id, "allow");
                assert_eq!(inter.message_ref, "msg123");
                assert_eq!(inter.conversation_id.0, "telegram-42");
                assert_eq!(inter.channel.channel_id, "42");
                assert_eq!(inter.channel.user_id, "99");
            }
            _ => panic!("expected Interaction variant"),
        }
    }

    #[test]
    fn normalize_callback_without_colon_uses_full_data_as_action() {
        let update = make_callback_update(42, 99, "simple_action");
        let event = normalize_update(update).unwrap();

        match event {
            IncomingEvent::Interaction(inter) => {
                assert_eq!(inter.action_id, "simple_action");
                assert_eq!(inter.message_ref, "");
            }
            _ => panic!("expected Interaction variant"),
        }
    }

    #[test]
    fn normalize_message_without_text_returns_none() {
        let mut update = make_text_update(42, 99, "will be removed");
        update.message.as_mut().unwrap().text = None;

        let result = normalize_update(update);
        assert!(result.is_none());
    }

    #[test]
    fn normalize_update_with_no_message_or_callback_returns_none() {
        let update = TelegramUpdate {
            _update_id: 999,
            message: None,
            callback_query: None,
        };

        assert!(normalize_update(update).is_none());
    }
}
