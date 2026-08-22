use crate::config::registry::{GuardSettings, SecretPolicy};
use crate::config::vault::redact_credential;
use crate::ports::guard_ports::{ContentScanner, Finding, GuardAction, ScanReport};

use llm_kernel::dlp::{self, FindingCategory};

/// MVP engine binding: llm-kernel 0.29 `dlp` L1 scan (18 rules — secrets,
/// Korean PII with RRN checksum, local filesystem paths) wrapped in the
/// claudy proxy contract.
///
/// Division of ownership with the kernel:
/// - non-JSON / unparseable-JSON fail-open markers stay proxy-side (the
///   kernel scans extracted text only);
/// - byte identity outside detected spans is preserved — masking splices
///   `[REDACTED:<kind>]` over span bytes and nothing else;
/// - masking keeps the claudy `[REDACTED:<kind>]` format (documented in
///   README/ledger) rather than the kernel's `apply_redactions` `****`,
///   so previews stay kind-labelled.
///
/// Category policy: `Secret` follows the user's `on_secret` setting;
/// `KoreanPii` and `FileSystemPath` are warn-only by default.
// ponytail: per-category config knobs when a user actually needs PII
// redaction or path stripping — warn-only keeps coding sessions
// functional (models need real paths to edit files).
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

/// Map a kernel rule label to the claudy finding kind (ledger vocabulary).
fn kind_for(rule: &str) -> &'static str {
    match rule {
        "bearer_header" => "auth_header",
        "key_value_assignment" => "key_value",
        "anthropic_key" => "anthropic_key",
        "openai_style_key" => "api_key",
        "stripe_secret_key" => "stripe_key",
        "figma_token" => "figma_token",
        "aws_access_key_id" | "aws_secret_key" => "aws_key",
        "github_token" => "github_token",
        "slack_token" => "slack_token",
        "private_key_header" => "private_key",
        "db_connection_string" => "db_connection",
        "rrn_kr" => "rrn",
        "bank_account_kr" => "bank_account",
        "phone_kr" => "phone",
        "home_path_posix" | "home_path_windows" | "tilde_path" => "local_path",
        _ => "secret",
    }
}

/// Kinds that should trigger the untrusted-provider re-route advisory.
/// Routine observations (image placeholders, boundary markers, local paths)
/// must not nag on every request.
pub fn is_advisory_sensitive(kind: &str) -> bool {
    !matches!(
        kind,
        "non_json" | "unparseable_json" | "local_path" | "image"
    )
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
        let text = if images_stripped == 0 {
            match std::str::from_utf8(body) {
                Ok(s) => s.to_string(),
                Err(_) => return fail_open("non_json"),
            }
        } else {
            serde_json::to_string(&root).unwrap_or_default()
        };

        let kreport = dlp::scan(&text);

        // Dedup spans (sk-ant keys double-fire anthropic_key +
        // openai_style_key; anthropic_key wins the kind label), resolve the
        // action per category, and collect redaction work.
        let mut findings: Vec<Finding> = Vec::new();
        // (span, kind, action) — one entry per unique span.
        let mut spans: Vec<(dlp::Span, String, GuardAction)> = Vec::new();
        let secret_action = self.secret_action();
        for kf in &kreport.findings {
            let kind = kind_for(&kf.rule);
            let action = match kf.category {
                FindingCategory::Secret => secret_action,
                FindingCategory::KoreanPii | FindingCategory::FileSystemPath => GuardAction::Warn,
                _ => GuardAction::Warn,
            };
            let preview = if matches!(kf.category, FindingCategory::Secret) {
                redact_credential(&text[kf.span.start..kf.span.end])
            } else {
                let raw = &text[kf.span.start..kf.span.end];
                raw.chars().take(48).collect()
            };
            findings.push(Finding {
                kind: kind.to_string(),
                action,
                preview,
            });
            match spans.iter_mut().find(|(s, _, _)| *s == kf.span) {
                Some((_, k, _)) => {
                    if kind == "anthropic_key" {
                        *k = kind.to_string();
                    }
                }
                None => spans.push((kf.span, kind.to_string(), action)),
            }
        }

        let mut redacted = text;
        let mut redacted_any = false;
        if spans
            .iter()
            .any(|(_, _, action)| *action == GuardAction::Redact)
        {
            // Replace descending by start so earlier byte offsets stay valid.
            let mut work: Vec<_> = spans
                .iter()
                .filter(|(_, _, action)| *action == GuardAction::Redact)
                .collect();
            work.sort_by_key(|(span, _, _)| std::cmp::Reverse(span.start));
            for (span, kind, _) in work {
                redacted.replace_range(span.start..span.end, &format!("[REDACTED:{}]", kind));
                redacted_any = true;
            }
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
            Some(redacted.into_bytes())
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
    fn redacts_authorization_bearer_header() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "leak: Authorization: Bearer abcdefghijklmnop123456 said hi"}]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert!(report.redacted_body.is_some());
        let out = String::from_utf8(report.redacted_body.unwrap()).unwrap();
        assert!(out.contains("[REDACTED:auth_header]"), "got: {out}");
        assert!(!out.contains("abcdefghijklmnop123456"));
        assert_eq!(report.findings[0].kind, "auth_header");
        assert!(!report.findings[0].preview.contains("abcdefghijklmnop"));
    }

    #[test]
    fn redacts_sk_ant_key_with_anthropic_priority() {
        // sk-ant keys double-fire anthropic_key + openai_style_key; the
        // kind label must resolve to anthropic_key.
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "key sk-ant-api03-abcdef1234567890abcdef was rotated"}]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert!(report.findings.iter().any(|f| f.kind == "anthropic_key"));
        let out = String::from_utf8(report.redacted_body.unwrap()).unwrap();
        assert!(out.contains("[REDACTED:anthropic_key]"));
        assert!(!out.contains("abcdef1234567890abcdef"));
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
        // JSON `"api_key":"value"` with a short value — no match.
        let body = br#"{"messages":[{"role":"user","content":"see api_key settings"}],"metadata":{"api_key":"short"}}"#;
        let report = scanner(SecretPolicy::Redact).scan(body, "application/json");
        assert!(report.findings.is_empty());
        assert!(report.redacted_body.is_none());
    }

    #[test]
    fn local_path_recorded_but_not_redacted() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "edit /Users/tester/dev/proj/src/main.rs please"}]
        });
        let report =
            scanner(SecretPolicy::Redact).scan(body.to_string().as_bytes(), "application/json");
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.kind == "local_path" && f.action == GuardAction::Warn)
        );
        assert!(report.redacted_body.is_none(), "paths stay functional");
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
            "messages": [{"role": "user", "content": "leak: Authorization: Bearer abcdefghijklmnop123456"}]
        });
        let report =
            scanner(SecretPolicy::Block).scan(body.to_string().as_bytes(), "application/json");
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.action == GuardAction::Block)
        );
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
