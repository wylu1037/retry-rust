<h1 align="center">Retry</h1>

## 1.基础重试策略

## 1.1 固定间隔重试 (Fixed Interval)

```rust
fn fixed_interval(attempt: u32) -> Duration {
    Duration::from_secs(2) // 固定2秒
}
```

+ 优点：简单直观
+ 缺点：可能造成"惊群效应"
+ 适用场景：简单场景，负载较轻

## 1.2 线性退避 (Linear Backoff)

```rust
fn linear_backoff(attempt: u32) -> Duration {
    Duration::from_secs(2 * attempt as u64)
}
```

+ 特点：重试间隔线性增长
+ 适用场景：需要温和增长的场景

## 2. 高级重试策略

### 2.1 指数退避 (Exponential Backoff)

```rust
fn exponential_backoff(attempt: u32) -> Duration {
    Duration::from_secs(2u64.pow(attempt))
}
```

+ 特点：间隔时间快速增长
+ 常见变体：截断指数退避（有最大值限制）

### 2.2 斐波那契退避 (Fibonacci Backoff)

```rust
fn fibonacci_backoff(attempt: u32) -> Duration {
    let mut a = 1;
    let mut b = 1;
    for _ in 0..attempt {
        let temp = a + b;
        a = b;
        b = temp;
    }
    Duration::from_secs(a)
}
```

+ 特点：增长速度介于线性和指数之间

## 3.随机化策略

### 3.1 全随机抖动 (Full Jitter)

```rust
use rand::Rng;

fn full_jitter(attempt: u32) -> Duration {
    let max = 2u64.pow(attempt);
    let random_secs = rand::thread_rng().gen_range(0..=max);
    Duration::from_secs(random_secs)
}
```

### 3.2 等比抖动 (Equal Jitter)

```rust
fn equal_jitter(attempt: u32) -> Duration {
    let base = 2u64.pow(attempt);
    let random_part = rand::thread_rng().gen_range(0..=base);
    Duration::from_secs(base / 2 + random_part / 2)
}
```

### 3.3 装饰器抖动 (Decorrelated Jitter)

```rust
fn decorrelated_jitter(previous: Duration, attempt: u32) -> Duration {
    let random_factor = rand::thread_rng().gen_range(1..3);
    min(
        Duration::from_secs(30), // max_interval
        previous * random_factor
    )
}
```

## 4.高级特性

### 4.1 自适应重试

```rust
struct AdaptiveRetry {
    base_interval: Duration,
    success_count: u32,
    failure_count: u32,
}

impl AdaptiveRetry {
    fn next_interval(&mut self) -> Duration {
        let factor = self.failure_count as f64 / (self.success_count + 1) as f64;
        self.base_interval * factor.ceil() as u32
    }
}
```

### 4.2 错误感知重试

```rust
enum RetryDecision {
    Retry(Duration),
    Stop,
}

fn error_aware_retry(error: &Error, attempt: u32) -> RetryDecision {
    match error {
        Error::Timeout => RetryDecision::Retry(exponential_backoff(attempt)),
        Error::RateLimit => RetryDecision::Retry(Duration::from_secs(60)),
        Error::InvalidInput => RetryDecision::Stop,
    }
}
```

## 5.建议增加的功能

### 5.1 重试限制

```rust
struct RetryConfig {
    max_attempts: u32,
    max_duration: Duration,
    max_delay: Duration,
}
```

### 5.2 条件重试

```rust
type RetryPredicate = Box<dyn Fn(&Error) -> bool>;

struct RetryPolicy {
    should_retry: RetryPredicate,
    backoff_strategy: Box<dyn BackoffStrategy>,
}
```

### 5.3 重试事件监听

```rust
trait RetryListener {
    fn on_retry(&self, attempt: u32, error: &Error);
    fn on_success(&self, attempts: u32);
    fn on_failure(&self, final_error: &Error);
}
```

### 5.4 上下文感知重试

```rust
struct RetryContext {
    attempt: u32,
    elapsed_time: Duration,
    last_error: Option<Error>,
    metadata: HashMap<String, String>,
}
```

## 如何实现？

1.使用特征（Trait）定义重试策略接口

```rust
trait RetryStrategy {
    fn next_delay(&self, context: &RetryContext) -> Option<Duration>;
}
```

2.提供组合器模式

```rust
struct RetryBuilder {
    strategy: Box<dyn RetryStrategy>,
    max_attempts: Option<u32>,
    max_duration: Option<Duration>,
    listeners: Vec<Box<dyn RetryListener>>,
}
```

3.支持异步操作

```rust
async fn retry<F, Fut, T, E>(strategy: impl RetryStrategy, f: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output=Result<T, E>>,
{
    // 实现
}
```