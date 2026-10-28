//! 段表引擎单测：初始分配与扩容判据

use super::{initial_workers, should_grow, should_shrink, MAX_WORKERS_PER_FILE};

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

#[test]
fn test_should_shrink_明显低于均值才收缩() {
    // 均值 1000，低于一半（500）才收缩
    assert!(should_shrink(400, 1000, 2));
    assert!(!should_shrink(600, 1000, 2));
    assert!(!should_shrink(1000, 1000, 2));
}

#[test]
fn test_should_shrink_单连接保底不减() {
    // 只剩一条连接时不减，否则这条慢源会被自己饿死
    assert!(!should_shrink(0, 1000, 1));
    assert!(!should_shrink(10, 1000, 0));
}

#[test]
fn test_should_shrink_均值为零不收缩() {
    // 全都没产出时由判死逻辑处理，不在这里减连接
    assert!(!should_shrink(0, 0, 3));
}
