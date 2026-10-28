//! 调度决策的纯函数测试

use super::{initial_workers, should_grow, should_retire, should_shrink, MAX_WORKERS_PER_FILE};

#[test]
fn test_initial_workers_单源拿满额度() {
    assert_eq!(initial_workers(1, 8), vec![8]);
    assert_eq!(
        initial_workers(1, MAX_WORKERS_PER_FILE),
        vec![MAX_WORKERS_PER_FILE]
    );
}

#[test]
fn test_initial_workers_多源先各一条() {
    assert_eq!(initial_workers(3, 8), vec![1, 1, 1]);
    assert_eq!(initial_workers(2, 2), vec![1, 1]);
}

#[test]
fn test_initial_workers_额度不足与零源() {
    assert_eq!(initial_workers(3, 2), vec![1, 1, 0]);
    assert!(initial_workers(0, 8).is_empty());
    // 额度为 0 也至少给 1，避免一条连接都不开
    assert_eq!(initial_workers(1, 0), vec![1]);
}

#[test]
fn test_should_grow_快于均值才加() {
    assert!(should_grow(1300, 1000, 1, 4));
    assert!(!should_grow(1100, 1000, 1, 4));
    assert!(!should_grow(2000, 1000, 4, 4), "额度用满就不再扩容");
}

#[test]
fn test_should_grow_均值为零时有产出就加() {
    assert!(should_grow(100, 0, 1, 4));
    assert!(!should_grow(0, 0, 1, 4));
}

#[test]
fn test_should_shrink_明显低于均值才收缩() {
    assert!(should_shrink(400, 1000, 2));
    assert!(!should_shrink(600, 1000, 2));
    assert!(!should_shrink(1000, 1000, 2));
}

#[test]
fn test_should_shrink_单连接保底不减() {
    assert!(!should_shrink(0, 1000, 1));
    assert!(!should_shrink(10, 1000, 0));
}

#[test]
fn test_should_shrink_均值为零不收缩() {
    assert!(!should_shrink(0, 0, 3));
}

#[test]
fn test_should_retire_零产出且有替补才判死() {
    assert!(should_retire(0, 2, 2));
    assert!(!should_retire(0, 1, 2), "唯一活源不能判死");
    assert!(!should_retire(0, 2, 1), "整体只有一条连接时保守处理");
    assert!(!should_retire(1, 2, 2), "有产出就不判死");
}
