use retry_rust::backoff::{BackoffExt, ExponentialBackoff, FixedInterval};
use retry_rust::combinator::{FullJitter, MaxDelay};
use retry_rust::{retry, retry_if};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[test]
fn public_backoff_combinators_are_usable() {
    let mut backoff = ExponentialBackoff::new(Duration::from_millis(10), 2)
        .full_jitter()
        .max_delay(Duration::from_millis(15))
        .take(4);

    for delay in &mut backoff {
        assert!(delay <= Duration::from_millis(15));
    }
}

#[test]
fn public_combinator_types_are_constructible() {
    let mut jitter = FullJitter::new(vec![Duration::from_millis(5)].into_iter());
    assert!(jitter.next().unwrap() <= Duration::from_millis(5));

    let mut capped = MaxDelay::new(
        vec![Duration::from_millis(10)].into_iter(),
        Duration::from_millis(3),
    );
    assert_eq!(capped.next(), Some(Duration::from_millis(3)));
}

#[test]
fn public_retry_retries_until_success() {
    let count = AtomicU32::new(0);
    let result: Result<&str, &str> = retry(FixedInterval::new(Duration::ZERO).take(3), || {
        let n = count.fetch_add(1, Ordering::SeqCst);
        if n < 2 {
            Err("temporary")
        } else {
            Ok("done")
        }
    });

    assert_eq!(result, Ok("done"));
    assert_eq!(count.load(Ordering::SeqCst), 3);
}

#[test]
fn public_retry_if_stops_on_non_retryable_error() {
    let count = AtomicU32::new(0);
    let result: Result<&str, &str> = retry_if(
        FixedInterval::new(Duration::ZERO).take(3),
        || {
            count.fetch_add(1, Ordering::SeqCst);
            Err("fatal")
        },
        |error| *error != "fatal",
    );

    assert_eq!(result, Err("fatal"));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
