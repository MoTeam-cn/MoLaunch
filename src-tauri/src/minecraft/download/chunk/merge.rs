//! 分片合并：按序合并所有 .part 文件到目标文件

use std::io::Write;

/// 按序合并分片到目标文件
pub(super) fn merge_chunks(local_path: &str, chunk_count: usize) -> std::io::Result<()> {
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
