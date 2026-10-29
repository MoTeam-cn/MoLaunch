//! 文件系统操作（阶段 4：runtime 目录定位 + 路径穿越校验 + 可执行文件查找）

use std::path::{Path, PathBuf};

use super::types::{RuntimeFile, RuntimeManifest};

use crate::log_warn;

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

/// 安装失败后清理本次新建的 runtime 目录
///
/// `dir_existed` 是**安装开始前**目录是否已存在。这个目录
/// （`{APPDATA}/.minecraft/runtime/{component}`）与官方启动器共用：
/// 整删会把别的启动器装好、正在用的 Java 一起毁掉，而这跟「我们这次装失败」
/// 没有半点关系。所以只有本次新建的目录才能删。
pub fn cleanup_failed_install(runtime_dir: &Path, dir_existed: bool) {
    if dir_existed {
        // 保留目录，但要把「安装中」标记摘掉：目录里的文件都过了 SHA1，
        // 混着旧文件通常仍然能用，而挂着标记会让扫描器永久忽略它
        clear_install_marker(runtime_dir);
        log_warn!(
            "[JavaDownload] 安装失败但 runtime 目录原本就存在，保留不清理: {}",
            runtime_dir.display()
        );
        return;
    }
    match std::fs::remove_dir_all(runtime_dir) {
        Ok(()) => {
            log_warn!(
                "[JavaDownload] 安装失败，已清理本次新建的 runtime 目录: {}",
                runtime_dir.display()
            );
        }
        Err(e) => {
            log_warn!(
                "[JavaDownload] 清理 runtime 目录失败: {} ({})",
                runtime_dir.display(),
                e
            );
        }
    }
}

#[cfg(test)]
#[path = "files_test.rs"]
mod files_test;

/// 安装中标记文件名（放在 runtime 目录根部）
pub const INSTALL_MARKER: &str = ".molaunch-installing";

/// 标记「这个 runtime 正在安装」
///
/// 装到一半的 `java.exe` 往往已经能执行，但模块不全。扫描器把它当成可用 Java
/// 收进来，用户一选就在启动时崩——所以安装期间与「进程中途被杀」之后，
/// 目录里都要留着这个标记，让扫描跳过。
pub fn mark_install_started(runtime_dir: &Path) {
    if let Err(e) = std::fs::create_dir_all(runtime_dir) {
        log_warn!(
            "[JavaDownload] 创建 runtime 目录失败: {} ({})",
            runtime_dir.display(),
            e
        );
        return;
    }
    if let Err(e) = std::fs::write(
        runtime_dir.join(INSTALL_MARKER),
        std::process::id().to_string(),
    ) {
        log_warn!(
            "[JavaDownload] 写安装标记失败: {} ({})",
            runtime_dir.display(),
            e
        );
    }
}

/// 摘掉安装中标记（下载与验证都通过才算装好）
pub fn clear_install_marker(runtime_dir: &Path) {
    let _ = std::fs::remove_file(runtime_dir.join(INSTALL_MARKER));
}

/// 没有安装中标记才算装好
pub fn is_install_complete(runtime_dir: &Path) -> bool {
    !runtime_dir.join(INSTALL_MARKER).exists()
}
