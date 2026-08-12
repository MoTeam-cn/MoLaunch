//! 文件系统操作（阶段 4：runtime 目录定位 + 路径穿越校验 + 可执行文件查找）

use std::path::{Path, PathBuf};

use super::types::{RuntimeFile, RuntimeManifest};

/// 从 manifest 中过滤出需要下载的文件（有 downloads.raw 的）
pub fn filter_downloadable_files(manifest: &RuntimeManifest) -> Vec<(String, RuntimeFile)> {
    manifest
        .files
        .iter()
        .filter(|(_, f)| f.downloads.is_some())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// 计算待下载文件的总字节数
pub fn total_bytes(files_to_download: &[(String, RuntimeFile)]) -> u64 {
    files_to_download
        .iter()
        .map(|(_, f)| f.downloads.as_ref().unwrap().raw.size)
        .sum()
}

/// 获取 Java Runtime 存储目录（{APPDATA}\.minecraft\runtime\{component}\）
///
/// 存到官启默认 .minecraft 目录下，跨游戏目录共享，不随 game_dir 删除而丢失。
pub fn get_runtime_dir(component: &str) -> Result<PathBuf, String> {
    crate::utils::cache_app::runtime_dir(component)
}

/// 在 runtime 目录中查找 java 可执行文件
///
/// 跨平台支持：
/// - Windows: 查找 `java.exe`，候选子目录 `windows-x64`
/// - macOS:   查找 `java`，候选子目录 `mac-os`（Mojang 官方 manifest 命名）
/// - Linux:   查找 `java`，候选子目录 `linux`
///
/// 递归查找兜底所有平台通用，覆盖 manifest 路径变化。
pub fn find_java_exe(runtime_dir: &Path) -> Result<PathBuf, String> {
    // 平台相关的可执行文件名与子目录名
    let exe_name = if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    };
    let platform_subdir = if cfg!(target_os = "windows") {
        "windows-x64"
    } else if cfg!(target_os = "macos") {
        "mac-os"
    } else {
        "linux"
    };

    // 常见路径：
    //   runtime/{component}/bin/{exe_name}
    //   runtime/{component}/{platform_subdir}/{component}/bin/{exe_name}
    let candidates = [
        runtime_dir.join("bin").join(exe_name),
        runtime_dir
            .join(platform_subdir)
            .join(runtime_dir.file_name().unwrap_or_default())
            .join("bin")
            .join(exe_name),
    ];

    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }

    // 递归查找可执行文件（兜底所有平台，覆盖 manifest 路径变化）
    fn find_recursive(dir: &Path, exe_name: &str) -> Option<PathBuf> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir).ok()?.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(found) = find_recursive(&path, exe_name) {
                        return Some(found);
                    }
                } else if path.file_name().map(|n| n == exe_name).unwrap_or(false) {
                    return Some(path);
                }
            }
        }
        None
    }

    find_recursive(runtime_dir, exe_name)
        .ok_or_else(|| format!("在 {} 中未找到 {}", runtime_dir.display(), exe_name))
}

/// 校验路径穿越：manifest 来自远程，必须确保最终路径仍在 runtime_dir 内
///
/// 1. 拒绝显式包含 ".." 的路径
/// 2. canonicalize 校验最终路径父目录仍位于 runtime_dir 内
pub fn validate_path_traversal(
    path_str: &str,
    local_path: &Path,
    runtime_dir: &Path,
) -> Result<(), String> {
    if !crate::utils::path::is_safe_relative_path(path_str) {
        return Err(format!(
            "Path traversal detected in manifest path: {}",
            path_str
        ));
    }
    let canonical_base = runtime_dir
        .canonicalize()
        .unwrap_or_else(|_| runtime_dir.to_path_buf());
    if let Some(parent) = local_path.parent() {
        if let Ok(canonical_parent) = parent.canonicalize() {
            if !canonical_parent.starts_with(&canonical_base) {
                return Err(format!(
                    "Path traversal detected: {} is outside runtime dir",
                    path_str
                ));
            }
        }
    }
    Ok(())
}
