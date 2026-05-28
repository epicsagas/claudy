use std::time::Instant;

use tokio::io::{AsyncBufReadExt, BufReader};

use crate::adapters::channel::retry::{RetryPolicy, retry_edit};
use crate::domain::channel_events::{ChannelIdentity, OutboundMessage};
use crate::ports::channel_ports::ChannelPort;

/// Indicates the stream ended due to a context window limit error from the
/// Claude API. The caller should trigger compaction or start a new session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLimitError {
    /// The raw error message from Claude.
    pub message: String,
}

impl std::fmt::Display for ContextLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context window limit: {}", self.message)
    }
}

impl std::error::Error for ContextLimitError {}

/// Checks whether a stream line or accumulated text indicates a context
/// window limit error from the Claude API.
pub fn is_context_limit_error(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("context window limit")
        || lower.contains("context length exceeded")
        || lower.contains("max_context_tokens")
        || lower.contains("prompt is too long")
}

pub struct StreamResult {
    /// Session ID extracted from the first stream event that contains one.
    pub session_id: Option<String>,
    /// Working directory captured from stream events.
    pub cwd: Option<String>,
    /// Whether any response content was received.
    pub has_content: bool,
    /// The full accumulated response text (for post-stream analysis).
    pub accumulated_text: String,
    /// Git branch name from stream events.
    pub branch: Option<String>,
    /// Model name from the last assistant event.
    pub model: Option<String>,
    /// Accumulated input tokens across all assistant events in this stream.
    pub input_tokens: i64,
    /// Accumulated output tokens across all assistant events in this stream.
    pub output_tokens: i64,
    /// Set when the stream detected a context window limit error.
    pub context_limit: Option<ContextLimitError>,
}

async fn do_edit(
    channel: &dyn ChannelPort,
    channel_identity: &ChannelIdentity,
    initial_message_id: &str,
    text: String,
) {
    let edit_msg = OutboundMessage {
        conversation_id: crate::domain::channel_events::ConversationId::new(),
        channel: channel_identity.clone(),
        text,
        message_ref: Some(initial_message_id.to_string()),
        interaction: None,
    };
    // Streaming edits are high-frequency — use a fast retry policy to avoid
    // blocking the stream pipeline on platform API issues.
    let fast_policy = RetryPolicy {
        max_attempts: 2,
        base_delay: std::time::Duration::from_millis(500),
        max_delay: std::time::Duration::from_secs(1),
        jitter: false,
    };
    if let Err(e) = retry_edit(channel, &edit_msg, &fast_policy).await {
        let err_str = e.to_string();
        // Silently ignore "not modified" — content is already up to date
        if err_str.contains("not modified") {
            return;
        }
        tracing::warn!(error = %e, "Failed to edit message, sending as new message");
        let max_len = channel_identity.platform.max_message_length();
        let fallback_text = truncate_message(&edit_msg.text, max_len);
        let fallback_msg = OutboundMessage {
            conversation_id: crate::domain::channel_events::ConversationId::new(),
            channel: channel_identity.clone(),
            text: fallback_text,
            message_ref: None,
            interaction: None,
        };
        if let Err(e2) = channel.send_message(&fallback_msg).await {
            tracing::error!(error = %e2, "Failed to send fallback message");
            let err_msg = OutboundMessage {
                conversation_id: crate::domain::channel_events::ConversationId::new(),
                channel: channel_identity.clone(),
                text: "Response too long to display. The full response is available in your Claude session.".to_string(),
                message_ref: None,
                interaction: None,
            };
            let _ = channel.send_message(&err_msg).await;
        }
    }
}

/// Seconds to wait for the very first output from Claude before giving up.
const FIRST_BYTE_TIMEOUT_SECS: u64 = 120;

