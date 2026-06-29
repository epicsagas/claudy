use std::time::Duration;

use crate::domain::channel_events::{OutboundMessage, Platform};
use crate::ports::channel_ports::ChannelPort;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}

impl RetryPolicy {
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::Telegram => Self {
                max_attempts: 3,
                base_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(30),
                jitter: true,
            },
            Platform::Slack => Self {
                max_attempts: 3,
                base_delay: Duration::from_secs(2),
                max_delay: Duration::from_secs(60),
                jitter: true,
            },
            Platform::Discord => Self {
                max_attempts: 3,
                base_delay: Duration::from_secs(5),
                max_delay: Duration::from_secs(120),
                jitter: true,
            },
        }
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_ms = self.base_delay.as_millis() as u64;
        let raw_delay = base_ms.saturating_mul(2u64.saturating_pow(attempt));
        let capped = raw_delay.min(self.max_delay.as_millis() as u64);

        if self.jitter {
            // Full Jitter (AWS pattern): uniform random in [0, capped].
            // Guards against an empty range (capped == 0) to avoid panicking.
            if capped == 0 {
                return Duration::from_millis(0);
            }
            Duration::from_millis(rand::random_range(0..=capped))
        } else {
            Duration::from_millis(capped)
        }
    }
}

pub async fn retry_send(
    channel: &dyn ChannelPort,
    msg: &OutboundMessage,
    policy: &RetryPolicy,
) -> anyhow::Result<crate::domain::channel_events::MessageDelivery> {
    let mut attempt = 0;
    loop {
        match channel.send_message(msg).await {
            Ok(delivery) => return Ok(delivery),
            Err(e) if attempt + 1 < policy.max_attempts => {
                let delay = policy.delay_for_attempt(attempt);
                tracing::warn!(attempt, error = %e, "Retrying send after delay");
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

pub async fn retry_edit(
    channel: &dyn ChannelPort,
    msg: &OutboundMessage,
    policy: &RetryPolicy,
) -> anyhow::Result<()> {
    let mut attempt = 0;
    loop {
        match channel.edit_message(msg).await {
            Ok(()) => return Ok(()),
            Err(e) if attempt + 1 < policy.max_attempts => {
                let delay = policy.delay_for_attempt(attempt);
                tracing::warn!(attempt, error = %e, "Retrying edit after delay");
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    use crate::domain::channel_events::{ChannelIdentity, ConversationId, MessageDelivery};

    /// A mock ChannelPort that tracks call counts and can be configured to
    /// fail a configurable number of times before succeeding.
    struct MockChannel {
        send_fail_count: Arc<Mutex<usize>>,
        edit_fail_count: Arc<Mutex<usize>>,
        send_call_count: Arc<Mutex<usize>>,
        edit_call_count: Arc<Mutex<usize>>,
    }

    impl MockChannel {
        fn new() -> Self {
            Self {
                send_fail_count: Arc::new(Mutex::new(0)),
                edit_fail_count: Arc::new(Mutex::new(0)),
                send_call_count: Arc::new(Mutex::new(0)),
                edit_call_count: Arc::new(Mutex::new(0)),
            }
        }

        fn with_send_failures(self, n: usize) -> Self {
            *self.send_fail_count.lock().unwrap() = n;
            self
        }

        fn with_edit_failures(self, n: usize) -> Self {
            *self.edit_fail_count.lock().unwrap() = n;
            self
        }
    }

    #[async_trait]
    impl ChannelPort for MockChannel {
        async fn send_message(&self, _msg: &OutboundMessage) -> anyhow::Result<MessageDelivery> {
            let mut calls = self.send_call_count.lock().unwrap();
            *calls += 1;
            let mut fails = self.send_fail_count.lock().unwrap();
            if *fails > 0 {
                *fails -= 1;
                Err(anyhow::anyhow!("transient send error"))
            } else {
                Ok(MessageDelivery {
                    platform_message_id: "msg_123".to_string(),
                })
            }
        }

        async fn edit_message(&self, _msg: &OutboundMessage) -> anyhow::Result<()> {
            let mut calls = self.edit_call_count.lock().unwrap();
            *calls += 1;
            let mut fails = self.edit_fail_count.lock().unwrap();
            if *fails > 0 {
                *fails -= 1;
                Err(anyhow::anyhow!("transient edit error"))
            } else {
                Ok(())
            }
        }

        async fn delete_message(
            &self,
            _channel: &ChannelIdentity,
            _message_ref: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn ack_interaction(
            &self,
            _channel: &ChannelIdentity,
            _interaction_id: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn send_typing(&self, _channel: &ChannelIdentity) -> anyhow::Result<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    fn test_msg() -> OutboundMessage {
        OutboundMessage {
            conversation_id: ConversationId::new(),
            channel: ChannelIdentity::new(Platform::Telegram, "ch1", "user1", None, None),
            text: "hello".to_string(),
            message_ref: None,
            interaction: None,
        }
    }

    fn instant_policy() -> RetryPolicy {
        RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(1),
            jitter: false,
        }
    }

    #[tokio::test]
    async fn retry_edit_succeeds_on_first_try() {
        let mock = MockChannel::new();
        let edit_calls = mock.edit_call_count.clone();
        let policy = instant_policy();
        let msg = test_msg();

        let result = retry_edit(&mock, &msg, &policy).await;
        assert!(result.is_ok());
        assert_eq!(*edit_calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn retry_edit_retries_and_succeeds() {
        let mock = MockChannel::new().with_edit_failures(2);
        let edit_calls = mock.edit_call_count.clone();
        let policy = instant_policy();
        let msg = test_msg();

        let result = retry_edit(&mock, &msg, &policy).await;
        assert!(result.is_ok());
        assert_eq!(*edit_calls.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn retry_edit_exhausts_attempts_and_fails() {
        let mock = MockChannel::new().with_edit_failures(10);
        let edit_calls = mock.edit_call_count.clone();
        let policy = instant_policy();
        let msg = test_msg();

        let result = retry_edit(&mock, &msg, &policy).await;
        assert!(result.is_err());
        assert_eq!(*edit_calls.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn retry_send_succeeds_on_first_try() {
        let mock = MockChannel::new();
        let send_calls = mock.send_call_count.clone();
        let policy = instant_policy();
        let msg = test_msg();

        let result = retry_send(&mock, &msg, &policy).await;
        assert!(result.is_ok());
        assert_eq!(*send_calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn retry_send_retries_and_succeeds() {
        let mock = MockChannel::new().with_send_failures(2);
        let send_calls = mock.send_call_count.clone();
        let policy = instant_policy();
        let msg = test_msg();

        let result = retry_send(&mock, &msg, &policy).await;
        assert!(result.is_ok());
        assert_eq!(*send_calls.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn retry_send_exhausts_attempts_and_fails() {
        let mock = MockChannel::new().with_send_failures(10);
        let send_calls = mock.send_call_count.clone();
        let policy = instant_policy();
        let msg = test_msg();

        let result = retry_send(&mock, &msg, &policy).await;
        assert!(result.is_err());
        assert_eq!(*send_calls.lock().unwrap(), 3);
    }

    #[test]
    fn delay_for_attempt_without_jitter_is_exponential() {
        let policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            jitter: false,
        };
        assert_eq!(policy.delay_for_attempt(0), Duration::from_secs(1)); // 1 * 2^0
        assert_eq!(policy.delay_for_attempt(1), Duration::from_secs(2)); // 1 * 2^1
        assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(4)); // 1 * 2^2
        assert_eq!(policy.delay_for_attempt(3), Duration::from_secs(8));
        assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(16));
    }

    #[test]
    fn delay_for_attempt_caps_at_max() {
        let policy = RetryPolicy {
            max_attempts: 10,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            jitter: false,
        };
        assert_eq!(policy.delay_for_attempt(10), Duration::from_secs(30));
    }

    #[test]
    fn delay_for_attempt_with_jitter_stays_in_range() {
        // Full Jitter: result must be in [0, base * 2^attempt] capped at max.
        let policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            jitter: true,
        };
        for attempt in 0..5u32 {
            let upper = (1000u128 * 2u128.pow(attempt)).min(30_000);
            for _ in 0..20 {
                let got = policy.delay_for_attempt(attempt).as_millis();
                assert!(
                    got <= upper,
                    "attempt {attempt}: jittered delay {got}ms exceeds cap {upper}ms"
                );
            }
        }
    }

    #[test]
    fn delay_for_attempt_with_jitter_eventually_varies() {
        // Statistical sanity check: full jitter over many samples must produce
        // more than one distinct value (otherwise jitter is effectively off).
        let policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            jitter: true,
        };
        let mut distinct = std::collections::HashSet::new();
        for _ in 0..20 {
            distinct.insert(policy.delay_for_attempt(2).as_millis());
        }
        assert!(
            distinct.len() >= 2,
            "Jitter did not produce variation over 20 samples"
        );
    }
}
