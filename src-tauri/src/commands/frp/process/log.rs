//! frpc 日志文件读取：列表 + 尾部读取 + 跨隧道合并

use std::time::UNIX_EPOCH;

use crate::commands::frp::{frp_logs_dir, LogFileContent, LogFileInfo};
use crate::log_info;

/// 列出所有日志文件
///
/// 扫描 `<base_dir>/frp/logs/` 目录，返回 `.log` 文件列表，
/// 按修改时间倒序排列。
pub async fn list_log_files() -> Result<Vec<LogFileInfo>, String> {
    let logs_dir = frp_logs_dir();
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
        let tunnel_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        files.push(LogFileInfo {
            tunnel_id,
            size_bytes: metadata.len(),
            modified_at: modified,
        });
    }
    // 按修改时间倒序
    files.sort_by_key(|b| std::cmp::Reverse(b.modified_at));
    Ok(files)
}

/// 清空指定隧道的日志文件内容（保留文件，不清除磁盘路径）
///
/// `tunnel_id` 为空时清空所有日志文件。文件不存在时静默成功。
/// 与前端「清空当前显示」的区别：本函数直接删除文件内容，
/// 重启隧道 / 刷新后会真正看到日志已清空。
pub async fn clear_log_file(tunnel_id: String) -> Result<(), String> {
    let logs_dir = frp_logs_dir();
    if !logs_dir.exists() {
        return Ok(());
    }

    if tunnel_id.trim().is_empty() {
        // 清空所有日志文件
        let mut cleared = 0usize;
        for entry in std::fs::read_dir(&logs_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") {
                continue;
            }
            std::fs::write(&path, "")
                .map_err(|e| format!("清空日志失败 {}: {}", path.display(), e))?;
            cleared += 1;
        }
        log_info!("[Frp] 已清空 {} 个日志文件", cleared);
        return Ok(());
    }

    let path = logs_dir.join(format!("{}.log", tunnel_id));
    if path.exists() {
        std::fs::write(&path, "").map_err(|e| format!("清空日志失败 {}: {}", path.display(), e))?;
        log_info!("[Frp] 已清空日志文件: {}", path.display());
    }
    Ok(())
}

/// 读取日志文件内容（尾部 maxLines 行）
///
/// `tunnel_id` 为空时合并所有隧道日志（按行内时间戳排序，限 max 行）。
/// `max_lines` 为 None 时默认返回最后 500 行。
/// 文件不存在时返回空内容。
pub async fn read_log_file(
    tunnel_id: String,
    max_lines: Option<usize>,
) -> Result<LogFileContent, String> {
    let max = max_lines.unwrap_or(500);

    // 空隧道 ID：合并所有日志文件
    if tunnel_id.trim().is_empty() {
        return read_all_logs(max).await;
    }

    let path = frp_logs_dir().join(format!("{}.log", tunnel_id));
    if !path.exists() {
        return Ok(LogFileContent {
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
    Ok(LogFileContent { lines, has_more })
}

/// 合并所有隧道日志文件，按行内时间戳排序，返回最后 max 行
///
/// 日志行格式：`[HH:MM:SS.ms] [LEVEL] message` 或 `[YYYY-MM-DD HH:MM:SS.ms] [LEVEL] ...`。
/// 无时间戳的行按文件顺序排在一起。
/// 跨文件合并后总行数超过 max 时截取尾部 max 行，并标记 has_more=true。
async fn read_all_logs(max: usize) -> Result<LogFileContent, String> {
    let logs_dir = frp_logs_dir();
    if !logs_dir.exists() {
        return Ok(LogFileContent {
            lines: Vec::new(),
            has_more: false,
        });
    }

    // 收集 (时间戳 sortable key, tunnel_id, 行内容)
    // 时间戳提取：行首 `[...]` 内的内容；若无法提取则用空字符串（排最前）
    let mut entries: Vec<(String, String, String)> = Vec::new();

    for entry in std::fs::read_dir(&logs_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let tid = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let ts = extract_log_timestamp(line);
            entries.push((ts, tid.clone(), line.to_string()));
        }
    }

    if entries.is_empty() {
        return Ok(LogFileContent {
            lines: Vec::new(),
            has_more: false,
        });
    }

    // 按时间戳排序（相同时间戳保持稳定排序）
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let total = entries.len();
    let has_more = total > max;
    let start = if has_more { total - max } else { 0 };
    let lines: Vec<String> = entries[start..]
        .iter()
        .map(|(ts, tid, line)| {
            // 所有行都已有时间戳前缀（capture_stream 写入时统一加），直接返回
            let _ = (ts, tid);
            line.clone()
        })
        .collect();

    Ok(LogFileContent { lines, has_more })
}

/// 从日志行提取时间戳排序键
///
/// 支持格式：
/// - `[HH:MM:SS.ms] [LEVEL] ...` → 当天时间，按字符串排序即可
/// - `[YYYY-MM-DD HH:MM:SS.ms] [LEVEL] ...` → 完整时间戳
/// - `2026-07-31 16:47:21.286 [I] ...` → frpc 原生格式，无方括号
///
/// 提取失败返回空字符串（排在前面）。
fn extract_log_timestamp(line: &str) -> String {
    let trimmed = line.trim_start();
    // 形式 1/2：以 [ 开头
    if trimmed.starts_with('[') {
        if let Some(end) = trimmed.find(']') {
            return trimmed[1..end].to_string();
        }
    }
    // 形式 3：frpc 原生格式 "YYYY-MM-DD HH:MM:SS.ms ..."
    // 取前 23 个字符（"YYYY-MM-DD HH:MM:SS.mmm" 长度）
    if trimmed.len() >= 23 {
        let prefix = &trimmed[..23];
        // 简单校验是否符合日期格式
        if prefix.chars().nth(4) == Some('-')
            && prefix.chars().nth(7) == Some('-')
            && prefix.chars().nth(10) == Some(' ')
            && prefix.chars().nth(13) == Some(':')
        {
            return prefix.to_string();
        }
    }
    String::new()
}