pub async fn stream_response(
    stdout: &mut tokio::process::ChildStdout,
    channel: &dyn ChannelPort,
    channel_identity: &ChannelIdentity,
    initial_message_id: &str,
    idle_timeout_secs: u64,
) -> anyhow::Result<StreamResult> {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let mut accumulated = String::new();
    let mut last_edit_len: usize = 0;
    let mut session_id: Option<String> = None;
    let mut cwd: Option<String> = None;
    let mut first_output_received = false;
    let mut last_activity = Instant::now();
    let mut branch: Option<String> = None;
    let mut model: Option<String> = None;
    let mut input_tokens: i64 = 0;
    let mut output_tokens: i64 = 0;
    let mut context_limit: Option<ContextLimitError> = None;

    loop {
        let line_future = lines.next_line();
        tokio::pin!(line_future);

        let effective_timeout = if first_output_received {
            idle_timeout_secs
        } else {
            FIRST_BYTE_TIMEOUT_SECS
        };

        let elapsed = last_activity.elapsed().as_secs();
        let remaining = effective_timeout.saturating_sub(elapsed);
        if remaining == 0 {
            if !first_output_received {
                tracing::error!(
                    "Claude produced no output after {}s — first-byte timeout",
                    elapsed,
                );
                return Err(anyhow::anyhow!(
                    "Claude did not start responding within {} seconds",
                    FIRST_BYTE_TIMEOUT_SECS,
                ));
            }
            tracing::error!(
                idle_secs = elapsed,
                "Claude stream idle timeout — no output for {}s",
                elapsed,
            );
            return Err(anyhow::anyhow!(
                "No response from Claude for {} seconds (idle timeout)",
                elapsed,
            ));
        }

        match tokio::time::timeout(std::time::Duration::from_secs(remaining), line_future).await {
            Ok(Ok(Some(line))) => {
                last_activity = Instant::now();
                first_output_received = true;
                process_line(
                    &line,
                    channel,
                    channel_identity,
                    initial_message_id,
                    &mut StreamState {
                        accumulated: &mut accumulated,
                        last_edit_len: &mut last_edit_len,
                        session_id: &mut session_id,
                        cwd: &mut cwd,
                        branch: &mut branch,
                        model: &mut model,
                        input_tokens: &mut input_tokens,
                        output_tokens: &mut output_tokens,
                        context_limit: &mut context_limit,
                    },
                )
                .await;
            }
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {
                tracing::error!(idle_timeout_secs, "Idle timeout waiting for Claude output",);
                return Err(anyhow::anyhow!(
                    "No response from Claude for {} seconds (idle timeout)",
                    idle_timeout_secs,
                ));
            }
        }
    }

    if !accumulated.is_empty() && accumulated.len() > last_edit_len {
        let max_len = channel_identity.platform.max_message_length();
        let text = truncate_message(&accumulated, max_len);
        do_edit(channel, channel_identity, initial_message_id, text).await;
    }

    Ok(StreamResult {
        session_id,
        cwd,
        has_content: !accumulated.is_empty(),
        accumulated_text: accumulated,
        branch,
        model,
        input_tokens,
        output_tokens,
        context_limit,
    })
}

struct StreamState<'a> {
    accumulated: &'a mut String,
    last_edit_len: &'a mut usize,
    session_id: &'a mut Option<String>,
    cwd: &'a mut Option<String>,
    branch: &'a mut Option<String>,
    model: &'a mut Option<String>,
    input_tokens: &'a mut i64,
    output_tokens: &'a mut i64,
    context_limit: &'a mut Option<ContextLimitError>,
}

