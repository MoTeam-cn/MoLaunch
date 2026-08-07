//! 崩溃分析主流程编排
//!
//! 流程：Collect（收集各源文本）→ Detect（多路检测器并行提取证据）→ Score（置信度聚合）→ 兜底。
//! 规则以声明式数据表承载（rules.rs），检测器与评分器解耦，与早期"顺序短路 if 链"形态无关。

use super::super::types::{CrashCategory, CrashInfo, LogEntry};
use super::detector::{
    CaughtExceptionDetector, ClassNotFoundDetector, Detector, Evidence, KeywordDetector,
    ShortOutputDetector,
};
use super::detector_stack::StackDetector;
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

    // 步骤1: Collect — 收集各源文本
    let sources = super::collect::collect_sources(logs, game_dir);

    // 步骤2: Detect — 并行收集证据
    let evidence = detect_all(&sources);

    // 步骤3: Score — 置信度聚合
    if let Some(info) = super::scorer::score(&evidence, &sources) {
        return Some(info);
    }

    // 兜底：未识别的崩溃
    crate::log_info!("[CrashAnalyzer] 未匹配到已知崩溃模式，返回通用崩溃信息");
    let log_tail: Vec<String> = if !sources.latest_log_tail.is_empty() {
        sources.latest_log_tail
    } else {
        logs.iter()
            .rev()
            .take(30)
            .rev()
            .map(|e| e.message.clone())
            .collect()
    };

    Some(CrashInfo {
        reason: format!("游戏异常退出（退出码 {}）", exit_code),
        category: CrashCategory::Unknown,
        log_lines: sources.error_lines,
        suggestion: "未识别到已知的崩溃模式。请查看日志文件获取更多信息，或尝试将崩溃报告发送给他人寻求帮助。".to_string(),
        problematic_mod: None,
        crash_report_path: sources
            .crash_report
            .as_ref()
            .map(|(p, _)| p.to_string_lossy().to_string()),
        log_tail,
    })
}

/// 运行全部检测器收集证据
///
/// 检测器注册顺序即证据收集顺序；评分器在置信度并列时保留先收集到的证据。
fn detect_all(sources: &super::collect::CollectedSources) -> Vec<Evidence> {
    let detectors: Vec<Box<dyn Detector>> = vec![
        Box::new(KeywordDetector),
        Box::new(ClassNotFoundDetector),
        Box::new(CaughtExceptionDetector),
        Box::new(ShortOutputDetector),
        Box::new(StackDetector),
    ];

    let mut all = Vec::new();
    for detector in detectors {
        let evs = detector.detect(sources);
        crate::log_info!(
            "[CrashAnalyzer] 检测器 {} 产出 {} 条证据",
            detector.name(),
            evs.len()
        );
        all.extend(evs);
    }
    all
}

#[cfg(test)]
#[path = "analyze_tests.rs"]
mod tests;
