use crate::domain::channel_events::InteractionButtons;

/// Convert domain `InteractionButtons` into a Slack Block Kit JSON value.
///
/// Produces:
/// - A **section** block with the `prompt_text` as markdown.
/// - An **actions** block containing one `button` element per `Button`.
pub fn to_block_kit(buttons: &InteractionButtons) -> serde_json::Value {
    let block_elements: Vec<serde_json::Value> = buttons
        .buttons
        .iter()
        .map(|btn| {
            serde_json::json!({
                "type": "button",
                "action_id": btn.id,
                "text": {
                    "type": "plain_text",
                    "text": btn.label,
                },
            })
        })
        .collect();

    serde_json::json!([
        {
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": buttons.prompt_text,
            },
        },
        {
            "type": "actions",
            "elements": block_elements,
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_events::Button;

    fn make_buttons(prompt: &str, labels: &[(&str, &str)]) -> InteractionButtons {
        InteractionButtons {
            prompt_text: prompt.to_string(),
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
    fn to_block_kit_single_button() {
        let interaction = make_buttons("Choose one:", &[("yes", "Yes")]);
        let blocks = to_block_kit(&interaction);

        let blocks_arr = blocks.as_array().expect("should be array");
        assert_eq!(blocks_arr.len(), 2);

        // Section block.
        let section = &blocks_arr[0];
        assert_eq!(section["type"], "section");
        assert_eq!(section["text"]["type"], "mrkdwn");
        assert_eq!(section["text"]["text"], "Choose one:");

        // Actions block.
        let actions = &blocks_arr[1];
        assert_eq!(actions["type"], "actions");
        let elements = actions["elements"]
            .as_array()
            .expect("elements should be array");
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0]["type"], "button");
        assert_eq!(elements[0]["action_id"], "yes");
        assert_eq!(elements[0]["text"]["text"], "Yes");
    }

    #[test]
    fn to_block_kit_multiple_buttons() {
        let interaction = make_buttons(
            "Pick an option:",
            &[
                ("approve", "Approve"),
                ("deny", "Deny"),
                ("review", "Needs Review"),
            ],
        );
        let blocks = to_block_kit(&interaction);

        let actions = &blocks.as_array().expect("should be array")[1];
        let elements = actions["elements"]
            .as_array()
            .expect("elements should be array");
        assert_eq!(elements.len(), 3);

        assert_eq!(elements[0]["action_id"], "approve");
        assert_eq!(elements[0]["text"]["text"], "Approve");

        assert_eq!(elements[1]["action_id"], "deny");
        assert_eq!(elements[1]["text"]["text"], "Deny");

        assert_eq!(elements[2]["action_id"], "review");
        assert_eq!(elements[2]["text"]["text"], "Needs Review");
    }

    #[test]
    fn to_block_kit_empty_buttons() {
        let interaction = make_buttons("No buttons", &[]);
        let blocks = to_block_kit(&interaction);

        let actions = &blocks.as_array().expect("should be array")[1];
        let elements = actions["elements"]
            .as_array()
            .expect("elements should be array");
        assert!(elements.is_empty());
    }

    #[test]
    fn to_block_kit_roundtrip_json() {
        let interaction = make_buttons("Proceed?", &[("go", "Go")]);
        let blocks = to_block_kit(&interaction);

        // Verify the output is valid JSON that can round-trip.
        let serialized = serde_json::to_string(&blocks).expect("should serialize");
        let deserialized: serde_json::Value =
            serde_json::from_str(&serialized).expect("should deserialize");
        assert_eq!(blocks, deserialized);
    }
}
