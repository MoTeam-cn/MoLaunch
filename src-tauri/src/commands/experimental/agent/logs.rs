//! Agent 日志读取工具：游戏日志（latest.log）与启动器日志

use crate::commands::experimental::agent::AgentContext;
use crate::commands::tools::crash_analyzer::locate_keyword_context;
use crate::utils::format::{read_line_range, truncate_chars};
use crate::utils::fs::{read_tail, tail_lines};

use super::tools::effective_dir;

/// 读取游戏日志（默认末尾 N 行）
pub(super) fn read_game_logs(
    version_id: &str,
    ctx: &AgentContext,
    lines: usize,
) -> Result<String, String> {
    read_game_logs_ex(version_id, ctx, lines, None, None, false, None)
}

/// 读取游戏日志：末尾 N 行 / startLine~endLine 行范围 / localAnalyze 本地预检 / keyword 关键词搜索
pub(super) fn read_game_logs_ex(
    version_id: &str,
    ctx: &AgentContext,
    lines: usize,
    start: Option<i64>,
    end: Option<i64>,
    local_analyze: bool,
    keyword: Option<String>,
) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id);
    let path = dir.join("logs").join("latest.log");
    if !path.exists() {
        return Ok(format!(
            "（未找到版本 {} 的 latest.log，可能尚未启动过游戏或已清理日志）",
            version_id
        ));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取日志失败: {}", e))?;

    // 本地预检：规则引擎初检，返回问题范围摘要（不返回全文，省 token）
    if local_analyze {
        return Ok(local_precheck_summary(version_id, &content, &path));
    }

    // 关键词搜索：定位首个命中行，返回其前后各 15 行上下文（带行号）
    if let Some(kw) = keyword.as_deref().filter(|k| !k.trim().is_empty()) {
        let (hit_line, window) = locate_keyword_context(&content, kw, 15);
        return match hit_line {
            Some(line_no) => Ok(format!(
                "【版本 {} 的 logs/latest.log 关键词“{}”命中第 {} 行（前后各 15 行上下文）】\n{}",
                version_id, kw, line_no, window
            )),
            None => Ok(format!(
                "（在版本 {} 的 logs/latest.log 中未找到关键词“{}”）",
                version_id, kw
            )),
        };
    }

    // 指定行范围读取
    if let (Some(s), Some(e)) = (start, end) {
        let snippet = read_line_range(&content, s, e, 4000)?;
        return Ok(format!(
            "【版本 {} 的 logs/latest.log 第 {}~{} 行】\n{}",
            version_id, s, e, snippet
        ));
    }

    let tail = read_tail(&path, lines, 6000)?;
    Ok(format!(
        "【版本 {} 的 logs/latest.log 末尾 {} 行】\n{}",
        version_id, lines, tail
    ))
}

/// 读取游戏日志的指定行段（日志分析页 / AI 工具复用入口）
pub(super) fn read_log_lines(
    version_id: &str,
    ctx: &AgentContext,
    start: i64,
    end: i64,
) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id);
    let path = dir.join("logs").join("latest.log");
    if !path.exists() {
        return Ok(format!(
            "（未找到版本 {} 的 latest.log，可能尚未启动过游戏或已清理日志）",
            version_id
        ));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取日志失败: {}", e))?;
    let snippet = read_line_range(&content, start, end, 6000)?;
    Ok(format!(
        "【版本 {} 的 logs/latest.log 第 {}~{} 行】\n{}",
        version_id, start, end, snippet
    ))
}

/// 关键词搜索游戏日志：定位首个命中行，返回其前后各 15 行上下文（带行号）
pub(super) fn search_log_keyword(
    version_id: &str,
    ctx: &AgentContext,
    keyword: &str,
) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id);
    let path = dir.join("logs").join("latest.log");
    if !path.exists() {
        return Ok(format!(
            "（未找到版本 {} 的 latest.log，可能尚未启动过游戏或已清理日志）",
            version_id
        ));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取日志失败: {}", e))?;
    let (hit_line, window) = locate_keyword_context(&content, keyword, 15);
    match hit_line {
        Some(line_no) => Ok(format!(
            "【版本 {} 的 logs/latest.log 关键词“{}”命中第 {} 行（前后各 15 行上下文）】\n{}",
            version_id, keyword, line_no, window
        )),
        None => Ok(format!(
            "（在版本 {} 的 logs/latest.log 中未找到关键词“{}”）",
            version_id, keyword
        )),
    }
}

/// 本地预检摘要：用规则引擎初检日志文本，返回收敛后的问题范围（省 token）
pub(super) fn local_precheck_summary(
    version_id: &str,
    content: &str,
    path: &std::path::Path,
) -> String {
    let items = crate::commands::tools::crash_analyzer::analyze_log_text(content);
    if items.is_empty() {
        return format!(
            "【版本 {} 的 {} 本地预检结果】\n未识别到已知崩溃模式（日志正常或错误特征未命中规则）。若需要，可读取原文进一步排查。",
            version_id,
            path.file_name().unwrap_or_default().to_string_lossy()
        );
    }
    let mut text = format!(
        "【版本 {} 的 {} 本地预检结果】检测到 {} 个可能问题：\n",
        version_id,
        path.file_name().unwrap_or_default().to_string_lossy(),
        items.len()
    );
    for (i, it) in items.iter().enumerate() {
        text.push_str(&format!(
            "{}. [{}][{}] {}\n",
            i + 1,
            it.category,
            it.severity,
            it.title
        ));
        if !it.detail.is_empty() {
            text.push_str(&format!("   关键行: {}\n", it.detail));
        }
        if !it.suggestion.is_empty() {
            text.push_str(&format!("   建议: {}\n", it.suggestion));
        }
    }
    text.push_str("\n若需要更多上下文，可调用 read_log_lines 读取日志对应行段。");
    text
}

/// 读取启动器日志末尾 N 行（复用日志脱敏，避免敏感信息传给模型）
pub(super) fn read_launcher_logs(lines: usize) -> Result<String, String> {
    let storage = crate::storage::Storage::instance();
    let logs_dir = storage.logs_dir();
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&logs_dir)
        .map_err(|e| format!("读取启动器日志目录失败: {}", e))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "log").unwrap_or(false))
        .collect();
    if files.is_empty() {
        return Ok("（未找到启动器日志文件）".to_string());
    }
    files.sort_by_key(|p| p.metadata().and_then(|m| m.modified()).ok());
    let newest = files.last().expect("files 非空");
    let raw = std::fs::read_to_string(newest).map_err(|e| format!("读取启动器日志失败: {}", e))?;
    // 先取末尾 N 行再截断：`truncate_chars` 保留行首（最新内容），避免只拿到日志中间一段
    let sanitized = crate::logger::sanitize_sensitive_info(&raw);
    let trimmed = tail_lines(&sanitized, lines);
    let tail = truncate_chars(&trimmed, 6000);
    Ok(format!(
        "【启动器日志 {} 末尾 {} 行】\n{}",
        newest.file_name().unwrap_or_default().to_string_lossy(),
        lines,
        tail
    ))
}
