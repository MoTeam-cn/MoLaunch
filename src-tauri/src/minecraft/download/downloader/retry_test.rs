//! 分片策略测试

use super::{chunkable_size, stream_expected, MIN_CHUNK_BYTES};

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

#[test]
fn test_stream_expected_响应给出可信长度时用任务大小() {
    assert_eq!(stream_expected(1401, 1401), 1401);
    assert_eq!(stream_expected(0, 2778), 0);
}

#[test]
fn test_stream_expected_响应无长度时不校验大小() {
    // 压缩响应被解码、或 chunked 传输：服务端自报长度不可信，跳过大小校验
    assert_eq!(stream_expected(1401, 0), 0);
    assert_eq!(stream_expected(10_000, 0), 0);
}
