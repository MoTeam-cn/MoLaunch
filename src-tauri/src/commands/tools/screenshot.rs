//! 游戏截图管理
//! - `list`：列出 screenshots 目录下所有文件（不递归），按修改时间降序
//!   - 默认扫全局 `{game_dir}/screenshots/`
//!   - 传入 `version_id` 时按版本隔离配置解析该版本的有效游戏目录
//! - `delete`：批量删除截图，删除前校验路径在 screenshots 目录内，防穿越

use std::path::{Path, PathBuf};

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::isolation::{get_effective_game_dir, IsolationMode};
use crate::state::AppState;
use crate::state::resolve_game_dir;

use super::types::{
    ScreenshotDeleteParams, ScreenshotDeleteResult, ScreenshotFailedItem, ScreenshotItem,
    ScreenshotListParams, ScreenshotListResult,
};

/// 解析截图目录
///
/// - 不传 version_id：返回全局 game_dir/screenshots/
/// - 传 version_id：按版本隔离配置解析该版本有效游戏目录后拼接 screenshots/
///
/// 复用 version::list::resolve_isolation_mode + minecraft::isolation::get_effective_game_dir，
/// 与启动/Mod 管理等业务路径完全一致，避免重复实现隔离逻辑。
async fn resolve_shots_dir(state: &AppState, version_id: Option<&str>) -> PathBuf {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    match version_id {
        None => game_dir.join("screenshots"),
        Some(vid) => {
            let global_mode = state.config.lock().await.isolation_mode;
            let isolation_mode =
                crate::commands::version::list::resolve_isolation_mode(&game_dir, vid, global_mode);
            let version_type =
                crate::commands::version::list::detect_version_type_from_dir(&game_dir, vid);
            let mode = IsolationMode::from_u32(isolation_mode);
            let effective_dir =
                get_effective_game_dir(&game_dir, vid, mode, version_type);
            effective_dir.join("screenshots")
        }
    }
}

/// 列出 screenshots 目录下所有文件（不递归），按 modified 降序
pub async fn list(
    state: &AppState,
    params: ScreenshotListParams,
) -> Result<serde_json::Value, String> {
    let shots_dir = resolve_shots_dir(state, params.version_id.as_deref()).await;

    log_info!("[Screenshot] 列目录: {}", shots_dir.display());

    if !shots_dir.exists() {
        log_warn!(
            "[Screenshot] screenshots 目录不存在: {}",
            shots_dir.display()
        );
        let result = ScreenshotListResult {
            items: Vec::new(),
            total_size: 0,
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    let shots_dir_clone = shots_dir.clone();
    let (items, total_size) = tokio::task::spawn_blocking(
        move || -> (Vec<ScreenshotItem>, u64) {
            let mut items: Vec<ScreenshotItem> = Vec::new();
            let mut total_size: u64 = 0;
            let read = match std::fs::read_dir(&shots_dir_clone) {
                Ok(r) => r,
                Err(_) => return (items, total_size),
            };
            for entry in read.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let meta = match std::fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let size = meta.len();
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                total_size += size;
                items.push(ScreenshotItem {
                    path: path_to_string(&path),
                    name,
                    size,
                    modified,
                });
            }
            // 按修改时间降序
            items.sort_by(|a, b| b.modified.cmp(&a.modified));
            (items, total_size)
        },
    )
    .await
    .map_err(log_err("Screenshot 列目录任务失败"))?;

    log_info!(
        "[Screenshot] 列出 {} 个截图，总 {} 字节",
        items.len(),
        total_size
    );

    let result = ScreenshotListResult { items, total_size };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 批量删除截图，校验每个 path 规范化后位于 screenshots 目录内
pub async fn delete(
    state: &AppState,
    params: ScreenshotDeleteParams,
) -> Result<serde_json::Value, String> {
    let shots_dir = resolve_shots_dir(state, params.version_id.as_deref()).await;

    log_info!("[Screenshot] 准备删除 {} 个文件", params.paths.len());

    let paths = params.paths;
    let shots_dir_clone = shots_dir.clone();
    let (deleted_count, freed_bytes, failed) = tokio::task::spawn_blocking(
        move || -> (u64, u64, Vec<ScreenshotFailedItem>) {
            let mut deleted_count: u64 = 0;
            let mut freed_bytes: u64 = 0;
            let mut failed: Vec<ScreenshotFailedItem> = Vec::new();

            let shots_canon = match shots_dir_clone.canonicalize() {
                Ok(c) => c,
                Err(_) => {
                    // screenshots 目录不存在或无法访问，所有删除均失败
                    for raw in paths {
                        failed.push(ScreenshotFailedItem {
                            path: raw,
                            error: "screenshots 目录不存在".to_string(),
                        });
                    }
                    return (deleted_count, freed_bytes, failed);
                }
            };

            for raw in paths {
                let target = PathBuf::from(&raw);
                let target_canon = match target.canonicalize() {
                    Ok(c) => c,
                    Err(e) => {
                        failed.push(ScreenshotFailedItem {
                            path: raw,
                            error: format!("路径解析失败: {}", e),
                        });
                        continue;
                    }
                };
                // 路径安全：规范化后必须在 screenshots 目录内
                if !target_canon.starts_with(&shots_canon) {
                    failed.push(ScreenshotFailedItem {
                        path: raw,
                        error: "路径不在 screenshots 目录内".to_string(),
                    });
                    continue;
                }
                if !target_canon.is_file() {
                    failed.push(ScreenshotFailedItem {
                        path: raw,
                        error: "文件不存在或非普通文件".to_string(),
                    });
                    continue;
                }
                let size = std::fs::metadata(&target_canon)
                    .map(|m| m.len())
                    .unwrap_or(0);
                match std::fs::remove_file(&target_canon) {
                    Ok(()) => {
                        deleted_count += 1;
                        freed_bytes += size;
                    }
                    Err(e) => {
                        failed.push(ScreenshotFailedItem {
                            path: raw,
                            error: e.to_string(),
                        });
                    }
                }
            }
            (deleted_count, freed_bytes, failed)
        },
    )
    .await
    .map_err(log_err("Screenshot 删除任务失败"))?;

    log_info!(
        "[Screenshot] 删除完成: 成功 {}, 释放 {} 字节, 失败 {}",
        deleted_count,
        freed_bytes,
        failed.len()
    );

    let result = ScreenshotDeleteResult {
        deleted_count,
        freed_bytes,
        failed,
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 将路径转为字符串（UTF-8，丢失非 UTF-8 字符）
fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap_or("").to_string()
}