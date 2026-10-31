//! 位图落盘与数据刷盘：这两件事都发生在「一块数据可信与否」的边界上

use std::path::Path;

use crate::log_error;

use super::table::SegmentTable;

/// 把数据文件刷到稳定存储。
///
/// 必须用写句柄打开：Windows 的 FlushFileBuffers 要求句柄带写权限，
/// 只读句柄的 sync_all 会直接失败——那样等于没刷。
pub(crate) fn flush_data_file(path: &str) -> std::io::Result<()> {
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.sync_all()
}

/// 保存续传位图
///
/// 没有指纹（不允许续传）时什么都不做：留着也没人会信它。
pub(crate) fn save_map(table: &SegmentTable, map_path: &str, resume_key: Option<u64>) {
    let Some(key) = resume_key else {
        return;
    };
    if let Err(e) = table.save(Path::new(map_path), key) {
        crate::log_error!("[Segment] 续传位图落盘失败，下次会从头下载: {}", e);
    }
}

#[cfg(test)]
#[path = "map_io_test.rs"]
mod map_io_test;
