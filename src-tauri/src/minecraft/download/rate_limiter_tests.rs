//! RateLimiter 单元测试

use super::*;

#[test]
fn test_unlimited_limiter() {
    // bytes_per_second=0 表示不限速
    let mut limiter = RateLimiter::new(0);
    // 不限速时 acquire 总是返回请求数
    assert_eq!(limiter.acquire(1000), 1000);
    assert_eq!(limiter.acquire(1_000_000), 1_000_000);
    // wait_time_ms 总是 0
    assert_eq!(limiter.wait_time_ms(1000), 0);
}

#[test]
fn test_acquire_within_bucket() {
    // 1000 bytes/sec，桶容量 500（0.5 秒突发）
    let mut limiter = RateLimiter::new(1000);
    // 初始有 500 令牌，请求 300 应全部满足
    assert_eq!(limiter.acquire(300), 300);
    // 剩余 200 令牌，请求 300 只能拿到 200
    assert_eq!(limiter.acquire(300), 200);
}

#[test]
fn test_acquire_empty_bucket() {
    let mut limiter = RateLimiter::new(1000);
    // 耗尽令牌（500）
    limiter.acquire(500);
    // 桶空时返回 0
    assert_eq!(limiter.acquire(100), 0);
}

#[test]
fn test_wait_time_ms_calculation() {
    let limiter = RateLimiter::new(1000); // 1000 bytes/sec
                                          // 耗尽令牌（500）
    let mut limiter = limiter;
    limiter.acquire(500);
    // 请求 1000 字节，缺 1000，需等待 1000ms
    assert_eq!(limiter.wait_time_ms(1000), 1000);
    // 请求 500 字节，缺 500，需等待 500ms
    assert_eq!(limiter.wait_time_ms(500), 500);
}

#[test]
fn test_wait_time_ms_with_available_tokens() {
    let limiter = RateLimiter::new(1000);
    // 初始有 500 令牌，请求 300，wait_time_ms=0（令牌足够）
    assert_eq!(limiter.wait_time_ms(300), 0);
}

#[test]
fn test_refill_after_time() {
    let mut limiter = RateLimiter::new(1000);
    // 耗尽令牌
    limiter.acquire(500);
    assert_eq!(limiter.acquire(100), 0);
    // 等待一小段时间（让 refill 生效）
    std::thread::sleep(std::time::Duration::from_millis(50));
    // 50ms 后应补充约 50 字节（1000 bytes/sec * 0.05s = 50）
    let granted = limiter.acquire(100);
    // 由于时间精度，granted 应在 40-60 范围内（允许误差）
    assert!(
        granted >= 40 && granted <= 60,
        "granted={} 应在 40-60 范围内",
        granted
    );
}
