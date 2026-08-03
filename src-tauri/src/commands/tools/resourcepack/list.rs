//! 资源包列目录（list）

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::{resolve_packs_dir, path_to_string};
use super::super::types::{ResourcePackItem, ResourcePackListParams, ResourcePackListResult};

/// 列出 resourcepacks 目录下顶层条目（.zip 文件 → zip；目录 → folder）
pub async fn list(
    state: &AppState,
    params: ResourcePackListParams,
) -> Result<serde_json::Value, String> {
    let packs_dir = resolve_packs_dir(state, params.version_id.as_deref()).await;

    log_info!("[ResourcePack] 列目录: {}", packs_dir.display());

    if !packs_dir.exists() {
        log_warn!(
            "[ResourcePack] resourcepacks 目录不存在: {}",
            packs_dir.display()
        );
        let result = ResourcePackListResult { items: Vec::new() };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    let packs_dir_clone = packs_dir.clone();
    let items = tokio::task::spawn_blocking(move || -> Vec<ResourcePackItem> {
        let mut items: Vec<ResourcePackItem> = Vec::new();
        let read = match std::fs::read_dir(&packs_dir_clone) {
            Ok(r) => r,
            Err(_) => return items,
        };
        for entry in read.flatten() {
            let path = entry.path();
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if path.is_file() {
                if !name.to_lowercase().ends_with(".zip") {
                    continue;
                }
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                items.push(ResourcePackItem {
                    name,
                    path: path_to_string(&path),
                    format: "zip".to_string(),
                    size,
                });
            } else if path.is_dir() {
                let size = dir_total_size(&path);
                items.push(ResourcePackItem {
                    name,
                    path: path_to_string(&path),
                    format: "folder".to_string(),
                    size,
                });
            }
        }
        // 按名称排序，保证输出稳定
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    })
    .await
    .map_err(log_err("ResourcePack 列目录任务失败"))?;

    log_info!("[ResourcePack] 列出 {} 个资源包", items.len());

    let result = ResourcePackListResult { items };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 递归计算目录总字节数
fn dir_total_size(dir: &std::path::Path) -> u64 {
    let mut total: u64 = 0;
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_file() {
            total += std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        } else if path.is_dir() {
            total += dir_total_size(&path);
        }
    }
    total
}