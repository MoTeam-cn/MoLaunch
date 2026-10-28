//! 段表单测：块划分、认领顺序、失败隔离、完成判定

use super::{seg_len_for, segments_for, SegmentTable, DEFAULT_SEG_LEN, MAX_SEGS, MAX_SEG_FAILURES};

#[test]
fn test_segments_for_全覆盖且不重叠() {
    let ranges = segments_for(10 * 1024 * 1024 + 123, DEFAULT_SEG_LEN);
    assert_eq!(ranges.first().expect("非空").0, 0);
    assert_eq!(ranges.last().expect("非空").1, 10 * 1024 * 1024 + 122);
    for pair in ranges.windows(2) {
        assert_eq!(
            pair[1].0,
            pair[0].1 + 1,
            "相邻块必须首尾相接，不能有空洞或重叠"
        );
    }
}

#[test]
fn test_segments_for_小文件只有一块() {
    let ranges = segments_for(1024, DEFAULT_SEG_LEN);
    assert_eq!(ranges, vec![(0, 1023)]);
}

#[test]
fn test_segments_for_零大小没有块() {
    assert!(segments_for(0, DEFAULT_SEG_LEN).is_empty());
    assert!(segments_for(1024, 0).is_empty());
}

#[test]
fn test_seg_len_for_超大文件增大块长以控制块数() {
    // 小文件用默认块长
    assert_eq!(seg_len_for(100 * 1024 * 1024), DEFAULT_SEG_LEN);
    // 极大文件：块数不超过上限
    let huge = 256_u64 * 1024 * 1024 * 1024;
    let seg_len = seg_len_for(huge);
    assert!(seg_len > DEFAULT_SEG_LEN);
    assert!(huge.div_ceil(seg_len) <= MAX_SEGS);
}

#[test]
fn test_claim_从尾部开始并标记在飞() {
    let table = SegmentTable::new(3 * DEFAULT_SEG_LEN);
    let first = table.claim(false).expect("应能认领");
    assert_eq!(first.index, 2, "尾块最难凑齐，应优先派出");
    let second = table.claim(false).expect("应能认领");
    assert_eq!(second.index, 1);
    let third = table.claim(false).expect("应能认领");
    assert_eq!(third.index, 0);
    assert!(table.claim(false).is_none(), "全部在飞时不应重复派出");
}

#[test]
fn test_complete_累加字节并可判定完成() {
    let table = SegmentTable::new(2 * DEFAULT_SEG_LEN);
    assert!(!table.is_complete());
    assert_eq!(table.done_bytes(), 0);
    let seg = table.claim(false).expect("应能认领");
    table.complete(seg.index);
    assert_eq!(table.done_bytes(), DEFAULT_SEG_LEN);
    assert!(!table.is_complete());
    let seg = table.claim(false).expect("应能认领");
    table.complete(seg.index);
    assert!(table.is_complete());
    assert_eq!(table.done_bytes(), 2 * DEFAULT_SEG_LEN);
}

#[test]
fn test_fail_释放回待领() {
    let table = SegmentTable::new(DEFAULT_SEG_LEN);
    let seg = table.claim(false).expect("应能认领");
    table.fail(seg.index);
    let again = table.claim(false).expect("失败后应可重新认领");
    assert_eq!(again.index, seg.index);
}

#[test]
fn test_fail_达到阈值后隔离且镜像不能再领() {
    let table = SegmentTable::new(DEFAULT_SEG_LEN);
    for _ in 0..MAX_SEG_FAILURES {
        let seg = table.claim(false).expect("应能认领");
        table.fail(seg.index);
    }
    assert!(table.claim(false).is_none(), "隔离的块不应再交给镜像源");
    let official = table.claim(true).expect("官方源仍可领隔离块");
    assert_eq!(official.index, 0);
}

#[test]
fn test_完成后再失败不影响已有进度() {
    let table = SegmentTable::new(DEFAULT_SEG_LEN);
    let seg = table.claim(false).expect("应能认领");
    table.complete(seg.index);
    table.fail(seg.index);
    assert_eq!(table.done_bytes(), DEFAULT_SEG_LEN);
    assert!(table.is_complete());
}
