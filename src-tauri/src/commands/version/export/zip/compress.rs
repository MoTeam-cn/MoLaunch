//! Compress 格式（.zip 兜底）：仅 `.minecraft/` 前缀，无 manifest

use std::path::Path;

use tauri::AppHandle;

use crate::commands::version::export::types::ExportFileInfo;
use crate::log_info;

use super::helpers::{create_zip_writer, emit_zip_progress, write_file_entry};

/// 构建 Compress 格式整合包（.zip 兜底）
///
/// 直接打包 .minecraft/ 前缀，无 manifest 文件。
pub(super) fn build_compress_zip(
    files: &[ExportFileInfo],
    pack_path: &Path,
    app: &AppHandle,
    version_id: &str,
) -> Result<(), String> {
    log_info!(
        "[Export] Compress 打包：总 {} 文件，全部打包到 .minecraft/",
        files.len()
    );

    let (mut zip, options) = create_zip_writer(pack_path)?;

    let total = files.len();
    for (i, f) in files.iter().enumerate() {
        let zip_path = format!(".minecraft/{}", f.relative_path);
        write_file_entry(&mut zip, options, &zip_path, &f.abs_path)?;
        emit_zip_progress(app, version_id, i + 1, total);
    }
    log_info!("[Export] .minecraft/ 文件写入完成");

    zip.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}
