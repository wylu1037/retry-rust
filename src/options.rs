use std::time::Duration;

pub struct Config {
    attempts: usize,
    delay: Duration,
    max_delay: Duration,
    max_jitter: Duration,
}
