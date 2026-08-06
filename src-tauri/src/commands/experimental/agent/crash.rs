//! Agent 崩溃报告工具：读取与分析（crash-reports）

use crate::commands::experimental::agent::AgentContext;
use crate::commands::tools::crash_analyzer::locate_keyword_context;
use crate::utils::format::{read_line_range, truncate_chars};
use crate::utils::fs::newest_file;

use super::logs::local_precheck_summary;
use super::tools::effective_dir;

/// 读取最新崩溃报告（默认全文截断）
pub(super) fn read_crash_report(version_id: &str, ctx: &AgentContext) -> Result<String, String> {
    read_crash_report_ex(version_id, ctx, None, None, false, None)
}

/// 读取最新崩溃报告：全文（截断）/ 行范围 / localAnalyze 本地预检 / keyword 关键词搜索
pub(super) fn read_crash_report_ex(
    version_id: &str,
    ctx: &AgentContext,
    start: Option<i64>,
    end: Option<i64>,
    local_analyze: bool,
    keyword: Option<String>,
) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id).join("crash-reports");
    let Some(newest) = newest_file(&dir, Some("txt")) else {
        return Ok(format!(
            "（版本 {} 的 crash-reports 目录不存在或为空，游戏可能未崩溃过）",
            version_id
        ));
    };
    let content =
        std::fs::read_to_string(&newest).map_err(|e| format!("读取崩溃报告失败: {}", e))?;

    if local_analyze {
        return Ok(local_precheck_summary(version_id, &content, &newest));
    }

    // 关键词搜索：定位首个命中行，返回其前后各 15 行上下文（带行号）
    if let Some(kw) = keyword.as_deref().filter(|k| !k.trim().is_empty()) {
        let (hit_line, window) = locate_keyword_context(&content, kw, 15);
        return match hit_line {
            Some(line_no) => Ok(format!(
                "【版本 {} 最新崩溃报告 {} 关键词“{}”命中第 {} 行（前后各 15 行上下文）】\n{}",
                version_id,
                newest.file_name().unwrap_or_default().to_string_lossy(),
                kw,
                line_no,
                window
            )),
            None => Ok(format!(
                "（在版本 {} 最新崩溃报告 {} 中未找到关键词“{}”）",
                version_id,
                newest.file_name().unwrap_or_default().to_string_lossy(),
                kw
            )),
        };
    }

    if let (Some(s), Some(e)) = (start, end) {
        let snippet = read_line_range(&content, s, e, 6000)?;
        return Ok(format!(
            "【版本 {} 最新崩溃报告 {} 第 {}~{} 行】\n{}",
            version_id,
            newest.file_name().unwrap_or_default().to_string_lossy(),
            s,
            e,
            snippet
        ));
    }

    let trimmed = truncate_chars(&content, 6000);
    Ok(format!(
        "【版本 {} 最新崩溃报告 {}】\n{}",
        version_id,
        newest.file_name().unwrap_or_default().to_string_lossy(),
        trimmed
    ))
}

/// 分析崩溃日志：读取最新崩溃报告 → 本地预检 → 返回问题范围摘要
pub(super) fn analyze_crash_log(version_id: &str, ctx: &AgentContext) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id).join("crash-reports");
    let Some(newest) = newest_file(&dir, Some("txt")) else {
        return Ok(format!(
            "（版本 {} 的 crash-reports 目录不存在或为空，游戏可能未崩溃过）",
            version_id
        ));
    };
    let content =
        std::fs::read_to_string(&newest).map_err(|e| format!("读取崩溃报告失败: {}", e))?;
    Ok(local_precheck_summary(version_id, &content, &newest))
}
