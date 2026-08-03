//! 清理执行实现：删除严格限定在扫描允许目录内的路径
//! execute 时再次做路径安全检查，杜绝路径遍历。

use std::path::PathBuf;

use super::super::types::{CleanupExecuteParams, CleanupExecuteResult, CleanupFailedItem};
use super::fs;
use super::scan::build_allowed_parents;
use crate::config;
use crate::log_info;
use crate::state::resolve_game_dir;

/// 执行清理：删除指定路径下的所有文件（含空目录）
///
/// 安全检查：每个 path 必须在 `build_allowed_parents` 返回的允许目录内，
/// 拒绝路径遍历。
pub async fn execute(params: CleanupExecuteParams) -> Result<serde_json::Value, String> {
    let game_dir = {
        let config = config::load_config()
            .map_err(|e| format!("加载配置失败: {}", e))?
            .unwrap_or_default();
        resolve_game_dir(&config.game_dir)
    };

    let allowed_parents = build_allowed_parents(&game_dir);

    let mut cleaned_size: u64 = 0;
    let mut cleaned_files: u64 = 0;
    let mut failed: Vec<CleanupFailedItem> = Vec::new();

    for path_str in &params.paths {
        let path = PathBuf::from(path_str);

        if !fs::is_path_safe(&path, &allowed_parents) {
            failed.push(CleanupFailedItem {
                path: path_str.clone(),
                error: "路径不在允许的清理目录内".to_string(),
            });
            continue;
        }

        if !path.exists() {
            continue;
        }

        match fs::remove_dir_recursive(&path) {
            Ok((size, files)) => {
                cleaned_size += size;
                cleaned_files += files;
            }
            Err(e) => {
                failed.push(CleanupFailedItem {
                    path: path_str.clone(),
                    error: e,
                });
            }
        }
    }

    log_info!(
        "[Cleanup] 清理完成: 已清理 {} 文件 / {} 字节, 失败 {} 项",
        cleaned_files,
        cleaned_size,
        failed.len()
    );

    let result = CleanupExecuteResult {
        cleaned_size,
        cleaned_files,
        failed,
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}