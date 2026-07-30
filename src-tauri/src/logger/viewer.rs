//! 日志文件查看 API
//!
//! 供 system_manager dispatcher 调用，提供日志文件路径查询、列表、读取功能。
//! 薄包装层将 PathBuf / Vec<String> / anyhow::Result<String> 转为序列化友好类型。

use crate::storage::Storage;
use std::path::PathBuf;

use super::sanitize::sanitize_sensitive_info;

/// 获取日志文件路径
pub fn get_log_path_inner() -> PathBuf {
    let storage = Storage::instance();
    let logs_dir = storage.logs_dir();
    let now = chrono::Local::now();
    let filename = format!("molaunch_{}.log", now.format("%Y-%m-%d"));
    logs_dir.join(filename)
}

/// 获取所有日志文件
pub fn list_log_files_inner() -> Vec<String> {
    let storage = Storage::instance();
    let logs_dir = storage.logs_dir();

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".log") {
                files.push(name);
            }
        }
    }
    files.sort();
    files.reverse(); // 最新的在前
    files
}

/// 读取日志文件内容
pub fn read_log_file_inner(filename: &str) -> anyhow::Result<String> {
    let storage = Storage::instance();
    let path = storage.logs_dir().join(filename);
    Ok(std::fs::read_to_string(&path)?)
}

/// 获取今日日志文件完整路径（字符串形式）
pub fn get_log_path() -> String {
    get_log_path_inner().to_string_lossy().to_string()
}

/// 获取所有日志文件名列表（最新的在前）
pub fn list_log_files() -> Vec<String> {
    list_log_files_inner()
}

/// 读取指定日志文件内容
///
/// `filename` 仅允许 `.log` 后缀，且不得包含路径分隔符（防止路径遍历）。
/// 安全修复：返回前对内容进行脱敏，避免前端日志查看器显示 token 等敏感信息
pub fn read_log_file(filename: String) -> Result<String, String> {
    if filename.is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
        || !filename.ends_with(".log")
    {
        return Err(format!("非法日志文件名: {}", filename));
    }
    let content = read_log_file_inner(&filename).map_err(|e| format!("读取日志文件失败: {}", e))?;
    Ok(sanitize_sensitive_info(&content))
}
