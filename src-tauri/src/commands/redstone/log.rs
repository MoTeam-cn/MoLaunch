//! 红石内核日志文件读取：列表 + 尾部读取
//!
//! hongshi 内核按官方接入文档约定在工作目录（`<temp>/MoLaunch/hongshi/`）
//! 下写 `logs/<YYYY-MM-DD>.log`，与控制台同步输出，级别 INFO/WARN/ERROR。

use serde::Serialize;
use std::time::UNIX_EPOCH;

/// 日志文件信息（redstone_log_files 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedstoneLogFileInfo {
    /// 日志文件名（如 `2026-08-17.log`）
    pub file_name: String,
    /// 文件大小（字节）
    pub size_bytes: u64,
    /// 最后修改时间（Unix 毫秒）
    pub modified_at: u64,
}

/// 日志文件内容（redstone_read_log 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedstoneLogContent {
    /// 日志行（尾部截取，每行原始文本）
    pub lines: Vec<String>,
    /// 是否还有更早的历史行未展示
    pub has_more: bool,
}

/// 按修改时间倒序列出日志文件
pub fn list_log_files() -> Result<Vec<RedstoneLogFileInfo>, String> {
    let logs_dir = crate::resources::hongshi_logs_dir();
    if !logs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&logs_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        files.push(RedstoneLogFileInfo {
            file_name,
            size_bytes: metadata.len(),
            modified_at,
        });
    }
    files.sort_by_key(|f| std::cmp::Reverse(f.modified_at));
    Ok(files)
}

/// 读取指定日志文件内容（尾部 max_lines 行，规则与 frp read_log_file 一致）
///
/// 文件名须为 `logs/` 目录下的普通文件名（禁止路径穿越）；
/// 文件不存在或为空时返回空内容。
pub fn read_log_file(
    file_name: String,
    max_lines: Option<usize>,
) -> Result<RedstoneLogContent, String> {
    let max = max_lines.unwrap_or(500);

    // 防路径穿越：文件名只允许日期文件形式 `YYYY-MM-DD.log`
    let name = file_name.trim();
    let is_valid_name = !name.is_empty()
        && name.len() <= 16
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        && !name.contains("..")
        && name.ends_with(".log");
    if !is_valid_name {
        return Err("日志文件名非法".to_string());
    }

    let path = crate::resources::hongshi_logs_dir().join(name);
    if !path.is_file() {
        return Ok(RedstoneLogContent {
            lines: Vec::new(),
            has_more: false,
        });
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let all_lines: Vec<&str> = content.lines().collect();
    let (lines, has_more) = if all_lines.len() > max {
        let start = all_lines.len() - max;
        (
            all_lines[start..].iter().map(|s| s.to_string()).collect(),
            true,
        )
    } else {
        (all_lines.iter().map(|s| s.to_string()).collect(), false)
    };
    Ok(RedstoneLogContent { lines, has_more })
}
