//! 列出 saves 目录下所有存档（子文件夹），按名称排序。

use crate::commands::tools::types::{ArchiveItem, ArchiveListParams, ArchiveListResult};
use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::helpers::{dir_total_size, path_to_string};
use super::resolve_saves_dir;

/// 列出 saves 目录下所有存档（子文件夹），按名称排序
pub async fn list(
    state: &AppState,
    params: ArchiveListParams,
) -> Result<serde_json::Value, String> {
    let saves_dir = resolve_saves_dir(state, params.version_id.as_deref()).await;

    log_info!("[Archive] 列目录: {}", saves_dir.display());

    if !saves_dir.exists() {
        log_warn!("[Archive] saves 目录不存在: {}", saves_dir.display());
        let result = ArchiveListResult {
            items: Vec::new(),
            total_size: 0,
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    let saves_dir_clone = saves_dir.clone();
    let (items, total_size) = tokio::task::spawn_blocking(move || -> (Vec<ArchiveItem>, u64) {
        let mut items: Vec<ArchiveItem> = Vec::new();
        let mut total_size: u64 = 0;
        let read = match std::fs::read_dir(&saves_dir_clone) {
            Ok(r) => r,
            Err(_) => return (items, total_size),
        };
        for entry in read.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let size = dir_total_size(&path);
            let modified = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let has_level_dat = path.join("level.dat").is_file();
            total_size += size;
            items.push(ArchiveItem {
                name,
                path: path_to_string(&path),
                size,
                modified,
                has_level_dat,
            });
        }
        // 按名称排序
        items.sort_by(|a, b| a.name.cmp(&b.name));
        (items, total_size)
    })
    .await
    .map_err(log_err("Archive 列目录任务失败"))?;

    log_info!(
        "[Archive] 列出 {} 个存档，总 {} 字节",
        items.len(),
        total_size
    );

    let result = ArchiveListResult { items, total_size };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}
