use rand::Rng;
use std::time::Duration;

fn random_duration_inclusive(max: Duration) -> Duration {
    let nanos = rand::thread_rng().gen_range(0..=max.as_nanos());
    duration_from_nanos(nanos)
}

fn duration_from_nanos(nanos: u128) -> Duration {
    Duration::new(
        (nanos / 1_000_000_000) as u64,
        (nanos % 1_000_000_000) as u32,
    )
}

fn duration_div_2(duration: Duration) -> Duration {
    duration_from_nanos(duration.as_nanos() / 2)
}

/// Full Jitter: randomizes delay to [0, delay] range.
///
/// Best for distributing retries. AWS recommends using with exponential backoff.
pub struct FullJitter<I> {
    inner: I,
}

impl<I> FullJitter<I> {
    /// Wrap a backoff iterator with Full Jitter.
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I> Iterator for FullJitter<I>
where
    I: Iterator<Item = Duration>,
{
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        self.inner.next().map(random_duration_inclusive)
    }
}

/// Equal Jitter: randomizes delay to [delay/2, delay] range.
///
/// Guarantees at least half the delay, for minimum wait requirements.
pub struct EqualJitter<I> {
    inner: I,
}

impl<I> EqualJitter<I> {
    /// Wrap a backoff iterator with Equal Jitter.
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I> Iterator for EqualJitter<I>
where
    I: Iterator<Item = Duration>,
{
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        self.inner.next().map(|d| {
            let half = duration_div_2(d);
            half + random_duration_inclusive(half)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_jitter_in_range() {
        let delays = vec![Duration::from_secs(10)];
        let mut jitter = FullJitter::new(delays.into_iter());
        let result = jitter.next().unwrap();
        assert!(result <= Duration::from_secs(10));
    }

    #[test]
    fn test_full_jitter_zero() {
        let delays = vec![Duration::ZERO];
        let mut jitter = FullJitter::new(delays.into_iter());
        assert_eq!(jitter.next(), Some(Duration::ZERO));
    }

    #[test]
    fn test_full_jitter_preserves_sub_millisecond_range() {
        let delays = vec![Duration::from_nanos(1)];
        let mut jitter = FullJitter::new(delays.into_iter());
        let result = jitter.next().unwrap();
        assert!(result <= Duration::from_nanos(1));
    }

    #[test]
    fn test_full_jitter_none_passthrough() {
        let delays: Vec<Duration> = vec![];
        let mut jitter = FullJitter::new(delays.into_iter());
        assert_eq!(jitter.next(), None);
    }

    #[test]
    fn test_equal_jitter_in_range() {
        let delays = vec![Duration::from_secs(10)];
        let mut jitter = EqualJitter::new(delays.into_iter());
        let result = jitter.next().unwrap();
        assert!(result >= Duration::from_secs(5));
        assert!(result <= Duration::from_secs(10));
    }

    #[test]
    fn test_equal_jitter_zero() {
        let delays = vec![Duration::ZERO];
        let mut jitter = EqualJitter::new(delays.into_iter());
        assert_eq!(jitter.next(), Some(Duration::ZERO));
    }

    #[test]
    fn test_equal_jitter_preserves_sub_millisecond_range() {
        let delays = vec![Duration::from_nanos(3)];
        let mut jitter = EqualJitter::new(delays.into_iter());
        let result = jitter.next().unwrap();
        assert!(result >= Duration::from_nanos(1));
        assert!(result <= Duration::from_nanos(3));
    }

    #[test]
    fn test_equal_jitter_none_passthrough() {
        let delays: Vec<Duration> = vec![];
        let mut jitter = EqualJitter::new(delays.into_iter());
        assert_eq!(jitter.next(), None);
    }
}
