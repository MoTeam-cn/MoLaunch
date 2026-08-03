//! 从 zip 解压恢复存档到 `saves/{world_name}/`。
//!
//! world_name 为空时用 zip 文件名（去 .zip 后缀）；目标目录已存在则返回失败。

use std::path::PathBuf;

use crate::commands::tools::types::{ArchiveRestoreParams, ArchiveRestoreResult};
use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::helpers::unzip_to_dir;
use super::resolve_saves_dir;

/// 从 zip 解压恢复存档到 `saves/{world_name}/`
///
/// world_name 为空时用 zip 文件名（去 .zip 后缀）；目标目录已存在则返回失败
pub async fn restore(
    state: &AppState,
    params: ArchiveRestoreParams,
) -> Result<serde_json::Value, String> {
    let saves_dir = resolve_saves_dir(state, params.version_id.as_deref()).await;

    let zip_path = PathBuf::from(&params.zip_path);
    if !zip_path.is_file() {
        log_warn!("[Archive] zip 文件不存在: {}", zip_path.display());
        let result = ArchiveRestoreResult {
            success: false,
            world_name: String::new(),
            message: format!("zip 文件不存在: {}", zip_path.display()),
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    // world_name 为空时用 zip 文件名（去 .zip 后缀）
    let world_name = if params.world_name.trim().is_empty() {
        zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    } else {
        params.world_name.clone()
    };

    // 路径安全：world_name 不允许为空、含 ".."
    if world_name.is_empty() || !crate::utils::path::is_safe_relative_path(&world_name) {
        log_warn!("[Archive] 非法 world_name: {:?}", world_name);
        let result = ArchiveRestoreResult {
            success: false,
            world_name,
            message: "存档名称非法".to_string(),
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    let target = saves_dir.join(&world_name);
    // 路径安全：target 必须仍位于 saves 目录内（拦截绝对路径等异常输入）
    if !target.starts_with(&saves_dir) {
        log_warn!("[Archive] 目标路径不在 saves 目录内");
        let result = ArchiveRestoreResult {
            success: false,
            world_name,
            message: "目标路径不在 saves 目录内".to_string(),
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }
    // 目标目录已存在则返回失败
    if target.exists() {
        log_warn!("[Archive] 目标目录已存在: {}", target.display());
        let result = ArchiveRestoreResult {
            success: false,
            world_name,
            message: format!("目标目录已存在: {}", target.display()),
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    log_info!(
        "[Archive] 恢复: {} -> {}",
        zip_path.display(),
        target.display()
    );

    let target_clone = target.clone();
    let restore_result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        unzip_to_dir(&zip_path, &target_clone)
    })
    .await
    .map_err(log_err("Archive 恢复任务失败"))?;

    match restore_result {
        Ok(()) => {
            log_info!("[Archive] 恢复成功: {}", target.display());
            let result = ArchiveRestoreResult {
                success: true,
                world_name,
                message: "恢复成功".to_string(),
            };
            serde_json::to_value(&result).map_err(|e| e.to_string())
        }
        Err(e) => {
            log_warn!("[Archive] 恢复失败: {}", e);
            // 解压失败时清理可能已创建的部分目录
            let _ = std::fs::remove_dir_all(&target);
            let result = ArchiveRestoreResult {
                success: false,
                world_name,
                message: e,
            };
            serde_json::to_value(&result).map_err(|e| e.to_string())
        }
    }
}
