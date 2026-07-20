//! 崩溃分析（运行时日志 + 崩溃报告文件 + hs_err + latest.log）
//!
//! CrashAnalyzer 四步流程：
//! 1. Collect  — 收集 crash-reports/*.txt、hs_err_pid*.log、logs/latest.log、运行时日志
//! 2. Prepare  — 提取各源文本（头N行 + 尾M行）
//! 3. Analyze  — 三级关键字匹配（高优先级精准 → 堆栈分析 → 低优先级）
//! 4. Output   — 返回 CrashInfo（含 reason/suggestion/crash_report_path/log_tail）

mod collect;
mod crit1;
mod crit3;
mod stack;
mod util;

use super::types::{CrashCategory, CrashInfo, LogEntry};
use std::path::Path;

/// 分析崩溃（主入口）
///
/// 综合 exit_code、运行时日志、crash-reports 文件、hs_err 文件、latest.log 判断崩溃原因
pub(crate) async fn analyze_crash(
    exit_code: i32,
    logs: &[LogEntry],
    game_dir: &Path,
) -> Option<CrashInfo> {
    // 正常退出不分析
    if exit_code == 0 {
        return None;
    }

    crate::log_info!("[CrashAnalyzer] 开始崩溃分析（exit_code={}）", exit_code);

    // ===== 步骤1: Collect — 收集各源文本 =====
    let sources = collect::collect_sources(logs, game_dir);

    // ===== 步骤2: Analyze — 三级关键字匹配 =====

    // 第一级：高优先级精准匹配
    if let Some(info) = crit1::analyze_crit1(
        &sources.runtime_log,
        &sources.crash_report_text,
        &sources.hs_err_text,
        &sources.error_lines,
        sources.crash_report.as_ref().map(|(p, _)| p.as_path()),
    ) {
        crate::log_info!("[CrashAnalyzer] 一级匹配命中: {}", info.reason);
        return Some(info);
    }

    // 第二级：堆栈分析（仅当存在 Mod 加载器时）
    let has_mod_loader = sources.runtime_log.contains("orge")
        || sources.runtime_log.contains("abric")
        || sources.runtime_log.contains("uilt")
        || sources.runtime_log.contains("iteloader")
        || sources.runtime_log.contains("ModLauncher")
        || sources.runtime_log.contains("fmlloader");
    if has_mod_loader {
        if let Some(info) = stack::analyze_stack(
            &sources.runtime_log,
            &sources.crash_report_text,
            &sources.hs_err_text,
            &sources.error_lines,
            sources.crash_report.as_ref().map(|(p, _)| p.as_path()),
        ) {
            crate::log_info!("[CrashAnalyzer] 堆栈分析命中: {}", info.reason);
            return Some(info);
        }
    }

    // 第三级：低优先级匹配
    if let Some(info) = crit3::analyze_crit3(
        &sources.runtime_log,
        &sources.crash_report_text,
        &sources.error_lines,
        sources.crash_report.as_ref().map(|(p, _)| p.as_path()),
    ) {
        crate::log_info!("[CrashAnalyzer] 三级匹配命中: {}", info.reason);
        return Some(info);
    }

    // ===== 兜底：未识别的崩溃 =====
    crate::log_info!("[CrashAnalyzer] 未匹配到已知崩溃模式，返回通用崩溃信息");
    let log_tail: Vec<String> = if !sources.latest_log_tail.is_empty() {
        sources.latest_log_tail
    } else {
        logs.iter().rev().take(30).rev().map(|e| e.message.clone()).collect()
    };

    Some(CrashInfo {
        reason: format!("游戏异常退出（退出码 {}）", exit_code),
        category: CrashCategory::Unknown,
        log_lines: sources.error_lines,
        suggestion: "未识别到已知的崩溃模式。请查看日志文件获取更多信息，或尝试将崩溃报告发送给他人寻求帮助。".to_string(),
        problematic_mod: None,
        crash_report_path: sources.crash_report.as_ref().map(|(p, _)| p.to_string_lossy().to_string()),
        log_tail,
    })
}
