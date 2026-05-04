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
        let base = self.base_delay.as_millis() as u64 * 2u64.pow(attempt);
        let capped = base.min(self.max_delay.as_millis() as u64);
        Duration::from_millis(capped)
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
