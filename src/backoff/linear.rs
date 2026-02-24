use std::time::Duration;

/// Linear backoff strategy.
///
/// Delay grows linearly with each retry. Sequence (base=1s, step=1s): 1, 2, 3, 4, 5...
pub struct LinearBackoff {
    current: Duration,
    step: Duration,
}

impl LinearBackoff {
    /// Create a new linear backoff strategy.
    pub fn new(base: Duration, step: Duration) -> Self {
        Self {
            current: base,
            step,
        }
    }
}

impl Default for LinearBackoff {
    fn default() -> Self {
        Self {
            current: Duration::from_secs(1),
            step: Duration::from_secs(1),
        }
    }
}

impl Iterator for LinearBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let delay = self.current;
        self.current += self.step;
        Some(delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_backoff_sequence() {
        let mut backoff = LinearBackoff::default();
        assert_eq!(backoff.next(), Some(Duration::from_secs(1)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(2)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(3)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(4)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_linear_backoff_custom() {
        let mut backoff = LinearBackoff::new(Duration::from_millis(100), Duration::from_millis(200));
        assert_eq!(backoff.next(), Some(Duration::from_millis(100)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(300)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(500)));
    }
}
