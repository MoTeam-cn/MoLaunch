//! 段表引擎单测：初始分配与扩容判据

use super::{initial_workers, should_grow, MAX_WORKERS_PER_FILE};

#[test]
fn test_initial_workers_单源拿到全部额度() {
    assert_eq!(initial_workers(1, 8), vec![8]);
    assert_eq!(initial_workers(1, 0), vec![1], "额度至少为 1");
}

#[test]
fn test_initial_workers_多源先各一条() {
    assert_eq!(initial_workers(2, 8), vec![1, 1]);
    assert_eq!(initial_workers(3, 8), vec![1, 1, 1]);
}

#[test]
fn test_initial_workers_额度不足时优先前面的源() {
    assert_eq!(initial_workers(3, 2), vec![1, 1, 0]);
    assert_eq!(initial_workers(2, 1), vec![1, 0]);
}

#[test]
fn test_initial_workers_无源返回空() {
    assert!(initial_workers(0, 8).is_empty());
}

#[test]
fn test_should_grow_明显更快才扩容() {
    // 均值 1000，阈值 1.2 倍
    assert!(should_grow(1300, 1000, 2, 8));
    assert!(!should_grow(1100, 1000, 2, 8));
    assert!(!should_grow(1000, 1000, 2, 8));
}

#[test]
fn test_should_grow_额度用尽不扩容() {
    assert!(!should_grow(
        9999,
        1000,
        MAX_WORKERS_PER_FILE,
        MAX_WORKERS_PER_FILE
    ));
}

#[test]
fn test_should_grow_均值为零时只要有产出就扩() {
    assert!(should_grow(1, 0, 1, 8));
    assert!(!should_grow(0, 0, 1, 8));
}
