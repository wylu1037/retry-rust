# Retry

A Rust retry library built around `Iterator<Item = Duration>` backoff strategies.

Each backoff strategy is just an iterator that yields the next delay. This keeps the API small, composable, and idiomatic: use standard iterator adapters like `.take()`, `.map()`, and `.chain()`, or use the provided retry-focused combinators.

## Features

- Iterator-based backoff strategies
  - `FixedInterval`
  - `LinearBackoff`
  - `ExponentialBackoff`
  - `FibonacciBackoff`
- Backoff combinators
  - `full_jitter()`
  - `equal_jitter()`
  - `max_delay()`
- Synchronous retry helpers
  - `retry`
  - `retry_if`
- Optional Tokio-based async retry helpers
  - `retry_async`
  - `retry_async_if`

## Installation

```toml
[dependencies]
retry-rust = "0.1"
```

Enable async retry support with the `tokio` feature:

```toml
[dependencies]
retry-rust = { version = "0.1", features = ["tokio"] }
```

## Quick Start

```rust
use retry_rust::backoff::{BackoffExt, ExponentialBackoff};
use retry_rust::retry;
use std::time::Duration;

let result: Result<&str, &str> = retry(
    ExponentialBackoff::new(Duration::from_millis(100), 2)
        .full_jitter()
        .max_delay(Duration::from_secs(5))
        .take(5),
    || {
        // Call a fallible operation here.
        Ok("done")
    },
);

assert_eq!(result, Ok("done"));
```

The operation is called immediately. A delay is consumed only after a failed attempt. Therefore, `.take(5)` means at most 5 retries, or 6 total attempts including the initial call.

## Backoff Strategies

### Fixed Interval

Always yields the same delay.

```rust
use retry_rust::backoff::FixedInterval;
use std::time::Duration;

let backoff = FixedInterval::new(Duration::from_secs(1));
// Sequence: 1s, 1s, 1s, 1s, ...
```

### Linear Backoff

Increases the delay by a fixed step each time.

```rust
use retry_rust::backoff::LinearBackoff;
use std::time::Duration;

let backoff = LinearBackoff::new(
    Duration::from_secs(1),
    Duration::from_secs(1),
);
// Sequence: 1s, 2s, 3s, 4s, ...
```

### Exponential Backoff

Multiplies the delay by a factor each time.

```rust
use retry_rust::backoff::ExponentialBackoff;
use std::time::Duration;

let backoff = ExponentialBackoff::new(Duration::from_secs(1), 2);
// Sequence: 1s, 2s, 4s, 8s, ...
```

### Fibonacci Backoff

Grows according to the Fibonacci sequence.

```rust
use retry_rust::backoff::FibonacciBackoff;
use std::time::Duration;

let backoff = FibonacciBackoff::new(Duration::from_secs(1));
// Sequence: 1s, 1s, 2s, 3s, 5s, ...
```

## Combinators

Import `BackoffExt` to use the convenience methods on any `Iterator<Item = Duration>`.

```rust
use retry_rust::backoff::{BackoffExt, ExponentialBackoff};
use std::time::Duration;

let backoff = ExponentialBackoff::default()
    .full_jitter()
    .max_delay(Duration::from_secs(30))
    .take(5);
```

### Limit Retry Count

Use the standard iterator method `.take(n)`.

```rust
backoff.take(5); // At most 5 retries.
```

### Cap Maximum Delay

Use `.max_delay(max)` to ensure no single delay exceeds `max`.

```rust
use retry_rust::backoff::BackoffExt;
use std::time::Duration;

backoff.max_delay(Duration::from_secs(30));
```

### Full Jitter

`full_jitter()` randomizes each delay into the range `[0, delay]`.

This is useful for spreading retries across clients and avoiding thundering-herd behavior. It is commonly used with exponential backoff.

```rust
use retry_rust::backoff::BackoffExt;

backoff.full_jitter();
```

### Equal Jitter

`equal_jitter()` randomizes each delay into the range `[delay / 2, delay]`.

Use this when you want jitter while preserving at least half of the original delay.

```rust
use retry_rust::backoff::BackoffExt;

backoff.equal_jitter();
```

## Conditional Retry

Use `retry_if` when only some errors should be retried.

```rust
use retry_rust::backoff::FixedInterval;
use retry_rust::retry_if;
use std::time::Duration;

let result: Result<&str, &str> = retry_if(
    FixedInterval::new(Duration::from_millis(50)).take(3),
    || Err("fatal"),
    |error| *error == "temporary",
);

assert_eq!(result, Err("fatal"));
```

If the condition returns `false`, the error is returned immediately and no more retries are attempted.

## Async Retry

Async retry support is available behind the `tokio` feature.

```rust
use retry_rust::backoff::{BackoffExt, ExponentialBackoff};
use retry_rust::retry_async;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let result: Result<&str, &str> = retry_async(
        ExponentialBackoff::new(Duration::from_millis(100), 2)
            .full_jitter()
            .max_delay(Duration::from_secs(5))
            .take(5),
        || async {
            // Call an async fallible operation here.
            Ok("done")
        },
    )
    .await;

    assert_eq!(result, Ok("done"));
}
```

For conditional async retry, use `retry_async_if`.

## Public API Overview

Top-level helpers:

- `retry`
- `retry_if`
- `retry_async` with the `tokio` feature
- `retry_async_if` with the `tokio` feature

Backoff strategies under `retry_rust::backoff`:

- `FixedInterval`
- `LinearBackoff`
- `ExponentialBackoff`
- `FibonacciBackoff`
- `BackoffExt`

Combinator types under `retry_rust::combinator`:

- `FullJitter`
- `EqualJitter`
- `MaxDelay`

## Design Notes

The core design choice is to represent retry delays as `Iterator<Item = Duration>`.

Compared with a dedicated retry-strategy trait or enum, this approach has a few advantages:

- It is easy to compose with standard iterator adapters.
- It is zero-cost and idiomatic Rust.
- Users can define custom strategies by implementing `Iterator<Item = Duration>`.
- Retry limits naturally use `.take(n)` instead of a separate option type.

An empty iterator means no retries: the operation is still attempted once, and the first error is returned immediately.

## Development

Run the test suite:

```sh
cargo test
```

Run tests with async support enabled:

```sh
cargo test --all-features
```

## References

- [Exponential Backoff and Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
- [backon](https://github.com/Xuanwo/backon) — a Rust retry library that also uses iterator-based backoff strategies
- [retry-go](https://github.com/avast/retry-go) — a Go retry library
