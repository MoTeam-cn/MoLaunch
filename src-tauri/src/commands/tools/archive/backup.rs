//! 将存档打包为 zip（可选排除玩家数据用于分享）
//!
//! 失败返回 `success=false`（不返回 Err），由前端展示失败提示。

use std::path::PathBuf;

use crate::commands::tools::types::{ArchiveBackupParams, ArchiveBackupResult};
use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::helpers::zip_directory;
use super::resolve_saves_dir;

/// 将存档打包为 zip（可选排除玩家数据用于分享）
///
/// 失败返回 `success=false`（不返回 Err），由前端展示失败提示
pub async fn backup(
    state: &AppState,
    params: ArchiveBackupParams,
) -> Result<serde_json::Value, String> {
    let saves_dir = resolve_saves_dir(state, params.version_id.as_deref()).await;

    // 路径安全：world_name 不允许为空、含 ".." 或路径分隔符
    if params.world_name.is_empty()
        || params.world_name.contains("..")
        || params.world_name.contains('/')
        || params.world_name.contains('\\')
    {
        log_warn!("[Archive] 非法 world_name: {:?}", params.world_name);
        let result = ArchiveBackupResult {
            success: false,
            file_path: String::new(),
            file_size: 0,
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    let output_path = PathBuf::from(&params.output_path);
    // output_path 父目录必须存在
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            log_warn!("[Archive] 输出目录不存在: {}", parent.display());
            let result = ArchiveBackupResult {
                success: false,
                file_path: String::new(),
                file_size: 0,
            };
            return serde_json::to_value(&result).map_err(|e| e.to_string());
        }
    }

    // 源目录解析 + 路径安全：规范化后必须在 saves 目录内
    let saves_canon = match saves_dir.canonicalize() {
        Ok(c) => c,
        Err(e) => {
            log_warn!("[Archive] saves 目录解析失败: {}", e);
            let result = ArchiveBackupResult {
                success: false,
                file_path: String::new(),
                file_size: 0,
            };
            return serde_json::to_value(&result).map_err(|e| e.to_string());
        }
    };
    let source = saves_dir.join(&params.world_name);
    let source_canon = match source.canonicalize() {
        Ok(c) => c,
        Err(e) => {
            log_warn!("[Archive] 存档目录不存在: {} ({})", source.display(), e);
            let result = ArchiveBackupResult {
                success: false,
                file_path: String::new(),
                file_size: 0,
            };
            return serde_json::to_value(&result).map_err(|e| e.to_string());
        }
    };
    if !source_canon.starts_with(&saves_canon) {
        log_warn!("[Archive] 源路径不在 saves 目录内");
        let result = ArchiveBackupResult {
            success: false,
            file_path: String::new(),
            file_size: 0,
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    log_info!(
        "[Archive] 备份: {} -> {} (exclude_player_data={})",
        source_canon.display(),
        output_path.display(),
        params.exclude_player_data
    );

    let exclude_player_data = params.exclude_player_data;
    let output_path_clone = output_path.clone();
    let backup_result = tokio::task::spawn_blocking(move || -> Result<u64, String> {
        let exclude: Vec<&str> = if exclude_player_data {
            vec!["playerdata"]
        } else {
            Vec::new()
        };
        zip_directory(&source_canon, &output_path_clone, &exclude)?;
        let size = std::fs::metadata(&output_path_clone)
            .map(|m| m.len())
            .unwrap_or(0);
        Ok(size)
    })
    .await
    .map_err(log_err("Archive 备份任务失败"))?;

    match backup_result {
        Ok(size) => {
            log_info!(
                "[Archive] 备份成功: {} ({} 字节)",
                output_path.display(),
                size
            );
            let result = ArchiveBackupResult {
                success: true,
                file_path: params.output_path,
                file_size: size,
            };
            serde_json::to_value(&result).map_err(|e| e.to_string())
        }
        Err(e) => {
            log_warn!("[Archive] 备份失败: {}", e);
            let result = ArchiveBackupResult {
                success: false,
                file_path: String::new(),
                file_size: 0,
            };
            serde_json::to_value(&result).map_err(|e| e.to_string())
        }
    }
}
