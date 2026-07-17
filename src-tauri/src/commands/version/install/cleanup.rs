//! 安装失败时的清理逻辑
//!
//! 原位于 `install_merged` 末尾 else 分支：删除已下载的原版目录和加载器创建的目录。

use crate::{log_error, log_info};
use std::path::Path;

/// 清理失败的安装：删除原版目录 + 加载器创建的目录
pub(crate) fn cleanup_failed_install(
    game_dir: &Path,
    mc_version: &str,
    fabric_version: Option<&str>,
) {
    let versions_dir = game_dir.join("versions");

    // 删除原版目录
    let mc_version_dir = versions_dir.join(mc_version);
    if mc_version_dir.exists() {
        match std::fs::remove_dir_all(&mc_version_dir) {
            Ok(_) => log_info!("[Merged] 已清理原版目录: {}", mc_version_dir.display()),
            Err(e) => log_error!("[Merged] 清理原版目录失败: {}", e),
        }
    }

    // 删除加载器创建的目录（如 1.20.1-forge-47.4.20）
    // 注意：fabric 目录命名为 `fabric-{fabric_version}-{mc_version}`，
    // 之前用 `fabric-` 前缀过宽（会误删任意含 "fabric-" 的目录），
    // 改为仅在知道 fabric_version 时构造精确匹配。
    let mut loader_patterns = vec![
        format!("{}-forge-", mc_version),
        format!("{}-neoforge-", mc_version),
        format!("{}-LiteLoader", mc_version),
    ];
    if let Some(fv) = fabric_version {
        loader_patterns.push(format!("fabric-{}-{}", fv, mc_version));
    }

    if let Ok(entries) = std::fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            for pattern in &loader_patterns {
                if dir_name.contains(pattern) {
                    match std::fs::remove_dir_all(entry.path()) {
                        Ok(_) => {
                            log_info!("[Merged] 已清理加载器目录: {}", entry.path().display())
                        }
                        Err(e) => log_error!("[Merged] 清理加载器目录失败: {}", e),
                    }
                    break;
                }
            }
        }
    }
}
