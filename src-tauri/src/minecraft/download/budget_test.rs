//! 连接额度分摊的纯函数测试

use super::{per_file_share_of, set_total, total, total_for_tasks, FileLease};

#[test]
fn test_per_file_share_按活跃文件数均分() {
    // 总预算 8，单文件上限 12
    assert_eq!(per_file_share_of(8, 1, 12), 8);
    assert_eq!(per_file_share_of(8, 2, 12), 4);
    assert_eq!(per_file_share_of(8, 8, 12), 1);
}

#[test]
fn test_per_file_share_单文件上限仍然生效() {
    // 总预算很大，但用户只让它分 4 片
    assert_eq!(per_file_share_of(64, 1, 4), 4);
    assert_eq!(per_file_share_of(64, 2, 4), 4);
}

#[test]
fn test_per_file_share_拥挤时保底一条() {
    // 文件数超过总预算：每个文件仍有一条，不会归零
    assert_eq!(per_file_share_of(4, 16, 12), 1);
    assert_eq!(per_file_share_of(1, 3, 12), 1);
    assert_eq!(per_file_share_of(0, 0, 0), 1);
}

#[test]
fn test_set_total_按上下限收敛() {
    set_total(0);
    assert_eq!(total(), 1, "零预算没有意义，收敛到 1");
    set_total(999);
    assert_eq!(total(), 64, "超过上限没意义，收敛到 64");
    set_total(8);
    assert_eq!(total(), 8);
}

#[test]
fn test_file_lease_打开与归还() {
    // 只验证租约成对：并发测试下不假设全局计数器的绝对值
    let before = super::active_files();
    {
        let _lease = FileLease::open();
        assert!(super::active_files() >= before + 1);
    }
    assert!(super::active_files() >= before);
}

#[test]
fn test_total_for_tasks_按每任务两条折算() {
    assert_eq!(total_for_tasks(8), 16, "默认 8 线程 -> 16 条总预算");
    assert_eq!(total_for_tasks(1), 2);
    assert_eq!(total_for_tasks(100), 64, "收敛到上限");
    assert_eq!(total_for_tasks(0), 1, "零线程也留一条");
}
