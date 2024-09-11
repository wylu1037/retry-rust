use std::time::Duration;

/// define the retry strategy enum
pub enum RetryStrategy {
    /// fixed interval delay
    Fixed { interval: Duration, max_retries: u32 },
    Random { max_retries: u32 },
    Backoff { max_retries: u32 },
}