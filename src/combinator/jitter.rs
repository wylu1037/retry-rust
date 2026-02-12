use std::time::Duration;
use rand::Rng;

/// Full Jitter：将延迟随机化到 [0, delay] 范围
///
/// 完全随机化，分散效果最好。AWS 官方推荐搭配指数退避使用。
pub struct FullJitter<I> {
    inner: I,
}

impl<I> FullJitter<I> {
    /// 包装一个退避迭代器，添加 Full Jitter
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I: Iterator<Item = Duration>> Iterator for FullJitter<I> {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        self.inner.next().map(|d| {
            if d.is_zero() {
                return d;
            }
            let millis = rand::thread_rng().gen_range(0..=d.as_millis() as u64);
            Duration::from_millis(millis)
        })
    }
}

/// Equal Jitter：将延迟随机化到 [delay/2, delay] 范围
///
/// 保底一半延迟，在需要最低等待保证时使用。
pub struct EqualJitter<I> {
    inner: I,
}

impl<I> EqualJitter<I> {
    /// 包装一个退避迭代器，添加 Equal Jitter
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I: Iterator<Item = Duration>> Iterator for EqualJitter<I> {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        self.inner.next().map(|d| {
            if d.is_zero() {
                return d;
            }
            let half = d / 2;
            let half_millis = half.as_millis() as u64;
            let jitter = rand::thread_rng().gen_range(0..=half_millis);
            half + Duration::from_millis(jitter)
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
    fn test_equal_jitter_none_passthrough() {
        let delays: Vec<Duration> = vec![];
        let mut jitter = EqualJitter::new(delays.into_iter());
        assert_eq!(jitter.next(), None);
    }
}
