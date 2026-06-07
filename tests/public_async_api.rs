#![cfg(feature = "tokio")]

use retry_rust::retry_async;
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
