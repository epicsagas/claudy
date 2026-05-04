use crate::domain::channel_events::Button;

pub struct ResponseAnalysis {
    pub needs_interaction: bool,
    pub buttons: Vec<Button>,
}

/// Auto-continue patterns — only these trigger the timer in YOLO mode.
const AUTO_CONTINUE_MARKERS: &[&str] = &[
    "proceed?",
    "continue?",
    "go ahead?",
    "shall i proceed",
    "shall i continue",
    "should i proceed",
    "should i continue",
    "shall we proceed",
    "shall we continue",
    "want me to continue",
    "want me to proceed",
];

/// Patterns that indicate a yes/no question.
const YES_NO_MARKERS: &[&str] = &[
    "shall i",
    "should i",
    "do you want",
    "would you like",
    "can i",
    "may i",
    "will you",
    "is that",
    "are you sure",
    "do you agree",
    "shall we",
    "proceed?",
    "continue?",
    "go ahead?",
];

pub fn analyze_response(text: &str) -> ResponseAnalysis {
    let lower = text.to_lowercase();
    let mut buttons = Vec::new();

    // Detect numbered options (1. ... 2. ... 3. ...)
    let numbered = detect_numbered_options(text);
    if !numbered.is_empty() {
        for (n, opt_text) in &numbered {
            // Trim option text for button label; use number + text in callback
            let label: String = opt_text.chars().take(30).collect();
            let choice_text = format!("{}. {}", n, opt_text);
            // Telegram callback_data is 64 bytes max; truncate safely
            let callback = format!("choice:{}", truncate_bytes(&choice_text, 55));
            buttons.push(Button {
                id: callback,
                label,
            });
        }
    } else if has_yes_no_pattern(&lower) {
        // Yes/No buttons
        buttons.push(Button {
            id: "choice:yes".into(),
            label: "Yes".into(),
        });
        buttons.push(Button {
            id: "choice:no".into(),
            label: "No".into(),
        });
    }

    // Detect any question mark
    let has_question = lower.contains('?');

    // Always add Reply button
    buttons.push(Button {
        id: "reply".into(),
        label: "Reply".into(),
    });

    let needs_interaction = has_question || !numbered.is_empty();

    ResponseAnalysis {
        needs_interaction,
        buttons,
    }
}

/// Whether the response matches patterns suitable for auto-continue (YOLO mode only).
pub fn is_auto_continuable(text: &str) -> bool {
    let lower = text.to_lowercase();
    AUTO_CONTINUE_MARKERS.iter().any(|m| lower.contains(m))
}

fn has_yes_no_pattern(lower: &str) -> bool {
    YES_NO_MARKERS.iter().any(|m| lower.contains(m))
}

/// Detect lines starting with "N. " (1-4) and return (number, option_text) pairs.
fn detect_numbered_options(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        for n in 1u32..=4 {
            for prefix in [format!("{}. ", n), format!("{}) ", n), format!("{}: ", n)] {
                if let Some(rest) = trimmed.strip_prefix(&prefix)
                    && !found.iter().any(|(k, _)| k == &n.to_string())
                {
                    let opt_text = rest.split('\n').next().unwrap_or(rest).to_string();
                    found.push((n.to_string(), opt_text));
                }
            }
        }
    }
    found
}

/// Truncate a string to fit within `max_bytes` UTF-8 bytes.
fn truncate_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yes_no_detection() {
        let analysis = analyze_response("Shall I proceed with the implementation?");
        assert!(analysis.needs_interaction);
        let ids: Vec<&str> = analysis.buttons.iter().map(|b| b.id.as_str()).collect();
        assert!(ids.contains(&"choice:yes"));
        assert!(ids.contains(&"choice:no"));
        assert!(ids.contains(&"reply"));
    }

    #[test]
    fn test_numbered_options() {
        let text = "Here are the options:\n1. Use Redis\n2. Use SQLite\n3. Use in-memory";
        let analysis = analyze_response(text);
        assert!(analysis.needs_interaction);
        let ids: Vec<&str> = analysis.buttons.iter().map(|b| b.id.as_str()).collect();
        assert!(ids.iter().any(|id| id.starts_with("choice:1.")));
        assert!(ids.iter().any(|id| id.starts_with("choice:2.")));
        assert!(ids.iter().any(|id| id.starts_with("choice:3.")));
        assert!(ids.contains(&"reply"));
        // Verify option text is in the label
        assert!(
            analysis
                .buttons
                .iter()
                .any(|b| b.label.contains("Use Redis"))
        );
    }

    #[test]
    fn test_generic_question() {
        let analysis = analyze_response("What framework should we use?");
        assert!(analysis.needs_interaction);
        let ids: Vec<&str> = analysis.buttons.iter().map(|b| b.id.as_str()).collect();
        // No yes/no, no numbered — just Reply
        assert!(ids.contains(&"reply"));
        assert!(!ids.iter().any(|id| id.starts_with("choice:")));
    }

    #[test]
    fn test_no_question() {
        let analysis = analyze_response("The file has been updated successfully.");
        assert!(!analysis.needs_interaction);
        // Still has Reply button
        assert!(analysis.buttons.iter().any(|b| b.id == "reply"));
    }

    #[test]
    fn test_auto_continuable() {
        assert!(is_auto_continuable("Shall I proceed with the changes?"));
        assert!(is_auto_continuable("Should I continue?"));
        assert!(!is_auto_continuable("What is your name?"));
    }

    #[test]
    fn test_numbered_options_max_four() {
        let text = "1. A\n2. B\n3. C\n4. D\n5. E";
        let analysis = analyze_response(text);
        let choice_count = analysis
            .buttons
            .iter()
            .filter(|b| b.id.starts_with("choice:"))
            .count();
        assert_eq!(choice_count, 4);
    }

    #[test]
    fn test_paren_numbered() {
        let text = "1) First option\n2) Second option";
        let analysis = analyze_response(text);
        let ids: Vec<&str> = analysis.buttons.iter().map(|b| b.id.as_str()).collect();
        assert!(ids.iter().any(|id| id.starts_with("choice:1")));
        assert!(ids.iter().any(|id| id.starts_with("choice:2")));
        assert!(
            analysis
                .buttons
                .iter()
                .any(|b| b.label.contains("First option"))
        );
    }

    #[test]
    fn test_numbered_takes_precedence_over_yes_no() {
        let text = "Shall I proceed?\n1. Use Redis\n2. Use SQLite";
        let analysis = analyze_response(text);
        let ids: Vec<&str> = analysis.buttons.iter().map(|b| b.id.as_str()).collect();
        // Numbered options should win over yes/no
        assert!(ids.iter().any(|id| id.starts_with("choice:1.")));
        assert!(!ids.contains(&"choice:yes"));
    }
}
