//! 段表单测：块划分、认领、失败隔离、来源作废、位图落盘

use super::{seg_len_for, segments_for, SegmentTable, DEFAULT_SEG_LEN, MAX_SEGS, MAX_SEG_FAILURES};
use std::path::PathBuf;

fn temp_map(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("molaunch-segmap-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    dir.join(name)
}

#[test]
fn test_segments_for_全覆盖且不重叠() {
    let ranges = segments_for(10 * 1024 * 1024 + 123, DEFAULT_SEG_LEN);
    assert_eq!(ranges.first().expect("非空").0, 0);
    assert_eq!(ranges.last().expect("非空").1, 10 * 1024 * 1024 + 122);
    for pair in ranges.windows(2) {
        assert_eq!(pair[1].0, pair[0].1 + 1, "相邻块必须首尾相接");
    }
}

#[test]
fn test_segments_for_小文件只有一块() {
    assert_eq!(segments_for(1024, DEFAULT_SEG_LEN), vec![(0, 1023)]);
}

#[test]
fn test_segments_for_零大小没有块() {
    assert!(segments_for(0, DEFAULT_SEG_LEN).is_empty());
    assert!(segments_for(1024, 0).is_empty());
}

#[test]
fn test_seg_len_for_超大文件增大块长以控制块数() {
    assert_eq!(seg_len_for(100 * 1024 * 1024), DEFAULT_SEG_LEN);
    let huge = 256_u64 * 1024 * 1024 * 1024;
    let seg_len = seg_len_for(huge);
    assert!(seg_len > DEFAULT_SEG_LEN);
    assert!(huge.div_ceil(seg_len) <= MAX_SEGS);
}

#[test]
fn test_claim_从尾部开始并标记在飞() {
    let table = SegmentTable::new(3 * DEFAULT_SEG_LEN);
    assert_eq!(table.claim(false).expect("应能认领").index, 2);
    assert_eq!(table.claim(false).expect("应能认领").index, 1);
    assert_eq!(table.claim(false).expect("应能认领").index, 0);
    assert!(table.claim(false).is_none(), "全部在飞时不应重复派出");
}

#[test]
fn test_complete_累加字节并可判定完成() {
    let table = SegmentTable::new(2 * DEFAULT_SEG_LEN);
    assert!(!table.is_complete());
    assert_eq!(table.done_bytes(), 0);
    let seg = table.claim(false).expect("应能认领");
    table.complete(seg.index, 0);
    assert_eq!(table.done_bytes(), DEFAULT_SEG_LEN);
    assert_eq!(table.done_count(), 1);
    assert!(!table.is_complete());
    let seg = table.claim(false).expect("应能认领");
    table.complete(seg.index, 0);
    assert!(table.is_complete());
    assert_eq!(table.done_bytes(), 2 * DEFAULT_SEG_LEN);
}

#[test]
fn test_fail_释放回待领() {
    let table = SegmentTable::new(DEFAULT_SEG_LEN);
    let seg = table.claim(false).expect("应能认领");
    table.fail(seg.index);
    assert_eq!(table.claim(false).expect("应可重新认领").index, seg.index);
}

#[test]
fn test_fail_达到阈值后隔离且镜像不能再领() {
    let table = SegmentTable::new(DEFAULT_SEG_LEN);
    for _ in 0..MAX_SEG_FAILURES {
        let seg = table.claim(false).expect("应能认领");
        table.fail(seg.index);
    }
    assert!(table.claim(false).is_none(), "隔离的块不应再交给镜像源");
    assert_eq!(table.claim(true).expect("官方源仍可领").index, 0);
}

#[test]
fn test_完成后再失败不影响已有进度() {
    let table = SegmentTable::new(DEFAULT_SEG_LEN);
    let seg = table.claim(false).expect("应能认领");
    table.complete(seg.index, 0);
    table.fail(seg.index);
    assert_eq!(table.done_bytes(), DEFAULT_SEG_LEN);
    assert!(table.is_complete());
}

#[test]
fn test_invalidate_others_只作废非指定源的块() {
    let table = SegmentTable::new(3 * DEFAULT_SEG_LEN);
    // 块 2 由源 0（官方）完成，块 1 由源 1（镜像）完成
    table.complete(2, 0);
    table.claim(false);
    table.complete(1, 1);
    assert_eq!(table.done_count(), 2);

    let invalidated = table.invalidate_others(0);
    assert_eq!(invalidated, 1, "只应作废镜像下的那一块");
    assert_eq!(table.done_count(), 1);
    // 作废的块重新可领
    assert!(table.has_claimable());
}

#[test]
fn test_save_load_往返保留完成块与来源() {
    let path = temp_map("roundtrip.partmap");
    let table = SegmentTable::new(3 * DEFAULT_SEG_LEN);
    table.complete(2, 0);
    table.claim(false);
    table.complete(1, 1);
    table.save(&path).expect("保存位图");

    let file_size = 3 * DEFAULT_SEG_LEN;
    let restored = SegmentTable::load(&path, file_size, file_size).expect("应能读回");
    assert_eq!(restored.done_count(), 2);
    assert_eq!(restored.done_bytes(), 2 * DEFAULT_SEG_LEN);
    // 只保留官方源 -> 镜像那块的来源信息仍在，作废时能定位
    assert_eq!(restored.invalidate_others(0), 1);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_load_文件大小不匹配则拒绝() {
    let path = temp_map("mismatch.partmap");
    let table = SegmentTable::new(2 * DEFAULT_SEG_LEN);
    table.save(&path).expect("保存位图");
    assert!(SegmentTable::load(&path, 4 * DEFAULT_SEG_LEN, u64::MAX).is_none());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_load_数据文件没写到的块不算完成() {
    let path = temp_map("short.partmap");
    let file_size = 2 * DEFAULT_SEG_LEN;
    let table = SegmentTable::new(file_size);
    table.complete(1, 0);
    table.complete(0, 0);
    table.save(&path).expect("保存位图");
    // 数据文件只有半块长：两块都应被判为未完成
    let restored = SegmentTable::load(&path, file_size, DEFAULT_SEG_LEN / 2).expect("应能读回");
    assert_eq!(restored.done_count(), 0);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_load_损坏的位图返回None() {
    let path = temp_map("broken.partmap");
    std::fs::write(&path, b"broken").expect("写入脏数据");
    assert!(SegmentTable::load(&path, DEFAULT_SEG_LEN, u64::MAX).is_none());
    let _ = std::fs::remove_file(&path);
}
