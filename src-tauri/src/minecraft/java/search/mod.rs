//! Java 搜索模块
//!
//! 入口：平台扫描收集候选路径（platform）→ 验证并排序（version）。

mod platform;
mod version;

use std::path::PathBuf;

use super::JavaRuntime;
use platform::collect_java_candidates;
use version::{sort_java_list, verify_java_candidates};

/// 搜索系统中的Java
pub fn search_java() -> Vec<JavaRuntime> {
    search_java_with_paths(&[])
}

/// 带额外搜索路径的 Java 搜索
///
/// `extra_paths` 用于追加搜索根目录（如游戏目录、APPDATA 等），会全遍历搜索。
pub fn search_java_with_paths(extra_paths: &[PathBuf]) -> Vec<JavaRuntime> {
    crate::log_separator!("Java Search");
    crate::log_debug!("[Java] Starting Java search...");

    // 1. 收集候选路径（环境变量 / 全磁盘 / 用户目录 / 启动器目录 / runtime / 额外路径）
    let java_candidates = collect_java_candidates(extra_paths);

    crate::log_debug!(
        "[Java] Found {} candidates, verifying...",
        java_candidates.len()
    );

    // 2. 验证所有候选Java
    let java_list = verify_java_candidates(&java_candidates);

    // 3. 排序：大版本优先，其次 64 位优先
    let java_list = sort_java_list(java_list);

    crate::log_debug!(
        "[Java] Search completed, found {} valid Java installations",
        java_list.len()
    );
    crate::log_separator!("Java Search End");

    java_list
}