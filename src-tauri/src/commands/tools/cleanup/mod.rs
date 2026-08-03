//! 清理游戏垃圾文件（扫描 `.minecraft` 下可清理内容，分三类）
//! 根目录固定子目录：logs/crash-reports/.mixin.out/assets/cache/.fabric/remapCache/screenshots；
//! 版本目录下子目录（版本隔离模式）：logs/crash-reports/.mixin.out/.fabric/processedMods/
//! remappedJars；原生库提取目录 `<ver>-natives`（非 natives）。
//! `execute` 删除时严格路径安全检查，仅删扫描阶段发现的目录，避免路径遍历攻击。

mod fs;

use std::path::{Path, PathBuf};

use crate::log_info;
use crate::state::resolve_game_dir;
use crate::state::AppState;

use super::types::{
    CleanupExecuteParams, CleanupExecuteResult, CleanupFailedItem, CleanupItem, CleanupScanResult,
};

/// 根目录固定子目录扫描配置：(相对路径, 显示名, category)
///
/// category 为 "可清理" 的项默认选中，"可选" 的项默认不选中。
/// 兼容非版本隔离布局（isolation_mode=0），版本隔离模式下这些目录通常不存在。
const ROOT_SCAN_DIRS: &[(&str, &str, &str)] = &[
    ("logs", "游戏日志", "可清理"),
    ("crash-reports", "崩溃报告", "可清理"),
    (".mixin.out", "Mixin 编译输出", "可清理"),
    ("assets/cache", "资源索引缓存", "可清理"),
    (".fabric/remapCache", "Fabric 重映射缓存", "可清理"),
    ("screenshots", "截图", "可选"),
];

/// 版本目录下子目录扫描配置：(相对路径, 显示名, category)
///
/// MoLaunch 默认启用版本隔离（isolation_mode=4），日志、崩溃报告、Fabric 缓存等
/// 实际位于 `versions/<ver>/` 下而非 `.minecraft` 根目录。
const VERSION_SCAN_DIRS: &[(&str, &str, &str)] = &[
    ("logs", "游戏日志", "可清理"),
    ("crash-reports", "崩溃报告", "可清理"),
    (".mixin.out", "Mixin 编译输出", "可清理"),
    (".fabric/processedMods", "Fabric 处理缓存", "可清理"),
    (".fabric/remappedJars", "Fabric 重映射缓存", "可清理"),
];

/// 构建所有允许清理的目录列表
///
/// 包含：根目录固定子目录 + 各版本目录下子目录 + 各版本 natives 目录。
/// scan 与 execute 共用此函数，确保安全检查与扫描结果完全一致。
fn build_allowed_parents(game_dir: &Path) -> Vec<PathBuf> {
    let mut parents: Vec<PathBuf> = ROOT_SCAN_DIRS
        .iter()
        .map(|(sub, _, _)| game_dir.join(sub))
        .collect();

    // versions/<ver>/<sub> 和 versions/<ver>/<ver>-natives
    let versions_dir = game_dir.join("versions");
    if versions_dir.exists() {
        if let Ok(read) = std::fs::read_dir(&versions_dir) {
            let mut version_names: Vec<String> = read
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            version_names.sort();

            for version_name in &version_names {
                let version_path = versions_dir.join(version_name);
                // 版本目录下的固定子目录
                for (sub, _, _) in VERSION_SCAN_DIRS {
                    let dir = version_path.join(sub);
                    if dir.exists() {
                        parents.push(dir);
                    }
                }
                // natives 目录（命名约定：<ver>-natives）
                let natives_dir = version_path.join(format!("{}-natives", version_name));
                if natives_dir.exists() {
                    parents.push(natives_dir);
                }
            }
        }
    }

    parents
}

/// 扫描 `.minecraft` 下的可清理内容
///
/// 扫描顺序：根目录固定子目录 → 各版本目录下子目录 → 各版本 natives 目录
pub async fn scan(state: &AppState) -> Result<serde_json::Value, String> {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };

    let mut items: Vec<CleanupItem> = Vec::new();
    let mut total_size: u64 = 0;
    let mut total_files: u64 = 0;

    // 1. 根目录固定子目录（兼容非版本隔离布局）
    for (sub, display_name, category) in ROOT_SCAN_DIRS {
        let dir = game_dir.join(sub);
        if let Some(item) = fs::scan_directory(&dir, display_name, category) {
            total_size += item.size;
            total_files += item.file_count;
            items.push(item);
        }
    }

    // 2. versions/<ver>/<sub>：版本隔离模式下每个版本目录下的子目录
    let versions_dir = game_dir.join("versions");
    if versions_dir.exists() {
        if let Ok(read) = std::fs::read_dir(&versions_dir) {
            let mut version_names: Vec<String> = read
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            version_names.sort();

            for version_name in &version_names {
                let version_path = versions_dir.join(version_name);

                // 2a. 版本目录下固定子目录
                for (sub, display_name, category) in VERSION_SCAN_DIRS {
                    let dir = version_path.join(sub);
                    let full_display_name = format!("{} - {}", display_name, version_name);
                    if let Some(item) = fs::scan_directory(&dir, &full_display_name, category) {
                        total_size += item.size;
                        total_files += item.file_count;
                        items.push(item);
                    }
                }

                // 2b. natives 目录（命名约定：<ver>-natives）
                let natives_dir = version_path.join(format!("{}-natives", version_name));
                let natives_display_name = format!("原生库 - {}", version_name);
                if let Some(item) =
                    fs::scan_directory(&natives_dir, &natives_display_name, "可清理")
                {
                    total_size += item.size;
                    total_files += item.file_count;
                    items.push(item);
                }
            }
        }
    }

    let result = CleanupScanResult {
        items,
        total_size,
        total_files,
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 执行清理：删除指定路径下的所有文件（含空目录）
///
/// 安全检查：每个 path 必须在 `build_allowed_parents` 返回的允许目录内，
/// 拒绝路径遍历。
pub async fn execute(params: CleanupExecuteParams) -> Result<serde_json::Value, String> {
    let game_dir = {
        let config = crate::config::load_config()
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