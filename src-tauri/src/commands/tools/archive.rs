//! 存档管理（备份/恢复/导出）
//!
//! - `list`：列出 `{game_dir}/saves/` 下所有子文件夹（存档），按名称排序
//! - `backup`：将存档打包为 zip（可选排除 `playerdata/` 作为分享包）
//! - `restore`：从 zip 解压恢复存档到 `saves/{world_name}/`

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;
use crate::state::resolve_game_dir;

use super::types::{
    ArchiveBackupParams, ArchiveBackupResult, ArchiveItem, ArchiveListResult, ArchiveRestoreParams,
    ArchiveRestoreResult,
};

/// 列出 saves 目录下所有存档（子文件夹），按名称排序
pub async fn list(state: &AppState) -> Result<serde_json::Value, String> {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    let saves_dir = game_dir.join("saves");

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
    let (items, total_size) = tokio::task::spawn_blocking(
        move || -> (Vec<ArchiveItem>, u64) {
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
        },
    )
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

/// 将存档打包为 zip（可选排除玩家数据用于分享）
///
/// 失败返回 `success=false`（不返回 Err），由前端展示失败提示
pub async fn backup(
    state: &AppState,
    params: ArchiveBackupParams,
) -> Result<serde_json::Value, String> {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    let saves_dir = game_dir.join("saves");

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
            log_warn!(
                "[Archive] 输出目录不存在: {}",
                parent.display()
            );
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
            log_warn!(
                "[Archive] 存档目录不存在: {} ({})",
                source.display(),
                e
            );
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

/// 从 zip 解压恢复存档到 `saves/{world_name}/`
///
/// world_name 为空时用 zip 文件名（去 .zip 后缀）；目标目录已存在则返回失败
pub async fn restore(
    state: &AppState,
    params: ArchiveRestoreParams,
) -> Result<serde_json::Value, String> {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    let saves_dir = game_dir.join("saves");

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
    if world_name.is_empty() || world_name.contains("..") {
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

// ===== 辅助函数 =====

/// 递归计算目录总字节数
fn dir_total_size(dir: &Path) -> u64 {
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

/// 递归收集目录下所有文件路径
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_file() {
            out.push(path);
        } else if path.is_dir() {
            collect_files(&path, out);
        }
    }
}

/// 将目录内容打包为 zip（保留相对路径，zip 内使用正斜杠）
///
/// `exclude_top_dirs` 为 src_dir 顶层需跳过的子目录名（如 `["playerdata"]`）
fn zip_directory(src_dir: &Path, output_zip: &Path, exclude_top_dirs: &[&str]) -> Result<(), String> {
    let file =
        File::create(output_zip).map_err(|e| format!("创建 zip 失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 收集文件：顶层命中 exclude_top_dirs 的子目录整体跳过
    let mut entries: Vec<PathBuf> = Vec::new();
    let read = std::fs::read_dir(src_dir)
        .map_err(|e| format!("读取源目录失败: {}", e))?;
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_file() {
            entries.push(path);
        } else if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if exclude_top_dirs.contains(&name) {
                    continue;
                }
            }
            collect_files(&path, &mut entries);
        }
    }

    for file_path in entries {
        let rel = file_path
            .strip_prefix(src_dir)
            .map_err(|e| format!("路径前缀剥离失败: {}", e))?;
        let rel_str = rel.to_str().ok_or("路径包含非 UTF-8 字符")?;
        let zip_name = rel_str.replace('\\', "/");
        zip.start_file(&zip_name, options)
            .map_err(|e| format!("写入 zip 条目失败: {}", e))?;
        let mut f =
            File::open(&file_path).map_err(|e| format!("打开源文件失败: {}", e))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .map_err(|e| format!("读取源文件失败: {}", e))?;
        zip.write_all(&buf)
            .map_err(|e| format!("写入 zip 内容失败: {}", e))?;
    }

    zip.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}

/// 解压 zip 到目录
fn unzip_to_dir(src_zip: &Path, output_dir: &Path) -> Result<(), String> {
    let file = File::open(src_zip).map_err(|e| format!("打开 zip 失败: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("读取 zip 失败: {}", e))?;
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;
    archive
        .extract(output_dir)
        .map_err(|e| format!("解压失败: {}", e))?;
    Ok(())
}

/// 将路径转为字符串（UTF-8，丢失非 UTF-8 字符）
fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap_or("").to_string()
}
