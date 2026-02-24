use std::time::Duration;

/// Cap delay to ensure no single delay exceeds the maximum.
pub struct MaxDelay<I> {
    inner: I,
    max: Duration,
}

impl<I> MaxDelay<I> {
    /// Wrap a backoff iterator, capping any delay that exceeds `max`.
    pub fn new(inner: I, max: Duration) -> Self {
        Self { inner, max }
    }
}

impl<I> Iterator for MaxDelay<I>
where
    I: Iterator<Item = Duration>,
{
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
