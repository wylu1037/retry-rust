use std::time::Duration;

/// 同步重试
///
/// 使用退避策略不断重试操作直到成功或耗尽重试次数。
/// 第一次调用不等待，失败后按退避策略等待再重试。
pub fn retry<I, F, T, E>(backoff: I, mut f: F) -> Result<T, E>
where
    I: IntoIterator<Item = Duration>,
    F: FnMut() -> Result<T, E>,
{
    let mut iter = backoff.into_iter();
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => match iter.next() {
                Some(delay) => std::thread::sleep(delay),
                None => return Err(e),
            },
        }
    }
}

/// 条件重试
///
/// 只有当 `condition` 返回 `true` 时才重试，否则立即返回错误。
pub fn retry_if<I, F, T, E, C>(backoff: I, mut f: F, condition: C) -> Result<T, E>
where
    I: IntoIterator<Item = Duration>,
    F: FnMut() -> Result<T, E>,
    C: Fn(&E) -> bool,
{
    let mut iter = backoff.into_iter();
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                if !condition(&e) {
                    return Err(e);
                }
                match iter.next() {
                    Some(delay) => std::thread::sleep(delay),
                    None => return Err(e),
                }
            }
        }
    }
}

/// 异步重试
#[cfg(feature = "tokio")]
pub async fn retry_async<I, F, Fut, T, E>(backoff: I, mut f: F) -> Result<T, E>
where
    I: IntoIterator<Item = Duration>,
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut iter = backoff.into_iter();
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => match iter.next() {
                Some(delay) => tokio::time::sleep(delay).await,
                None => return Err(e),
            },
        }
    }
}

/// 异步条件重试
#[cfg(feature = "tokio")]
pub async fn retry_async_if<I, F, Fut, T, E, C>(backoff: I, mut f: F, condition: C) -> Result<T, E>
where
    I: IntoIterator<Item = Duration>,
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    C: Fn(&E) -> bool,
{
    let mut iter = backoff.into_iter();
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if !condition(&e) {
                    return Err(e);
                }
                match iter.next() {
                    Some(delay) => tokio::time::sleep(delay).await,
                    None => return Err(e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_retry_immediate_success() {
        let count = AtomicU32::new(0);
        let result: Result<i32, &str> = retry(
            vec![Duration::ZERO; 3],
            || {
                count.fetch_add(1, Ordering::SeqCst);
                Ok(42)
            },
        );
        assert_eq!(result, Ok(42));
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_retry_success_on_third_attempt() {
        let count = AtomicU32::new(0);
        let result: Result<i32, &str> = retry(
            vec![Duration::ZERO; 5],
            || {
                let n = count.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err("fail")
                } else {
                    Ok(42)
                }
            },
        );
        assert_eq!(result, Ok(42));
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_all_fail() {
        let count = AtomicU32::new(0);
        let result: Result<i32, &str> = retry(
            vec![Duration::ZERO; 3],
            || {
                count.fetch_add(1, Ordering::SeqCst);
                Err("fail")
            },
        );
        assert_eq!(result, Err("fail"));
        // 1 initial + 3 retries = 4 calls
        assert_eq!(count.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_retry_if_non_retryable() {
        let count = AtomicU32::new(0);
        let result: Result<i32, &str> = retry_if(
            vec![Duration::ZERO; 5],
            || {
                count.fetch_add(1, Ordering::SeqCst);
                Err("fatal")
            },
            |e| *e != "fatal",
        );
        assert_eq!(result, Err("fatal"));
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_retry_if_retryable() {
        let count = AtomicU32::new(0);
        let result: Result<i32, &str> = retry_if(
            vec![Duration::ZERO; 5],
            || {
                let n = count.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err("transient")
                } else {
                    Ok(42)
                }
            },
            |e| *e == "transient",
        );
        assert_eq!(result, Ok(42));
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }
}
