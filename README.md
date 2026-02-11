<h1 align="center">Retry</h1>

> 一个 Rust 重试库的设计文档。参考：[retry-go](https://github.com/avast/retry-go)、[backon](https://github.com/Xuanwo/backon)

## 核心设计决策

### API 风格：Iterator 抽象

每个退避策略实现 `Iterator<Item = Duration>`，调用者每次 `.next()` 获取下一次等待时间。`None` 表示停止重试。

```rust
use std::time::Duration;

// 退避策略 = Duration 的迭代器
let backoff = ExponentialBackoff::default();
for delay in backoff.take(5) {
    println!("wait {:?}", delay);
}
```

**为什么选 Iterator 而不是 Trait/Enum：**

| 方案 | 优点 | 缺点 |
|------|------|------|
| `Iterator<Item = Duration>` | 天然支持组合（`.take()` `.map()` `.chain()`）、零成本抽象、最 Rusty | 有状态策略（如 decorrelated jitter）需要 `&mut self` |
| `trait RetryStrategy { fn next_delay(&self, ctx) -> Option<Duration> }` | 可传入上下文 | 过重，组合需手动实现 |
| `enum RetryStrategy { Fixed, Backoff, ... }` | 最简单 | 无法扩展，用户不能自定义策略 |

---

## 退避策略

### 1. 固定间隔 (Fixed Interval)

> 序列：d, d, d, d, d...

```rust
struct FixedInterval {
    interval: Duration,
}

impl Iterator for FixedInterval {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> {
        Some(self.interval)
    }
}
```

- 适用场景：简单场景，负载较轻
- 缺点：高并发下造成"惊群效应"

### 2. 线性退避 (Linear Backoff)

> 序列（base=1s, step=1s）：1, 2, 3, 4, 5...

```rust
struct LinearBackoff {
    current: Duration,
    step: Duration,
}

impl Iterator for LinearBackoff {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> {
        let delay = self.current;
        self.current += self.step;
        Some(delay)
    }
}
```

- 适用场景：需要温和增长的场景

### 3. 指数退避 (Exponential Backoff)

> 序列（base=1s, factor=2）：1, 2, 4, 8, 16...

```rust
struct ExponentialBackoff {
    current: Duration,
    factor: u32,
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> {
        let delay = self.current;
        self.current *= self.factor;
        Some(delay)
    }
}
```

- 特点：间隔快速增长
- **生产环境通常搭配 Jitter 使用**（见下文）

### 4. 斐波那契退避 (Fibonacci Backoff)

> 序列（unit=1s）：1, 1, 2, 3, 5, 8, 13...

```rust
struct FibonacciBackoff {
    a: Duration,
    b: Duration,
}

impl Iterator for FibonacciBackoff {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> {
        let delay = self.a;
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(delay)
    }
}
```

- 特点：增长速度介于线性和指数之间

---

## 组合器（Combinator）

Iterator 抽象的核心优势——策略可以通过组合器自由叠加，而不是为每种组合写一个新类型。

### 1. 最大重试次数

```rust
// 最多重试 5 次，直接用标准库
backoff.take(5)
```

### 2. 最大延迟（截断）

```rust
// 单次延迟不超过 30s
backoff.map(|d| d.min(Duration::from_secs(30)))
```

### 3. Jitter（抖动）

防止惊群效应。这是最重要的组合器，需要自己实现：

```rust
struct Jitter<I> {
    inner: I,
    jitter_range: Duration,
}

impl<I: Iterator<Item = Duration>> Iterator for Jitter<I> {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> {
        self.inner.next().map(|d| {
            let jitter = rand::thread_rng().gen_range(Duration::ZERO..=self.jitter_range);
            d + jitter
        })
    }
}
```

#### Jitter 变体

| 变体 | 公式 | 说明 |
|------|------|------|
| Full Jitter | `random(0, base_delay)` | 完全随机，分散效果最好 |
| Equal Jitter | `base_delay/2 + random(0, base_delay/2)` | 保底一半延迟 |
| Decorrelated Jitter | `random(base, previous * 3)` | 基于上一次延迟，需要状态 |

**生产推荐：Exponential Backoff + Full Jitter**（AWS 官方推荐）

```rust
// 最终用法示例
let backoff = ExponentialBackoff::new(Duration::from_secs(1), 2)
    .full_jitter()              // 加抖动
    .map(|d| d.min(MAX_DELAY))  // 截断
    .take(5);                   // 最多 5 次
```

---

## Retry 函数设计

### 同步版本

```rust
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
```

### 异步版本

```rust
pub async fn retry_async<I, F, Fut, T, E>(backoff: I, mut f: F) -> Result<T, E>
where
    I: IntoIterator<Item = Duration>,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
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
```

**注意**：异步版本硬依赖 tokio。后续可通过 feature flag 支持其他 runtime。

### 条件重试

不是所有错误都应该重试。通过闭包让用户决定：

```rust
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
                    return Err(e); // 不可重试的错误，立即返回
                }
                match iter.next() {
                    Some(delay) => std::thread::sleep(delay),
                    None => return Err(e),
                }
            }
        }
    }
}
```

---

## MVP 范围（v0.1）

只做这些，其他全部砍掉：

- [x] 退避策略：Fixed、Linear、Exponential、Fibonacci
- [ ] 组合器：Jitter（Full Jitter）、MaxDelay（截断）
- [ ] retry 同步函数
- [ ] retry_async 异步函数
- [ ] retry_if 条件重试
- [ ] 基础测试

### 明确不做（v0.1）

- 自适应重试（复杂度高，场景不明确）
- 事件监听 / 回调（YAGNI）
- Circuit Breaker（独立关注点，不属于重试库）
- RetryContext / metadata（过度设计）
- 自定义 RetryError 类型（泛型 `E` 就够了）

---

## 依赖

```toml
[dependencies]
rand = "0.8"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

异步支持通过 feature flag 控制：

```toml
[features]
default = []
tokio = ["dep:tokio"]

[dependencies]
tokio = { version = "1", features = ["time"], optional = true }
```

---

## 参考资料

- [AWS: Exponential Backoff and Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
- [backon](https://github.com/Xuanwo/backon) — Rust 生态中优秀的重试库，Iterator 抽象的典范
- [retry-go](https://github.com/avast/retry-go) — Go 生态的重试库
