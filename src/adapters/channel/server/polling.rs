use std::sync::Arc;

use crate::domain::channel_events::IncomingEvent;

use super::{AppState, is_authorized, process_event};

pub(super) async fn start_telegram_polling(state: Arc<AppState>, bot_token: String) {
    let api = crate::adapters::channel::telegram::api::TelegramApi::new(bot_token);
    let mut offset: Option<i64> = None;

    tracing::info!("Telegram polling started");

    loop {
        match api.get_updates(offset, 30).await {
            Ok(updates) => {
                for update in updates {
                    offset = Some(update._update_id + 1);

                    let event =
                        match crate::adapters::channel::telegram::normalize::normalize_update(
                            update,
                        ) {
                            Some(e) => e,
                            None => continue,
                        };

                    let user_id = match &event {
                        IncomingEvent::TextMessage(msg) => {
                            tracing::info!(
                                user_id = &msg.channel.user_id,
                                text = &msg.text,
                                "Received Telegram message"
                            );
                            &msg.channel.user_id
                        }
                        IncomingEvent::BotCommand {
                            command, channel, ..
                        } => {
                            tracing::info!(
                                user_id = &channel.user_id,
                                command,
                                "Received Telegram command"
                            );
                            &channel.user_id
                        }
                        IncomingEvent::Interaction(inter) => &inter.channel.user_id,
                        _ => continue,
                    };
                    if !is_authorized(&state, user_id) {
                        tracing::warn!(user_id, "Unauthorized user");
                        continue;
                    }

                    let s = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = process_event(&s, event).await {
                            tracing::error!(error = %e, "Failed to process event");
                        }
                    });
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "getUpdates error, retrying in 5s");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}
