//! 慢速判定单测

use super::{is_slow, SlowDetector, FLOOR_BYTES_PER_SEC, MIN_REMAINING, WINDOW};
use std::time::Duration;

/// 窗口未满时不判慢（避免刚连上就误杀）
#[test]
fn test_is_slow_窗口未满不判慢() {
    assert_eq!(is_slow(0, WINDOW / 2, 64 * 1024 * 1024), None);
    assert_eq!(is_slow(0, Duration::ZERO, 64 * 1024 * 1024), None);
}

/// 剩余数据不足阈值时不判慢（收尾阶段抖动大）
#[test]
fn test_is_slow_收尾阶段不判慢() {
    assert_eq!(is_slow(0, WINDOW, MIN_REMAINING - 1), None);
    // 剩余量未知（0）同样不判慢
    assert_eq!(is_slow(0, WINDOW, 0), None);
}

/// 窗口满且吞吐低于下限时给出平均吞吐
#[test]
fn test_is_slow_低速命中() {
    // 5s 内只收到 100 KiB → 约 20 KiB/s，远低于 256 KiB/s
    let verdict = is_slow(100 * 1024, WINDOW, 64 * 1024 * 1024);
    assert_eq!(verdict, Some(20 * 1024));
}

/// 吞吐达到下限时不判慢
#[test]
fn test_is_slow_达标不判慢() {
    // 5s 内收到 2 MiB → 约 410 KiB/s
    assert_eq!(is_slow(2 * 1024 * 1024, WINDOW, 64 * 1024 * 1024), None);
    // 正好等于下限附近也应放行（严格小于才判慢）
    let exactly = FLOOR_BYTES_PER_SEC * WINDOW.as_secs();
    assert_eq!(is_slow(exactly, WINDOW, 64 * 1024 * 1024), None);
}

/// 探测器在窗口未满时不结算、也不清零（否则会把一个窗口拆成多个短窗）
#[test]
fn test_slow_detector_窗口未满不结算() {
    let mut detector = SlowDetector::new();
    detector.add(1024);
    detector.add(1024);
    // 窗口刚开，不判慢（也不重置）
    assert_eq!(detector.poll(64 * 1024 * 1024), None);
    detector.add(1024);
    assert_eq!(detector.poll(64 * 1024 * 1024), None);
}
