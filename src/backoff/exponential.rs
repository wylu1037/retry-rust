use std::time::Duration;

/// 指数退避策略
///
/// 每次重试等待时间按倍数增长。序列（base=1s, factor=2）：1, 2, 4, 8, 16...
pub struct ExponentialBackoff {
    current: Duration,
    factor: u32,
}

impl ExponentialBackoff {
    /// 创建指数退避策略
    pub fn new(base: Duration, factor: u32) -> Self {
        Self {
            current: base,
            factor,
        }
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self {
            current: Duration::from_secs(1),
            factor: 2,
        }
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let delay = self.current;
        self.current *= self.factor;
        Some(delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_sequence() {
        let mut backoff = ExponentialBackoff::default();
        assert_eq!(backoff.next(), Some(Duration::from_secs(1)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(2)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(4)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(8)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(16)));
    }

    #[test]
    fn test_exponential_backoff_factor_3() {
        let mut backoff = ExponentialBackoff::new(Duration::from_millis(100), 3);
        assert_eq!(backoff.next(), Some(Duration::from_millis(100)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(300)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(900)));
    }
}
