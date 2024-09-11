use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

// 示例 RetryStrategy 枚举
enum RetryStrategy {
    Fixed { interval: Duration, max_retries: u32 },
    Random { max_retries: u32 },
    Backoff { max_retries: u32 },
}

/// call retry function
pub fn act<F, T>(mut func: F, strategy: RetryStrategy) -> Result<T, Box<dyn Error>>
    where
        F: FnMut() -> Result<T, Box<dyn Error>>,
{
    let mut retries = 0;

    loop {
        match func() {
            Ok(result) => return Ok(result),
            Err(e) => {
                retries += 1;
                if retries > match strategy {
                    RetryStrategy::Fixed { max_retries, .. } => max_retries,
                    RetryStrategy::Random { max_retries, .. } => max_retries,
                    RetryStrategy::Backoff { max_retries, .. } => max_retries,
                } {
                    return Err(e);
                }

                let wait_duration = match strategy {
                    RetryStrategy::Fixed { interval, .. } => interval,
                    _ => Duration::ZERO,
                };

                if !wait_duration.is_zero() {
                    sleep(wait_duration);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::RetryError;
    use crate::retry;

    use super::*;

    #[test]
    fn test() {
        let result: Result<i32, Box<dyn Error>> = retry::act(
            || {
                println!("重试......");
                Err(Box::new(RetryError::custom(-1, "执行出错")))
            },
            RetryStrategy::Fixed {
                interval: Duration::from_secs(1),
                max_retries: 5,
            },
        );

        match result {
            Ok(_) => println!("操作成功"),
            Err(e) => println!("Operation failed after retries: {}", e),
        }
    }
}