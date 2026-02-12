use std::time::Duration;

/// 固定间隔退避策略
///
/// 每次重试等待相同的时间。序列：d, d, d, d, d...
pub struct FixedInterval {
    interval: Duration,
}

impl FixedInterval {
    /// 创建固定间隔策略
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }
}

impl Default for FixedInterval {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),
        }
    }
}

impl Iterator for FixedInterval {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        Some(self.interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_interval_sequence() {
        let mut backoff = FixedInterval::new(Duration::from_secs(2));
        for _ in 0..5 {
            assert_eq!(backoff.next(), Some(Duration::from_secs(2)));
        }
    }

    #[test]
    fn test_fixed_interval_default() {
        let mut backoff = FixedInterval::default();
        assert_eq!(backoff.next(), Some(Duration::from_secs(1)));
    }
}
