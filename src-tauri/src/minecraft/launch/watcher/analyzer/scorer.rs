//! 评分器：聚合多个检测器产出的证据，按置信度产出最终崩溃结论
//!
//! 评分规则：取置信度最高者作为最终结论；并列时保留先收集到的证据
//! （证据收集顺序即检测器注册顺序，规则表内条目顺序即同源规则优先级）。

use super::super::types::CrashInfo;
use super::collect::CollectedSources;
use super::detector::Evidence;

/// 从证据集聚合出最终崩溃结论
///
/// 返回 `None` 表示无任何证据命中，调用方应走兜底逻辑。
pub(super) fn score(evidence: &[Evidence], sources: &CollectedSources) -> Option<CrashInfo> {
    let best = evidence.iter().reduce(|acc, e| {
        if e.confidence > acc.confidence {
            e
        } else {
            acc
        }
    })?;

    crate::log_info!(
        "[CrashAnalyzer] 证据命中: [{}/{:?}] {}（置信度 {:.2}）",
        best.source.name(),
        best.category,
        best.reason,
        best.confidence
    );

    Some(CrashInfo {
        reason: best.reason.clone(),
        category: best.category.clone(),
        log_lines: sources.error_lines.clone(),
        suggestion: best.suggestion.clone(),
        problematic_mod: best.extracted_mod.clone(),
        crash_report_path: sources
            .crash_report
            .as_ref()
            .map(|(p, _)| p.to_string_lossy().to_string()),
        log_tail: Vec::new(),
    })
}
