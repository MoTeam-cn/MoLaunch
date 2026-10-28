//! 分片数规划单测
//!
//! 背景：分片数直接决定单个文件的并行度。用户设置是下限，大文件按 16 MiB/片
//! 放大（最多翻倍、上限 12），小文件保持原值；换源升档在此基础上再提高并行度，
//! 但永不缩小用户设置。布局只依赖输入参数，保证 `.part` 续传可复用。

use super::{plan_chunk_count, MAX_PLANNED_CHUNKS, TARGET_SEGMENT_BYTES};

/// 单流模式（分片数为 1）必须保持单流，不能被放大
#[test]
fn test_plan_chunk_count_单流不放大() {
    assert_eq!(plan_chunk_count(2 * 1024 * 1024 * 1024, 1, 1), 1);
    assert_eq!(plan_chunk_count(0, 1, 2), 1);
    assert_eq!(plan_chunk_count(0, 0, 1), 0);
}

/// 小于目标分片大小的文件保持用户设置，不做缩减也不放大
#[test]
fn test_plan_chunk_count_小文件保持用户设置() {
    assert_eq!(plan_chunk_count(TARGET_SEGMENT_BYTES / 2, 4, 1), 4);
    assert_eq!(plan_chunk_count(TARGET_SEGMENT_BYTES, 4, 1), 4);
    assert_eq!(plan_chunk_count(30 * 1024 * 1024, 3, 1), 3);
}

/// 大文件按 16 MiB/片放大，但最多翻一倍
#[test]
fn test_plan_chunk_count_大文件最多翻倍() {
    // 200 MiB / 16 MiB = 12 片 → 受「最多翻倍」限制为 8
    assert_eq!(plan_chunk_count(200 * 1024 * 1024, 4, 1), 8);
    // 32 MiB / 16 MiB = 2 片 → 低于用户设置的 4，保持 4
    assert_eq!(plan_chunk_count(32 * 1024 * 1024, 4, 1), 4);
    // 4 GiB 仍是 8（用户 4 的翻倍上限）
    assert_eq!(plan_chunk_count(4 * 1024 * 1024 * 1024, 4, 1), 8);
}

/// 用户设置本身很大时，放大结果受硬上限约束
#[test]
fn test_plan_chunk_count_受硬上限约束() {
    assert_eq!(
        plan_chunk_count(4 * 1024 * 1024 * 1024, 6, 1),
        MAX_PLANNED_CHUNKS
    );
    assert_eq!(
        plan_chunk_count(4 * 1024 * 1024 * 1024, 12, 1),
        MAX_PLANNED_CHUNKS
    );
    // 用户设置已超过硬上限时不 panic，也不超过硬上限
    assert_eq!(
        plan_chunk_count(4 * 1024 * 1024 * 1024, 20, 1),
        MAX_PLANNED_CHUNKS
    );
}

/// 同一输入必须得到同一结果（`.part` 分片布局可跨重试复用）
#[test]
fn test_plan_chunk_count_结果稳定() {
    let first = plan_chunk_count(500 * 1024 * 1024, 4, 1);
    for _ in 0..8 {
        assert_eq!(plan_chunk_count(500 * 1024 * 1024, 4, 1), first);
    }
    assert_eq!(first, 8);
}

/// 换源升档：大文件在第二条线路拿到更高并行度
#[test]
fn test_plan_chunk_count_升档提高并行度() {
    // 200 MiB、用户 4 片：不升档 8，升档后 12（硬上限）
    assert_eq!(plan_chunk_count(200 * 1024 * 1024, 4, 2), 12);
    // 32 MiB 文件升档也只到用户设置的 8（4×2），实际受大小约束仍为 8
    assert_eq!(plan_chunk_count(32 * 1024 * 1024, 4, 2), 8);
    // 单流用户不因升档被强行改成多流
    assert_eq!(plan_chunk_count(200 * 1024 * 1024, 1, 2), 1);
}

/// 档位异常值不放大也不 panic（0 按 1 处理）
#[test]
fn test_plan_chunk_count_档位异常按一处理() {
    assert_eq!(plan_chunk_count(200 * 1024 * 1024, 4, 0), 8);
    assert_eq!(
        plan_chunk_count(200 * 1024 * 1024, 4, 4),
        MAX_PLANNED_CHUNKS
    );
}

/// 同一档位重复调用结果稳定（升档后布局固定，才能续传）
#[test]
fn test_plan_chunk_count_升档后结果稳定() {
    let first = plan_chunk_count(500 * 1024 * 1024, 4, 2);
    for _ in 0..8 {
        assert_eq!(plan_chunk_count(500 * 1024 * 1024, 4, 2), first);
    }
}
