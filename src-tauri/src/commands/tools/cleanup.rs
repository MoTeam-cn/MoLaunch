//! 清理游戏垃圾文件（扫描 `.minecraft` 下可清理内容，分三类）
//! 根目录固定子目录：logs/crash-reports/.mixin.out/assets/cache/.fabric/remapCache/screenshots；
//! 版本目录下子目录（版本隔离模式）：logs/crash-reports/.mixin.out/.fabric/processedMods/
//! remappedJars；原生库提取目录 `<ver>-natives`（非 natives）。
//! `execute` 删除时严格路径安全检查，仅删扫描阶段发现的目录，避免路径遍历攻击。

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
        if let Some(item) = scan_directory(&dir, display_name, category) {
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
                    if let Some(item) = scan_directory(&dir, &full_display_name, category) {
                        total_size += item.size;
                        total_files += item.file_count;
                        items.push(item);
                    }
                }

                // 2b. natives 目录（命名约定：<ver>-natives）
                let natives_dir = version_path.join(format!("{}-natives", version_name));
                let natives_display_name = format!("原生库 - {}", version_name);
                if let Some(item) = scan_directory(&natives_dir, &natives_display_name, "可清理")
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

        if !is_path_safe(&path, &allowed_parents) {
            failed.push(CleanupFailedItem {
                path: path_str.clone(),
                error: "路径不在允许的清理目录内".to_string(),
            });
            continue;
        }

        if !path.exists() {
            continue;
        }

        match remove_dir_recursive(&path) {
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

/// 递归扫描目录：累计所有文件大小与文件数
///
/// 不存在返回 None；存在但为空目录返回 size=0 / file_count=0 的 CleanupItem。
fn scan_directory(dir: &Path, display_name: &str, category: &str) -> Option<CleanupItem> {
    if !dir.exists() {
        return None;
    }

    let mut size: u64 = 0;
    let mut file_count: u64 = 0;

    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        let read = match std::fs::read_dir(&p) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in read.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.is_file() {
                if let Ok(meta) = entry.metadata() {
                    size += meta.len();
                }
                file_count += 1;
            }
        }
    }

    Some(CleanupItem {
        path: dir.to_string_lossy().to_string(),
        display_name: display_name.to_string(),
        category: category.to_string(),
        size,
        file_count,
    })
}

/// 路径安全检查：path 必须等于 allowed_parents 中的某个，或位于其下
///
/// 防止路径遍历攻击（如传入 `..` 跳出 .minecraft）。
fn is_path_safe(path: &Path, allowed_parents: &[PathBuf]) -> bool {
    let canonical = match path.canonicalize() {
        Ok(c) => c,
        Err(_) => return false,
    };
    for parent in allowed_parents {
        let parent_canonical = match parent.canonicalize() {
            Ok(c) => c,
            Err(_) => continue,
        };
        if canonical == parent_canonical || canonical.starts_with(&parent_canonical) {
            return true;
        }
    }
    false
}

/// 递归删除目录下的所有文件与子目录，最后删除目录本身
///
/// 返回 (累计字节数, 文件数)。
fn remove_dir_recursive(root: &Path) -> Result<(u64, u64), String> {
    let mut size: u64 = 0;
    let mut files: u64 = 0;

    let mut dirs_to_walk: Vec<PathBuf> = vec![root.to_path_buf()];
    let mut all_files: Vec<PathBuf> = Vec::new();
    let mut all_dirs: Vec<PathBuf> = Vec::new();

    while let Some(d) = dirs_to_walk.pop() {
        let read = std::fs::read_dir(&d).map_err(|e| format!("读取目录失败: {}", e))?;
        for entry in read.flatten() {
            let p = entry.path();
            if p.is_dir() {
                dirs_to_walk.push(p.clone());
                all_dirs.push(p);
            } else if p.is_file() {
                if let Ok(meta) = entry.metadata() {
                    size += meta.len();
                }
                files += 1;
                all_files.push(p);
            }
        }
    }

    // 先删文件
    for f in &all_files {
        if let Err(e) = std::fs::remove_file(f) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(format!("删除文件失败: {}", e));
            }
        }
    }

    // 再删目录（自底向上：按深度倒序，子目录优先）
    all_dirs.sort_by_key(|p| p.components().count());
    all_dirs.reverse();
    for d in &all_dirs {
        if let Err(e) = std::fs::remove_dir(d) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(format!("删除目录失败: {}", e));
            }
        }
    }

    // 最后删除 root 本身
    if root.exists() {
        std::fs::remove_dir(root).map_err(|e| format!("删除根目录失败: {}", e))?;
    }

    Ok((size, files))
}