async fn process_line(
    line: &str,
    channel: &dyn ChannelPort,
    channel_identity: &ChannelIdentity,
    initial_message_id: &str,
    state: &mut StreamState<'_>,
) {
    let event: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return,
    };

    if let Some(sid) = event["session_id"].as_str() {
        *state.session_id = Some(sid.to_string());
    }
    if let Some(c) = event["cwd"].as_str() {
        *state.cwd = Some(c.to_string());
    }
    if let Some(b) = event["gitBranch"].as_str() {
        *state.branch = Some(b.to_string());
    }

    let event_type = event["type"].as_str().unwrap_or("");

    match event_type {
        "assistant" => {
            if let Some(m) = event["message"]["model"].as_str() {
                *state.model = Some(m.to_string());
            }
            if let Some(usage) = event["message"]["usage"].as_object() {
                *state.input_tokens += usage
                    .get("input_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                *state.output_tokens += usage
                    .get("output_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
            }
            if let Some(content) = event["message"]["content"].as_str() {
                state.accumulated.push_str(content);
            } else if let Some(arr) = event["message"]["content"].as_array() {
                for block in arr {
                    if block["type"].as_str() == Some("text")
                        && let Some(text) = block["text"].as_str()
                    {
                        state.accumulated.push_str(text);
                    }
                }
            }
        }
        "result" => {
            // Result contains the complete final response — replace accumulated
            // text instead of appending, to avoid duplicating assistant content.
            let result_text = if let Some(text) = event["result"].as_str() {
                Some(text.to_string())
            } else if let Some(arr) = event["result"].as_array() {
                let mut s = String::new();
                for block in arr {
                    if block["type"].as_str() == Some("text")
                        && let Some(text) = block["text"].as_str()
                    {
                        s.push_str(text);
                    }
                }
                if s.is_empty() { None } else { Some(s) }
            } else {
                None
            };
            if let Some(ref text) = result_text
                && is_context_limit_error(text)
            {
                *state.context_limit = Some(ContextLimitError {
                    message: text.clone(),
                });
            }
            // Also check the error field for API errors
            if let Some(err) = event["error"].as_str()
                && is_context_limit_error(err)
            {
                *state.context_limit = Some(ContextLimitError {
                    message: err.to_string(),
                });
            }
            if let Some(text) = result_text {
                *state.accumulated = text;
            }
        }
        "tool_use" => {}
        "control_request" => {}
        _ => {}
    }

    // Only edit when meaningful new content has accumulated (≥50 new chars)
    // to reduce Telegram API calls and rate limiting
    if state.accumulated.len() > *state.last_edit_len + 50 {
        let max_len = channel_identity.platform.max_message_length();
        let text = truncate_message(state.accumulated, max_len);
        do_edit(channel, channel_identity, initial_message_id, text).await;
        *state.last_edit_len = state.accumulated.len();
    }
}

/// Truncate a message to fit within a platform's character limit.
pub fn truncate_message(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let suffix = "\n\n[... truncated]";
    let budget = max_chars - suffix.chars().count();
    let truncated: String = text.chars().take(budget).collect();
    format!("{}{}", truncated, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_context_limit_error_detects_known_patterns() {
        assert!(is_context_limit_error(
            "The model has reached its context window limit."
        ));
        assert!(is_context_limit_error(
            "context length exceeded: too many tokens"
        ));
        assert!(is_context_limit_error("max_context_tokens exceeded"));
        assert!(is_context_limit_error(
            "prompt is too long: 201234 tokens > 200000 maximum"
        ));
        assert!(is_context_limit_error("context window limit reached"));
    }

    #[test]
    fn is_context_limit_error_rejects_normal_messages() {
        assert!(!is_context_limit_error("Hello, how can I help?"));
        assert!(!is_context_limit_error("Step 3 완료. Step 4 진행합니다."));
        assert!(!is_context_limit_error("The build completed successfully."));
        assert!(!is_context_limit_error(
            "in the context of our discussion, there is a limit"
        ));
        assert!(!is_context_limit_error(
            "context and rate limit considerations"
        ));
        // Must not match non-error messages mentioning max_tokens
        assert!(!is_context_limit_error(
            "The max_tokens parameter exceeded the recommended value"
        ));
        assert!(!is_context_limit_error(
            "Set max_tokens to 4096. If you exceeded that, adjust your config."
        ));
    }

    #[test]
    fn is_context_limit_error_detects_actual_api_errors() {
        // Actual Anthropic API error format
        assert!(is_context_limit_error(
            "prompt is too long: 201234 tokens > 200000 maximum"
        ));
        assert!(is_context_limit_error(
            "Error: prompt is too long: 150000 tokens > 128000 maximum"
        ));
    }

    #[test]
    fn context_limit_error_display() {
        let err = ContextLimitError {
            message: "test error".to_string(),
        };
        assert_eq!(format!("{}", err), "Context window limit: test error");
    }

    #[test]
    fn stream_result_context_limit_default_none() {
        let result = StreamResult {
            session_id: None,
            cwd: None,
            has_content: false,
            accumulated_text: String::new(),
            branch: None,
            model: None,
            input_tokens: 0,
            output_tokens: 0,
            context_limit: None,
        };
        assert!(result.context_limit.is_none());
    }

    #[test]
    fn stream_result_context_limit_set() {
        let result = StreamResult {
            session_id: None,
            cwd: None,
            has_content: true,
            accumulated_text: "error text".to_string(),
            branch: None,
            model: None,
            input_tokens: 0,
            output_tokens: 0,
            context_limit: Some(ContextLimitError {
                message: "context window limit reached".to_string(),
            }),
        };
        assert!(result.context_limit.is_some());
        assert_eq!(
            result.context_limit.unwrap().message,
            "context window limit reached"
        );
    }
}
