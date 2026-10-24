//! 服务端时间对齐测试

use super::*;

#[test]
fn test_compute_offset_服务端更快为正() {
    assert_eq!(compute_offset(1_000, 900), 100);
}

#[test]
fn test_compute_offset_本机更快为负() {
    assert_eq!(compute_offset(900, 1_000), -100);
}

#[test]
fn test_compute_offset_一致为零() {
    assert_eq!(compute_offset(1_700_000_000, 1_700_000_000), 0);
}

#[test]
fn test_compute_offset_极端值不溢出() {
    assert_eq!(compute_offset(u64::MAX, 0), u64::MAX as i64);
    assert_eq!(compute_offset(0, u64::MAX), -(u64::MAX as i64));
}

#[test]
fn test_server_now_未对齐时等于本机时间() {
    // 未对齐（LAST_SYNC_AT = 0）时偏移为 0，直接反映本机时钟
    let local = local_now();
    let now = server_now();
    assert!(now.abs_diff(local) <= 2, "未对齐时 server_now 应≈本机时间");
}
