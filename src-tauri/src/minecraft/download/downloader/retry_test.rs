//! 分片策略测试

use super::{chunkable_size, MIN_CHUNK_BYTES};

#[test]
fn test_chunkable_大文件且多分片时成立() {
    assert!(chunkable_size(10 * MIN_CHUNK_BYTES, 4));
}

#[test]
fn test_chunkable_小文件不分片() {
    // 元数据 JSON（本次 Fabric profile 只有 2.7 KB）不该分片
    assert!(!chunkable_size(2778, 4));
    assert!(!chunkable_size(MIN_CHUNK_BYTES - 1, 4));
}

#[test]
fn test_chunkable_大小未知不分片() {
    assert!(!chunkable_size(0, 4));
}

#[test]
fn test_chunkable_单分片不分片() {
    assert!(!chunkable_size(10 * MIN_CHUNK_BYTES, 1));
}
