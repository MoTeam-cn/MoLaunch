//! 版本验证与排序

use std::path::PathBuf;

use super::super::detect::detect_java;
use super::super::JavaRuntime;

/// 验证候选 Java 路径列表，返回成功检测的 JavaRuntime 列表
pub(super) fn verify_java_candidates(candidates: &[PathBuf]) -> Vec<JavaRuntime> {
    let mut java_list = Vec::new();
    for path in candidates {
        match detect_java(path) {
            Ok(java) => {
                crate::log_debug!("[Java] Valid: {} ({})", java.version, java.path_folder);
                java_list.push(java);
            }
            Err(e) => {
                crate::log_debug!("[Java] Invalid {}: {}", path.display(), e);
            }
        }
    }
    java_list
}

/// 排序：大版本优先，其次 64 位优先
pub(super) fn sort_java_list(mut java_list: Vec<JavaRuntime>) -> Vec<JavaRuntime> {
    java_list.sort_by(|a, b| {
        b.major_version
            .cmp(&a.major_version)
            .then(b.is_64bit.cmp(&a.is_64bit))
    });
    java_list
}