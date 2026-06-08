#![cfg(feature = "tokio")]

use retry_rust::{retry_async, retry_async_if};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[tokio::test]
async fn public_retry_async_retries_until_success() {
    let count = AtomicU32::new(0);
    let result: Result<&str, &str> = retry_async(vec![Duration::ZERO; 3], || async {
        let n = count.fetch_add(1, Ordering::SeqCst);
        if n < 2 {
            Err("temporary")
        } else {
            Ok("done")
        }
    })
    .await;

    assert_eq!(result, Ok("done"));
    assert_eq!(count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn public_retry_async_if_stops_on_non_retryable_error() {
    let count = AtomicU32::new(0);
    let result: Result<&str, &str> = retry_async_if(
        vec![Duration::ZERO; 3],
        || async {
            count.fetch_add(1, Ordering::SeqCst);
            Err("fatal")
        },
        |error| *error != "fatal",
    )
    .await;

    assert_eq!(result, Err("fatal"));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn public_retry_async_if_retries_until_success() {
    let count = AtomicU32::new(0);
    let result: Result<&str, &str> = retry_async_if(
        vec![Duration::ZERO; 3],
        || async {
            let n = count.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                Err("temporary")
            } else {
                Ok("done")
            }
        },
        |error| *error == "temporary",
    )
    .await;

    assert_eq!(result, Ok("done"));
    assert_eq!(count.load(Ordering::SeqCst), 3);
}
