//! 拆分提速单测：拆分点计算与拼装

use super::{assemble_split, plan_split_range};

#[test]
fn test_plan_split_range_偶数长度对半分() {
    // [0, 99] 共 100 字节 → 左 [0, 49]，右 [50, 99]
    assert_eq!(plan_split_range(0, 99), Some((0, 49, 50)));
}

#[test]
fn test_plan_split_range_奇数长度左短右长() {
    // [0, 98] 共 99 字节 → 左 [0, 48]（49 字节），右 [49, 98]（50 字节）
    let (left_start, left_end, right_start) = plan_split_range(0, 98).expect("应可拆分");
    assert_eq!((left_start, left_end, right_start), (0, 48, 49));
    let left_len = left_end - left_start + 1;
    let right_len = 98 - right_start + 1;
    assert_eq!(left_len + right_len, 99);
}

#[test]
fn test_plan_split_range_两半覆盖且不重叠() {
    let (left_start, left_end, right_start) =
        plan_split_range(1024, 1024 + 65535).expect("应可拆分");
    assert_eq!(left_start, 1024);
    assert_eq!(left_end + 1, right_start, "左半终点与右半起点必须相邻");
    assert!(left_end >= left_start);
}

#[test]
fn test_plan_split_range_单字节区间不拆() {
    // 只剩 1 个字节时拆分没有意义
    assert_eq!(plan_split_range(100, 100), None);
    // 起点越界（异常输入）也不能拆出负数区间
    assert_eq!(plan_split_range(200, 100), None);
}

#[test]
fn test_plan_split_range_两字节区间各一半() {
    assert_eq!(plan_split_range(7, 8), Some((7, 7, 8)));
}

#[test]
fn test_assemble_split_前缀加两半拼回原文件() {
    let dir = std::env::temp_dir().join(format!("molaunch-split-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    let part = dir.join("f.bin.part0");
    let left = dir.join("f.bin.part0.a");
    let right = dir.join("f.bin.part0.b");
    std::fs::write(&part, b"AAAA").expect("写前缀");
    std::fs::write(&left, b"BBBB").expect("写左半");
    std::fs::write(&right, b"CCCC").expect("写右半");

    assemble_split(
        part.to_str().expect("路径可转字符串"),
        left.to_str().expect("路径可转字符串"),
        right.to_str().expect("路径可转字符串"),
    )
    .expect("拼装应成功");

    let merged = std::fs::read(&part).expect("读拼装结果");
    assert_eq!(merged, b"AAAABBBBCCCC");
    // 两半临时文件应被清理
    assert!(!left.exists());
    assert!(!right.exists());
    let _ = std::fs::remove_dir_all(&dir);
}
