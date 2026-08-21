use std::sync::LazyLock;

use regex::Regex;

use crate::config::registry::{GuardSettings, SecretPolicy};
use crate::config::vault::redact_credential;
use crate::ports::guard_ports::{ContentScanner, Finding, GuardAction, ScanReport};

/// Secret-detection patterns. Adapted from llm-kernel 0.20 `safety::sanitize`
/// (feature-gated off for claudy) with two divergences: token charsets exclude
/// quotes/JSON structure so redaction cannot corrupt the enclosing JSON, and
/// minimum lengths suppress prose false positives ("bearer token was rotated").
static SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"((?i:bearer|basic) )([A-Za-z0-9_\-.=+/]{16,})",
        r"|((?:password|token|key|secret|api_key|apikey|access_token|private_key)=)([A-Za-z0-9_\-./+=]{8,})",
        r"|(sk-[A-Za-z0-9_\-]{8,})",
        r"|(AKIA[A-Za-z0-9]{16})",
        r"|(gh[posu]_[A-Za-z0-9]{20,})",
        r"|(xox[bpas]-[A-Za-z0-9-]{10,})",
    ))
    .expect("SECRET_RE is valid")
});

/// MVP inline `ContentScanner`: JSON image-block surgery + regex secret scan.
pub struct RegexScanner {
    strip_images: bool,
    on_secret: SecretPolicy,
}

impl RegexScanner {
    pub fn new(settings: &GuardSettings) -> Self {
        RegexScanner {
            strip_images: settings.strip_images,
            on_secret: settings.on_secret,
        }
    }

    fn secret_action(&self) -> GuardAction {
        match self.on_secret {
            SecretPolicy::Allow => GuardAction::Allow,
            SecretPolicy::Redact => GuardAction::Redact,
            SecretPolicy::Warn => GuardAction::Warn,
            SecretPolicy::Block => GuardAction::Block,
        }
    }
}

impl ContentScanner for RegexScanner {
    fn scan(&self, body: &[u8], content_type: &str) -> ScanReport {
        if !content_type.starts_with("application/json") {
            return fail_open("non_json");
        }
        let mut root: serde_json::Value = match serde_json::from_slice(body) {
            Ok(v) => v,
            Err(_) => return fail_open("unparseable_json"),
        };

        let mut images_stripped = 0usize;
        if self.strip_images {
            images_stripped = replace_image_blocks(&mut root);
        }

        // Scan the original bytes when no surgery happened, so clean requests
        // round-trip byte-identical (serde_json reorders object keys on
        // re-serialization).
        let mut text = if images_stripped == 0 {
            match std::str::from_utf8(body) {
                Ok(s) => s.to_string(),
                Err(_) => return fail_open("non_json"),
            }
        } else {
            serde_json::to_string(&root).unwrap_or_default()
        };

        let mut findings = Vec::new();
        let mut redacted_any = false;
        if self.on_secret != SecretPolicy::Allow {
            let action = self.secret_action();
            let redact = action == GuardAction::Redact;
            text = SECRET_RE
                .replace_all(&text, |caps: &regex::Captures| {
                    let (kind, token) = classify(caps);
                    findings.push(Finding {
                        kind: kind.to_string(),
                        action,
                        preview: redact_credential(token),
                    });
                    redacted_any = redacted_any || redact;
                    // Keep the `Bearer `/`key=` prefix, replace only the token.
                    let prefix = caps
                        .get(1)
                        .map(|m| m.as_str().to_string())
                        .or_else(|| caps.get(3).map(|m| m.as_str().to_string()))
                        .unwrap_or_default();
                    if redact {
                        format!("{}[REDACTED:{}]", prefix, kind)
                    } else {
                        caps.get(0).unwrap().as_str().to_string()
                    }
                })
                .into_owned();
        }

        if images_stripped > 0 {
            findings.push(Finding {
                kind: "image".to_string(),
                action: if self.strip_images {
                    GuardAction::Redact
                } else {
                    GuardAction::Allow
                },
                preview: format!("{} block(s)", images_stripped),
            });
        }

        let redacted_body = if images_stripped > 0 || redacted_any {
            Some(text.into_bytes())
        } else {
            None
        };

        ScanReport {
            findings,
            redacted_body,
            images_stripped,
        }
    }
}

