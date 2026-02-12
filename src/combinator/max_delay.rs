use std::time::Duration;

/// 截断延迟，确保单次延迟不超过最大值
pub struct MaxDelay<I> {
    inner: I,
    max: Duration,
}

impl<I> MaxDelay<I> {
    /// 包装一个退避迭代器，截断超过 max 的延迟
    pub fn new(inner: I, max: Duration) -> Self {
        Self { inner, max }
    }
}

impl<I: Iterator<Item = Duration>> Iterator for MaxDelay<I> {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        self.inner.next().map(|d| d.min(self.max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_below_max_unchanged() {
        let delays = vec![Duration::from_secs(1), Duration::from_secs(2)];
        let mut capped = MaxDelay::new(delays.into_iter(), Duration::from_secs(10));
        assert_eq!(capped.next(), Some(Duration::from_secs(1)));
        assert_eq!(capped.next(), Some(Duration::from_secs(2)));
    }

    #[test]
    fn test_above_max_capped() {
        let delays = vec![Duration::from_secs(100)];
        let mut capped = MaxDelay::new(delays.into_iter(), Duration::from_secs(30));
        assert_eq!(capped.next(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_none_passthrough() {
        let delays: Vec<Duration> = vec![];
        let mut capped = MaxDelay::new(delays.into_iter(), Duration::from_secs(10));
        assert_eq!(capped.next(), None);
    }
}
