//! 共享辅助：目录大小统计、文件递归收集、zip 打包/解压、路径转字符串
//!
//! 各 archive 子模块通过 `super::helpers::*` 复用，避免重复实现。

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::minecraft::isolation::{get_effective_game_dir, IsolationMode};
use crate::state::{resolve_game_dir, AppState};

/// 解析 saves 目录（同 screenshot::resolve_shots_dir 的语义）
pub(crate) async fn resolve_saves_dir(state: &AppState, version_id: Option<&str>) -> PathBuf {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    match version_id {
        None => game_dir.join("saves"),
        Some(vid) => {
            let global_mode = state.config.lock().await.isolation_mode;
            let isolation_mode =
                crate::commands::version::list::resolve_isolation_mode(&game_dir, vid, global_mode);
            let version_type =
                crate::commands::version::list::detect_version_type_from_dir(&game_dir, vid);
            let mode = IsolationMode::from_u32(isolation_mode);
            let effective_dir = get_effective_game_dir(&game_dir, vid, mode, version_type);
            effective_dir.join("saves")
        }
    }
}

/// 递归计算目录总字节数
pub(super) fn dir_total_size(dir: &Path) -> u64 {
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
pub(super) fn zip_directory(
    src_dir: &Path,
    output_zip: &Path,
    exclude_top_dirs: &[&str],
) -> Result<(), String> {
    let file = File::create(output_zip).map_err(|e| format!("创建 zip 失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 收集文件：顶层命中 exclude_top_dirs 的子目录整体跳过
    let mut entries: Vec<PathBuf> = Vec::new();
    let read = std::fs::read_dir(src_dir).map_err(|e| format!("读取源目录失败: {}", e))?;
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
        let mut f = File::open(&file_path).map_err(|e| format!("打开源文件失败: {}", e))?;
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
pub(super) fn unzip_to_dir(src_zip: &Path, output_dir: &Path) -> Result<(), String> {
    let file = File::open(src_zip).map_err(|e| format!("打开 zip 失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("读取 zip 失败: {}", e))?;
    std::fs::create_dir_all(output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;
    archive
        .extract(output_dir)
        .map_err(|e| format!("解压失败: {}", e))?;
    Ok(())
}

/// 将路径转为字符串（UTF-8，丢失非 UTF-8 字符）
pub(super) fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap_or("").to_string()
}
