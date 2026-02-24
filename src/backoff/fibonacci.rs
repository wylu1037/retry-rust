use std::time::Duration;

/// Fibonacci backoff strategy.
///
/// Delay grows according to the Fibonacci sequence. Sequence (unit=1s): 1, 1, 2, 3, 5, 8, 13...
pub struct FibonacciBackoff {
    a: Duration,
    b: Duration,
}

impl FibonacciBackoff {
    /// Create a new Fibonacci backoff strategy.
    pub fn new(unit: Duration) -> Self {
        Self { a: unit, b: unit }
    }
}

impl Default for FibonacciBackoff {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

impl Iterator for FibonacciBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let delay = self.a;
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_backoff_sequence() {
        let mut backoff = FibonacciBackoff::default();
        assert_eq!(backoff.next(), Some(Duration::from_secs(1)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(1)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(2)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(3)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(5)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(8)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(13)));
    }

    #[test]
    fn test_fibonacci_backoff_custom_unit() {
        let mut backoff = FibonacciBackoff::new(Duration::from_millis(100));
        assert_eq!(backoff.next(), Some(Duration::from_millis(100)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(100)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(200)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(300)));
    }
}
