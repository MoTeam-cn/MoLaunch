//! Rate limiter for download speed control

use std::time::Instant;

/// 令牌桶限速器
pub struct RateLimiter {
    /// 每秒允许的字节数
    bytes_per_second: u64,
    /// 当前可用令牌（字节）
    available_tokens: f64,
    /// 上次补充时间
    last_refill: Instant,
    /// 桶容量（允许突发）
    max_tokens: f64,
}

impl RateLimiter {
    pub fn new(bytes_per_second: u64) -> Self {
        let max_tokens = if bytes_per_second > 0 {
            bytes_per_second as f64 * 0.5 // 允许0.5秒的突发
        } else {
            f64::MAX
        };

        Self {
            bytes_per_second,
            available_tokens: max_tokens,
            last_refill: Instant::now(),
            max_tokens,
        }
    }

    /// 尝试获取令牌（字节数），返回实际可用的字节数
    pub fn acquire(&mut self, requested: u64) -> u64 {
        if self.bytes_per_second == 0 {
            return requested; // 不限速
        }

        self.refill();

        let available = self.available_tokens.min(requested as f64);
        if available >= 1.0 {
            let granted = available.floor() as u64;
            self.available_tokens -= granted as f64;
            granted
        } else {
            0 // 需要等待
        }
    }

    /// 补充令牌
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;

        let new_tokens = elapsed * self.bytes_per_second as f64;
        self.available_tokens = (self.available_tokens + new_tokens).min(self.max_tokens);
    }

    /// 获取需要等待的时间（毫秒）
    pub fn wait_time_ms(&self, requested: u64) -> u64 {
        if self.bytes_per_second == 0 {
            return 0;
        }

        let needed = requested as f64 - self.available_tokens;
        if needed <= 0.0 {
            return 0;
        }

        (needed / self.bytes_per_second as f64 * 1000.0) as u64
    }
}

#[cfg(test)]
#[path = "rate_limiter_tests.rs"]
mod tests;