fn fail_open(kind: &str) -> ScanReport {
    ScanReport {
        findings: vec![Finding {
            kind: kind.to_string(),
            action: GuardAction::Warn,
            preview: String::new(),
        }],
        redacted_body: None,
        images_stripped: 0,
    }
}

/// Classify a SECRET_RE match and return (kind, matched token).
fn classify<'c>(caps: &regex::Captures<'c>) -> (&'static str, &'c str) {
    if let Some(m) = caps.get(2) {
        return ("auth_header", m.as_str());
    }
    if let Some(m) = caps.get(4) {
        return ("key_value", m.as_str());
    }
    if let Some(m) = caps.get(5) {
        return (
            if m.as_str().starts_with("sk-ant") {
                "anthropic_key"
            } else {
                "api_key"
            },
            m.as_str(),
        );
    }
    if let Some(m) = caps.get(6) {
        return ("aws_key", m.as_str());
    }
    if let Some(m) = caps.get(7) {
        return ("github_token", m.as_str());
    }
    if let Some(m) = caps.get(8) {
        return ("slack_token", m.as_str());
    }
    ("secret", "")
}

/// Replace every `{"type":"image", ...}` block with a text placeholder.
/// Generic recursive walk: legitimate image blocks appear in
/// `system`/`messages[*].content`/`tool_result.content` block arrays, and a
/// schema-shaped `{"type":"image"}` object elsewhere is vanishingly rare.
/// Replacement (not deletion) keeps block counts and tool_result validity.
fn replace_image_blocks(v: &mut serde_json::Value) -> usize {
    match v {
        serde_json::Value::Object(obj) => {
            if obj.get("type").and_then(|t| t.as_str()) == Some("image") {
                *v = serde_json::json!({
                    "type": "text",
                    "text": "[claudy-guard: image block removed]"
                });
                return 1;
            }
            obj.values_mut().map(replace_image_blocks).sum()
        }
        serde_json::Value::Array(items) => items.iter_mut().map(replace_image_blocks).sum(),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scanner(on_secret: SecretPolicy) -> RegexScanner {
        RegexScanner::new(&GuardSettings {
            strip_images: true,
            on_secret,
            trusted_providers: vec!["native".to_string()],
        })
    }

    #[test]
    fn strips_image_block_in_messages_content() {
        let body = serde_json::json!({
            "model": "glm-5",
            "messages": [
                {"role": "user", "content": [
                    {"type": "text", "text": "what is this"},
                    {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "aGk="}}
                ]}
            ]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert_eq!(report.images_stripped, 1);
        let out: serde_json::Value =
            serde_json::from_slice(&report.redacted_body.unwrap()).unwrap();
        let content = out["messages"][0]["content"].as_array().unwrap();
        assert_eq!(content.len(), 2, "replacement, not deletion");
        assert_eq!(content[1]["type"], "text");
        assert!(
            content[1]["text"]
                .as_str()
                .unwrap()
                .contains("image block removed")
        );
    }

    #[test]
    fn strips_image_block_in_tool_result_nested_content() {
        let body = serde_json::json!({
            "messages": [
                {"role": "user", "content": [
                    {"type": "tool_result", "tool_use_id": "t1", "content": [
                        {"type": "text", "text": "screenshot"},
                        {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "aGk="}}
                    ]}
                ]}
            ]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert_eq!(report.images_stripped, 1);
        let out: serde_json::Value =
            serde_json::from_slice(&report.redacted_body.unwrap()).unwrap();
        let nested = &out["messages"][0]["content"][0]["content"];
        assert_eq!(nested.as_array().unwrap().len(), 2);
        assert_eq!(nested[1]["type"], "text");
    }

    #[test]
    fn strips_image_block_in_system_block_array() {
        let body = serde_json::json!({
            "system": [
                {"type": "text", "text": "be helpful"},
                {"type": "image", "source": {"type": "url", "url": "https://x/1.png"}}
            ],
            "messages": []
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert_eq!(report.images_stripped, 1);
    }

    #[test]
    fn clean_body_returns_none_redacted_body() {
        let body = br#"{"model":"glm-5","messages":[{"role":"user","content":"hello"}]}"#;
        let report = scanner(SecretPolicy::Redact).scan(body, "application/json");
        assert!(report.redacted_body.is_none());
        assert!(report.findings.is_empty());
    }

    #[test]
    fn redacts_bearer_token_in_text_block() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "leak: Bearer abcdefghijklmnop123456 said hi"}]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert!(report.redacted_body.is_some());
        let out = String::from_utf8(report.redacted_body.unwrap()).unwrap();
        assert!(out.contains("[REDACTED:auth_header]"), "got: {out}");
        assert!(!out.contains("abcdefghijklmnop123456"));
        assert!(out.contains("Bearer "), "prefix preserved");
        assert_eq!(report.findings[0].kind, "auth_header");
        assert!(!report.findings[0].preview.contains("abcdefghijklmnop"));
    }

    #[test]
    fn redacts_sk_ant_key_and_labels_kind() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "key sk-ant-api03-abcdef1234567890abcdef was rotated"}]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert_eq!(report.findings[0].kind, "anthropic_key");
        let out = String::from_utf8(report.redacted_body.unwrap()).unwrap();
        assert!(out.contains("[REDACTED:anthropic_key]"));
    }

    #[test]
    fn redacts_slack_xoxb_token() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "xoxb-1234567890abcdefghij"}]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert_eq!(report.findings[0].kind, "slack_token");
        let out = String::from_utf8(report.redacted_body.unwrap()).unwrap();
        assert!(out.contains("[REDACTED:slack_token]"));
    }

    #[test]
    fn json_key_colon_value_pair_not_flagged() {
        // JSON `"api_key":"value"` has no `=` and a short value — no match.
        let body = br#"{"messages":[{"role":"user","content":"see api_key settings"}],"metadata":{"api_key":"short"}}"#;
        let report = scanner(SecretPolicy::Redact).scan(body, "application/json");
        assert!(report.findings.is_empty());
        assert!(report.redacted_body.is_none());
    }

    #[test]
    fn non_json_content_type_fails_open() {
        let report = scanner(SecretPolicy::Redact).scan(b"raw", "text/event-stream");
        assert_eq!(report.findings[0].kind, "non_json");
        assert!(report.redacted_body.is_none());
    }

    #[test]
    fn unparseable_json_fails_open() {
        let report = scanner(SecretPolicy::Redact).scan(b"{broken", "application/json");
        assert_eq!(report.findings[0].kind, "unparseable_json");
        assert!(report.redacted_body.is_none());
    }

    #[test]
    fn block_policy_returns_finding_without_redacted_body() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "leak: Bearer abcdefghijklmnop123456"}]
        });
        let report =
            scanner(SecretPolicy::Block).scan(body.to_string().as_bytes(), "application/json");
        assert_eq!(report.findings[0].action, GuardAction::Block);
        assert!(
            report.redacted_body.is_none(),
            "blocked body must not be rewritten/forwarded"
        );
    }

    #[test]
    fn redaction_keeps_json_parseable() {
        // Redacted span must never swallow the closing quote of a JSON string.
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "password=supersecret123 was in .env"}]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        let out = String::from_utf8(report.redacted_body.unwrap()).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&out).expect("JSON stays parseable");
        assert!(reparsed["messages"][0]["content"].is_string());
    }
}
