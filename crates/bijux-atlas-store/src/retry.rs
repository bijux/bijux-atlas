use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_backoff_ms: u64,
}

pub trait BackoffPolicy {
    fn delay_for_attempt(&self, attempt: usize) -> Duration;
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_backoff_ms: 120,
        }
    }
}

impl BackoffPolicy for RetryPolicy {
    fn delay_for_attempt(&self, attempt: usize) -> Duration {
        Duration::from_millis(self.base_backoff_ms.saturating_mul(attempt as u64))
    }
}
