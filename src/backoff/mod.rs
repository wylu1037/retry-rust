mod exponential;
mod fibonacci;
mod fixed;
mod linear;

pub use exponential::ExponentialBackoff;
pub use fibonacci::FibonacciBackoff;
pub use fixed::FixedInterval;
pub use linear::LinearBackoff;

use crate::combinator::{FullJitter, MaxDelay};
use std::time::Duration;

/// 退避策略扩展 trait
///
/// 为所有 `Iterator<Item = Duration>` 提供组合器便捷方法。
pub trait BackoffExt: Iterator<Item = Duration> + Sized {
    /// 添加 Full Jitter：将延迟随机化到 [0, delay] 范围
    fn full_jitter(self) -> FullJitter<Self> {
        FullJitter::new(self)
    }

    /// 截断延迟，单次等待不超过 max
    fn max_delay(self, max: Duration) -> MaxDelay<Self> {
        MaxDelay::new(self, max)
    }
}

impl<I: Iterator<Item = Duration>> BackoffExt for I {}
