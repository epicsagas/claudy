use crate::domain::channel_events::InteractionButtons;

const DISCORD_MAX_BUTTONS_PER_ROW: usize = 5;
const DISCORD_MAX_LABEL_LEN: usize = 80;

/// Truncate label to Discord's 80-character limit, respecting char boundaries.
fn truncate_label(label: &str) -> String {
    match label.char_indices().nth(DISCORD_MAX_LABEL_LEN) {
        Some((idx, _)) => label[..idx].to_string(),
        None => label.to_string(),
    }
}

/// Build a single Discord button JSON value.
fn build_button_json(id: &str, label: &str) -> serde_json::Value {
    serde_json::json!({
        "type": 2,
        "style": button_style(id),
        "custom_id": id,
        "label": truncate_label(label),
    })
}

/// Convert domain [`InteractionButtons`] into a Discord action-row component JSON value.
///
/// Discord component schema (v10):
/// - Action row: `{ type: 1, components: [...] }`
/// - Button:     `{ type: 2, style: N, custom_id: "...", label: "..." }`
///
/// Style mapping uses button id heuristics:
/// - ids containing "allow" or "yes"  -> Primary (1)
/// - ids containing "deny" or "no"    -> Danger  (4)
/// - everything else                  -> Secondary (2)
pub fn to_action_row(buttons: &InteractionButtons) -> serde_json::Value {
    let discord_buttons: Vec<serde_json::Value> = buttons
        .buttons
        .iter()
        .map(|btn| build_button_json(&btn.id, &btn.label))
        .collect();

    serde_json::json!({
        "type": 1,
        "components": discord_buttons,
    })
}

/// Wrap action-row component(s) into the top-level `components` array Discord expects.
/// Splits into multiple action rows when exceeding Discord's 5-button-per-row limit.
pub fn to_components_value(buttons: &InteractionButtons) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = buttons
        .buttons
        .chunks(DISCORD_MAX_BUTTONS_PER_ROW)
        .map(|chunk| {
            let row_buttons: Vec<serde_json::Value> = chunk
                .iter()
                .map(|btn| build_button_json(&btn.id, &btn.label))
                .collect();
            serde_json::json!({
                "type": 1,
                "components": row_buttons,
            })
        })
        .collect();

    serde_json::json!(rows)
}

/// Determine the Discord button style from the button id.
fn button_style(id: &str) -> u8 {
    let lower = id.to_lowercase();
    if lower.contains("allow") || lower.contains("yes") {
        1 // Primary
    } else if lower.contains("deny") || lower.contains("no") {
        4 // Danger
    } else {
        2 // Secondary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_events::Button;

    fn sample_buttons() -> InteractionButtons {
        InteractionButtons {
            prompt_text: "Continue?".into(),
            buttons: vec![
                Button {
                    id: "allow".into(),
                    label: "Yes".into(),
                },
                Button {
                    id: "deny".into(),
                    label: "No".into(),
                },
                Button {
                    id: "maybe".into(),
                    label: "Maybe".into(),
                },
            ],
        }
    }

    #[test]
    fn action_row_structure() {
        let row = to_action_row(&sample_buttons());
        assert_eq!(row["type"], 1);

        let components = row["components"].as_array().expect("components is array");
        assert_eq!(components.len(), 3);

        // allow -> Primary (1)
        assert_eq!(components[0]["style"], 1);
        assert_eq!(components[0]["custom_id"], "allow");
        assert_eq!(components[0]["label"], "Yes");

        // deny -> Danger (4)
        assert_eq!(components[1]["style"], 4);

        // maybe -> Secondary (2)
        assert_eq!(components[2]["style"], 2);
    }

    #[test]
    fn to_components_wraps_in_array() {
        let val = to_components_value(&sample_buttons());
        let arr = val.as_array().expect("top-level is array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], 1);
    }

    #[test]
    fn button_style_cases() {
        assert_eq!(button_style("allow"), 1);
        assert_eq!(button_style("yes_confirm"), 1);
        assert_eq!(button_style("deny"), 4);
        assert_eq!(button_style("no_way"), 4);
        assert_eq!(button_style("other"), 2);
    }

    #[test]
    fn truncate_label_exactly_80_chars_unchanged() {
        let label: String = "a".repeat(80);
        assert_eq!(truncate_label(&label).len(), 80);
    }

    #[test]
    fn truncate_label_over_80_chars_truncated() {
        let label: String = "b".repeat(85);
        let result = truncate_label(&label);
        assert_eq!(result.chars().count(), 80);
    }

    #[test]
    fn truncate_label_under_80_chars_unchanged() {
        let label = "short";
        assert_eq!(truncate_label(label), "short");
    }

    #[test]
    fn truncate_label_multibyte_boundary_safe() {
        let label: String = "한".repeat(40); // 120 bytes, 40 chars — under 80
        assert_eq!(truncate_label(&label).chars().count(), 40);

        let label: String = "한".repeat(85); // 255 bytes, 85 chars — over 80
        let result = truncate_label(&label);
        assert_eq!(result.chars().count(), 80);
        assert!(result.is_char_boundary(result.len()));
    }

    #[test]
    fn to_components_5_buttons_single_row() {
        let buttons = InteractionButtons {
            prompt_text: "Pick".into(),
            buttons: (0..5)
                .map(|i| Button {
                    id: format!("btn_{i}"),
                    label: format!("Button {i}"),
                })
                .collect(),
        };
        let val = to_components_value(&buttons);
        let arr = val.as_array().expect("top-level is array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["components"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn to_components_6_buttons_splits_two_rows() {
        let buttons = InteractionButtons {
            prompt_text: "Pick".into(),
            buttons: (0..6)
                .map(|i| Button {
                    id: format!("btn_{i}"),
                    label: format!("Button {i}"),
                })
                .collect(),
        };
        let val = to_components_value(&buttons);
        let arr = val.as_array().expect("top-level is array");
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["components"].as_array().unwrap().len(), 5);
        assert_eq!(arr[1]["components"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn to_components_button_order_preserved_across_rows() {
        let buttons = InteractionButtons {
            prompt_text: "Pick".into(),
            buttons: (0..7)
                .map(|i| Button {
                    id: format!("btn_{i}"),
                    label: format!("Button {i}"),
                })
                .collect(),
        };
        let val = to_components_value(&buttons);
        let arr = val.as_array().expect("top-level is array");

        let row0_ids: Vec<&str> = arr[0]["components"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["custom_id"].as_str().unwrap())
            .collect();
        assert_eq!(row0_ids, vec!["btn_0", "btn_1", "btn_2", "btn_3", "btn_4"]);

        let row1_ids: Vec<&str> = arr[1]["components"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["custom_id"].as_str().unwrap())
            .collect();
        assert_eq!(row1_ids, vec!["btn_5", "btn_6"]);
    }

    #[test]
    fn to_components_long_label_truncated_to_80() {
        let buttons = InteractionButtons {
            prompt_text: "Pick".into(),
            buttons: vec![Button {
                id: "test".into(),
                label: "x".repeat(100),
            }],
        };
        let val = to_components_value(&buttons);
        let label = val[0]["components"][0]["label"].as_str().unwrap();
        assert_eq!(label.chars().count(), 80);
    }

    #[test]
    fn to_components_empty_buttons_returns_empty_array() {
        let buttons = InteractionButtons {
            prompt_text: "Empty".into(),
            buttons: vec![],
        };
        let val = to_components_value(&buttons);
        let arr = val.as_array().expect("top-level is array");
        assert!(arr.is_empty());
    }
}
