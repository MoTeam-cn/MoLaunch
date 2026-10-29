//! 调度决策的纯函数测试

use super::{
    initial_workers, plan_window, should_grow, should_retire, should_shrink, MAX_WORKERS_PER_FILE,
};

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

#[test]
fn test_plan_window_限流时只缩不扩() {
    let items = [(0_usize, 100_u64, 3_usize), (1, 100, 1)];
    let plan = plan_window(&items, 100, 8, true, 0);
    assert_eq!(plan.shrink, vec![0], "target>1 的源退一条");
    assert_eq!(plan.grow, None, "限流窗口不许扩容");
}

#[test]
fn test_plan_window_明显最快的源扩容() {
    let items = [(0_usize, 400_u64, 2_usize), (1, 100, 1)];
    let plan = plan_window(&items, 250, 8, false, 1);
    assert_eq!(plan.grow, Some(0), "速率高过均值 1.2 倍就该加一条");
    assert!(plan.shrink.is_empty(), "没有源该被收缩");
}

#[test]
fn test_plan_window_全同速时到探针窗口才加() {
    let items = [(0_usize, 100_u64, 2_usize), (1, 100, 2)];
    let quiet = plan_window(&items, 100, 8, false, 3);
    assert_eq!(quiet.grow, None, "非探针窗口、速度都一样就不动");
    let probe = plan_window(&items, 100, 8, false, 5);
    assert_eq!(
        probe.grow,
        Some(0),
        "探针窗口要试加一条，否则空闲额度永远用不上"
    );
}

#[test]
fn test_plan_window_没有空闲额度不探针() {
    let items = [(0_usize, 100_u64, 8_usize)];
    let plan = plan_window(&items, 100, 8, false, 5);
    assert_eq!(plan.grow, None, "已经占满份额就别再加");
    let higher = plan_window(&items, 100, 4, false, 0);
    assert_eq!(higher.shrink, vec![0], "超出份额要收缩");
}

#[test]
fn test_plan_window_单连接的源不会被收缩饿死() {
    let items = [(0_usize, 1_u64, 1_usize)];
    let plan = plan_window(&items, 1000, 1, false, 1);
    assert!(plan.shrink.is_empty(), "只剩一条连接时必须保底");
}
