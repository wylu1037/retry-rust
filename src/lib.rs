//! # Retry
//!
//! 基于 Iterator 抽象的 Rust 重试库。
//!
//! 每个退避策略实现 `Iterator<Item = Duration>`，通过组合器自由叠加。
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use std::time::Duration;
//! use retry_rust::backoff::ExponentialBackoff;
//! use retry_rust::backoff::BackoffExt;
//! use retry_rust::retry;
//!
//! let result: Result<i32, &str> = retry(
//!     ExponentialBackoff::default().max_delay(Duration::from_secs(30)).take(5),
//!     || {
//!         Ok(42)
//!     },
//! );
//! ```

pub mod backoff;
pub mod combinator;
pub mod retry;

pub use retry::retry;
pub use retry::retry_if;

#[cfg(feature = "tokio")]
pub use retry::retry_async;
