//! 工具函数

use super::super::types::{CrashCategory, CrashInfo};
use std::path::Path;

/// 从日志文本中提取 "Caught exception from {ModName}" 格式的 Mod 名称
pub(super) fn extract_mod_from_keyword(text: &str, prefix: &str) -> Option<String> {
    let text_l = text.to_lowercase();
    let prefix_l = prefix.to_lowercase();
    if let Some(pos) = text_l.find(&prefix_l) {
        let rest = &text[pos + prefix_l.len()..];
        // 取到行尾或下一个空格
        let end = rest.find(|c: char| c == '\n' || c == '\r').unwrap_or(rest.len());
        let mod_name = rest[..end].trim();
        if !mod_name.is_empty() {
            return Some(mod_name.to_string());
        }
    }
    None
}

/// 截取头 N 行 + 尾 M 行
pub(super) fn truncate_head_tail(content: &str, head: usize, tail: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= head + tail {
        return content.to_string();
    }
    let mut result = String::new();
    for line in &lines[..head] {
        result.push_str(line);
        result.push('\n');
    }
    result.push_str("...（省略中间部分）...\n");
    for line in &lines[lines.len() - tail..] {
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// 构造 CrashInfo 的快捷函数
pub(super) fn make_crash_info(
    reason: &str,
    category: CrashCategory,
    suggestion: &str,
    error_lines: &[String],
    crash_report_path: Option<&Path>,
) -> CrashInfo {
    CrashInfo {
        reason: reason.to_string(),
        category,
        log_lines: error_lines.to_vec(),
        suggestion: suggestion.to_string(),
        problematic_mod: None,
        crash_report_path: crash_report_path.map(|p| p.to_string_lossy().to_string()),
        log_tail: Vec::new(),
    }
}
