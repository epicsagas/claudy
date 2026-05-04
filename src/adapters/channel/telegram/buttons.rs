use crate::domain::channel_events::InteractionButtons;

/// Build a Telegram InlineKeyboardMarkup JSON from domain interaction buttons.
///
/// Produces: `{"inline_keyboard": [[{"text": "...", "callback_data": "..."}]]}`
/// Each button is placed in its own row for a vertical layout.
pub fn to_inline_keyboard(buttons: &InteractionButtons) -> serde_json::Value {
    let rows: Vec<Vec<serde_json::Value>> = buttons
        .buttons
        .iter()
        .map(|btn| {
            vec![serde_json::json!({
                "text": btn.label,
                "callback_data": btn.id,
            })]
        })
        .collect();

    serde_json::json!({ "inline_keyboard": rows })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_events::Button;

    fn make_buttons(labels: &[(&str, &str)]) -> InteractionButtons {
        InteractionButtons {
            prompt_text: "Choose".to_string(),
            buttons: labels
                .iter()
                .map(|(id, label)| Button {
                    id: id.to_string(),
                    label: label.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn single_button_produces_single_row() {
        let buttons = make_buttons(&[("allow:123", "Allow")]);
        let kb = to_inline_keyboard(&buttons);

        let keyboard = kb["inline_keyboard"].as_array().unwrap();
        assert_eq!(keyboard.len(), 1);
        let row = keyboard[0].as_array().unwrap();
        assert_eq!(row.len(), 1);
        assert_eq!(row[0]["text"], "Allow");
        assert_eq!(row[0]["callback_data"], "allow:123");
    }

    #[test]
    fn multiple_buttons_produce_multiple_rows() {
        let buttons = make_buttons(&[("allow:1", "Allow"), ("deny:1", "Deny")]);
        let kb = to_inline_keyboard(&buttons);

        let keyboard = kb["inline_keyboard"].as_array().unwrap();
        assert_eq!(keyboard.len(), 2);
        assert_eq!(keyboard[0].as_array().unwrap()[0]["text"], "Allow");
        assert_eq!(keyboard[1].as_array().unwrap()[0]["text"], "Deny");
    }

    #[test]
    fn empty_buttons_produces_empty_keyboard() {
        let buttons = InteractionButtons {
            prompt_text: "No buttons".to_string(),
            buttons: vec![],
        };
        let kb = to_inline_keyboard(&buttons);

        let keyboard = kb["inline_keyboard"].as_array().unwrap();
        assert!(keyboard.is_empty());
    }
}
