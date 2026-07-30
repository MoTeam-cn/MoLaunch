//! 分片合并：按序合并所有 `.part` 文件到目标文件
//!
//! 合并前校验每个 `.part` 文件大小与期望值匹配，避免部分下载
//! （如服务端提前断流、`bytes_stream` 提前 `Ok(None)`）被误合并为损坏的目标文件。

use std::io::Write;

use crate::log_warn;

/// 按序合并分片到目标文件
///
/// 合并前逐个校验 `.part` 文件大小与期望值匹配，不匹配则返回错误，
/// 避免部分下载被合并为损坏的目标文件。期望值由 `file_size` 与 `chunk_count`
/// 推导，与 `download_chunked` 切分逻辑严格一致：
/// - 前 `chunk_count-1` 个分片：`chunk_size = file_size / chunk_count`
/// - 最后一个分片：`file_size - (chunk_count - 1) * chunk_size`
pub(super) fn merge_chunks(
    local_path: &str,
    chunk_count: usize,
    file_size: u64,
) -> std::io::Result<()> {
    let chunk_size = file_size / chunk_count as u64;

    // 合并前校验：任一分片大小不匹配都拒绝合并
    for i in 0..chunk_count {
        let part_path = format!("{}.part{}", local_path, i);
        let expected = if i == chunk_count - 1 {
            // 最后一片包含整除余数
            file_size - (i as u64) * chunk_size
        } else {
            chunk_size
        };

        match std::fs::metadata(&part_path) {
            Ok(meta) => {
                let actual = meta.len();
                if actual != expected {
                    log_warn!(
                        "[Chunk] 合并校验失败: {} 大小不匹配 (实际 {} != 期望 {})",
                        part_path,
                        actual,
                        expected
                    );
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "part {} 大小不匹配：实际 {}，期望 {}",
                            i, actual, expected
                        ),
                    ));
                }
            }
            Err(e) => {
                log_warn!("[Chunk] 合并校验失败: {} 不存在 ({})", part_path, e);
                return Err(e);
            }
        }
    }

    let tmp_path = format!("{}.merging", local_path);
    {
        let mut output = std::fs::File::create(&tmp_path)?;
        for i in 0..chunk_count {
            let part_path = format!("{}.part{}", local_path, i);
            let mut part_file = std::fs::File::open(&part_path)?;
            std::io::copy(&mut part_file, &mut output)?;
        }
        output.flush()?;
    }
    // 原子替换
    std::fs::rename(&tmp_path, local_path)?;
    Ok(())
}
