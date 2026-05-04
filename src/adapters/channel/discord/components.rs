use crate::domain::channel_events::InteractionButtons;

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
        .map(|btn| {
            let style = button_style(&btn.id);
            serde_json::json!({
                "type": 2,
                "style": style,
                "custom_id": btn.id,
                "label": btn.label,
            })
        })
        .collect();

    serde_json::json!({
        "type": 1,
        "components": discord_buttons,
    })
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

/// Wrap action-row component(s) into the top-level `components` array Discord expects.
pub fn to_components_value(buttons: &InteractionButtons) -> serde_json::Value {
    serde_json::json!([to_action_row(buttons)])
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
}
