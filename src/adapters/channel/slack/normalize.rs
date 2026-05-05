use serde::Deserialize;

use crate::domain::channel_events::{
    ChannelIdentity, ConversationId, IncomingEvent, InteractionEvent, Platform, TextMessage,
};

// ---------------------------------------------------------------------------
// Slack event callback types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SlackEventCallback {
    pub token: String,
    pub team_id: String,
    pub api_app_id: String,
    pub event: SlackEvent,
    #[serde(rename = "type")]
    pub payload_type: String,
    pub event_id: Option<String>,
    pub event_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SlackEvent {
    #[serde(rename = "message")]
    Message(SlackMessageEvent),
    // Extend with other event types as needed.
}

#[derive(Debug, Deserialize)]
pub struct SlackMessageEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub channel: String,
    pub user: Option<String>,
    pub text: Option<String>,
    pub ts: Option<String>,
    pub thread_ts: Option<String>,
    /// Subtypes like "bot_message", "message_changed", etc.
    pub subtype: Option<String>,
}

// ---------------------------------------------------------------------------
// Slack interaction payload (block kit actions)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SlackInteractionPayload {
    pub type_field: Option<String>,
    #[serde(rename = "type")]
    pub payload_type: String,
    pub channel: Option<SlackInteractionChannel>,
    pub user: Option<SlackInteractionUser>,
    pub actions: Option<Vec<SlackAction>>,
    pub message: Option<SlackInteractionMessage>,
    pub response_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackInteractionChannel {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SlackInteractionUser {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SlackInteractionMessage {
    pub ts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub action_id: Option<String>,
    pub value: Option<String>,
}

// ---------------------------------------------------------------------------
// Normalization functions
// ---------------------------------------------------------------------------

/// Convert a Slack event callback into a domain `IncomingEvent`.
///
/// Returns `None` for non-message events, messages from bots, or messages
/// with no text content.
pub fn normalize_event(callback: &SlackEventCallback) -> Option<IncomingEvent> {
    let SlackEvent::Message(msg) = &callback.event;

    // Ignore bot messages and other subtypes.
    if msg.subtype.is_some() {
        return None;
    }

    let text = msg.text.as_deref()?.to_string();
    let channel_id = msg.channel.clone();
    let user_id = msg.user.as_deref()?.to_string();
    let _ts = msg.ts.as_deref()?.to_string();

    let channel = ChannelIdentity::new(
        Platform::Slack,
        channel_id.clone(),
        user_id.clone(),
        msg.thread_ts.clone(),
        None,
    );

    let conversation_id = ConversationId::from_platform(Platform::Slack, &channel_id);
    let reply_to_id = msg.thread_ts.clone();

    Some(IncomingEvent::TextMessage(TextMessage {
        conversation_id,
        channel,
        text,
        reply_to_id,
    }))
}

/// Convert a Slack interaction payload into a domain `IncomingEvent`.
///
/// Returns `None` if the payload has no actions or no channel/user info.
pub fn normalize_interaction(payload: &SlackInteractionPayload) -> Option<IncomingEvent> {
    let actions = payload.actions.as_ref()?;
    let first_action = actions.first()?;

    let action_id = first_action.action_id.as_deref()?;

    let slack_channel = payload.channel.as_ref()?;
    let slack_user = payload.user.as_ref()?;
    let slack_message = payload.message.as_ref()?;
    let ts = slack_message.ts.as_deref()?;

    let channel = ChannelIdentity::new(
        Platform::Slack,
        slack_channel.id.clone(),
        slack_user.id.clone(),
        None,
        None,
    );

    let conversation_id = ConversationId::from_platform(Platform::Slack, &slack_channel.id);
    let message_ref = format!("{}:{}", slack_channel.id, ts);

    Some(IncomingEvent::Interaction(InteractionEvent {
        conversation_id,
        channel,
        action_id: action_id.to_string(),
        message_ref,
        callback_message_id: None,
        callback_query_id: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message_event(channel: &str, user: &str, text: &str, ts: &str) -> SlackEventCallback {
        SlackEventCallback {
            token: "test_token".to_string(),
            team_id: "T123".to_string(),
            api_app_id: "A123".to_string(),
            event: SlackEvent::Message(SlackMessageEvent {
                event_type: "message".to_string(),
                channel: channel.to_string(),
                user: Some(user.to_string()),
                text: Some(text.to_string()),
                ts: Some(ts.to_string()),
                thread_ts: None,
                subtype: None,
            }),
            payload_type: "event_callback".to_string(),
            event_id: Some("Ev123".to_string()),
            event_time: Some(1234567890),
        }
    }

    #[test]
    fn normalize_event_basic_message() {
        let callback = make_message_event("C123", "U456", "hello world", "1234567890.123456");
        let event = normalize_event(&callback).expect("should normalize");

        match event {
            IncomingEvent::TextMessage(msg) => {
                assert_eq!(msg.text, "hello world");
                assert_eq!(msg.channel.channel_id, "C123");
                assert_eq!(msg.channel.user_id, "U456");
                assert_eq!(msg.channel.platform, Platform::Slack);
                assert!(msg.conversation_id.0.starts_with("slack-C123"));
            }
            _ => panic!("expected TextMessage"),
        }
    }

    #[test]
    fn normalize_event_with_thread() {
        let mut callback = make_message_event("C123", "U456", "reply", "1234567890.123456");
        let SlackEvent::Message(ref mut msg) = callback.event;
        msg.thread_ts = Some("1234567890.000000".to_string());

        let event = normalize_event(&callback).expect("should normalize");
        match event {
            IncomingEvent::TextMessage(msg) => {
                assert_eq!(msg.reply_to_id, Some("1234567890.000000".to_string()));
                assert_eq!(msg.channel.thread_id, Some("1234567890.000000".to_string()));
            }
            _ => panic!("expected TextMessage"),
        }
    }

    #[test]
    fn normalize_event_ignores_bot_messages() {
        let mut callback = make_message_event("C123", "U456", "bot says hi", "1234567890.123456");
        let SlackEvent::Message(ref mut msg) = callback.event;
        msg.subtype = Some("bot_message".to_string());
        assert!(normalize_event(&callback).is_none());
    }

    #[test]
    fn normalize_event_ignores_no_text() {
        let mut callback = make_message_event("C123", "U456", "has text", "1234567890.123456");
        let SlackEvent::Message(ref mut msg) = callback.event;
        msg.text = None;
        assert!(normalize_event(&callback).is_none());
    }

    #[test]
    fn normalize_event_ignores_no_user() {
        let mut callback = make_message_event("C123", "U456", "text", "1234567890.123456");
        let SlackEvent::Message(ref mut msg) = callback.event;
        msg.user = None;
        assert!(normalize_event(&callback).is_none());
    }

    #[test]
    fn normalize_interaction_basic() {
        let payload = SlackInteractionPayload {
            type_field: None,
            payload_type: "block_actions".to_string(),
            channel: Some(SlackInteractionChannel {
                id: "C789".to_string(),
            }),
            user: Some(SlackInteractionUser {
                id: "U012".to_string(),
            }),
            actions: Some(vec![SlackAction {
                action_type: "button".to_string(),
                action_id: Some("approve_btn".to_string()),
                value: Some("yes".to_string()),
            }]),
            message: Some(SlackInteractionMessage {
                ts: Some("1234567890.654321".to_string()),
            }),
            response_url: None,
        };

        let event = normalize_interaction(&payload).expect("should normalize");
        match event {
            IncomingEvent::Interaction(interaction) => {
                assert_eq!(interaction.action_id, "approve_btn");
                assert_eq!(interaction.message_ref, "C789:1234567890.654321");
                assert_eq!(interaction.channel.channel_id, "C789");
                assert_eq!(interaction.channel.user_id, "U012");
            }
            _ => panic!("expected Interaction"),
        }
    }

    #[test]
    fn normalize_interaction_no_actions() {
        let payload = SlackInteractionPayload {
            type_field: None,
            payload_type: "block_actions".to_string(),
            channel: Some(SlackInteractionChannel {
                id: "C789".to_string(),
            }),
            user: Some(SlackInteractionUser {
                id: "U012".to_string(),
            }),
            actions: None,
            message: Some(SlackInteractionMessage {
                ts: Some("1234567890.654321".to_string()),
            }),
            response_url: None,
        };
        assert!(normalize_interaction(&payload).is_none());
    }

    #[test]
    fn normalize_interaction_no_channel() {
        let payload = SlackInteractionPayload {
            type_field: None,
            payload_type: "block_actions".to_string(),
            channel: None,
            user: Some(SlackInteractionUser {
                id: "U012".to_string(),
            }),
            actions: Some(vec![SlackAction {
                action_type: "button".to_string(),
                action_id: Some("btn".to_string()),
                value: None,
            }]),
            message: Some(SlackInteractionMessage {
                ts: Some("1234567890.654321".to_string()),
            }),
            response_url: None,
        };
        assert!(normalize_interaction(&payload).is_none());
    }
}
