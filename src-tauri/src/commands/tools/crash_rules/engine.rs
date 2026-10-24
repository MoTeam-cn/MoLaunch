//! 崩溃规则引擎（证据驱动）
//!
//! 与旧实现的区别：命中关键词只是「候选」，必须再定位到具体日志行才算结论；
//! 找不到行就进「待确认」，不再凭关键词直接下结论。同一 cause 只输出一条，
//! 证据合并取最早 + 最晚各一条。

use std::collections::HashMap;
use std::sync::OnceLock;

use super::evidence::{compiled, LogIndex};
use super::schema::{CrashRule, RuleSet};
use super::special;
use crate::commands::tools::types::{CrashAnalysisItem, CrashEvidence};

/// 单条结论最多展示的证据行数
const MAX_EVIDENCE_PER_ITEM: usize = 4;

/// 解析内置规则集（首次调用时解析，失败则返回空规则集）
fn ruleset() -> &'static RuleSet {
    static RULES: OnceLock<RuleSet> = OnceLock::new();
    RULES.get_or_init(|| {
        let raw = crate::resources::read_resource("crash_rules.json").unwrap_or_default();
        match serde_json::from_str::<RuleSet>(&raw) {
            Ok(set) => {
                crate::log_info!(
                    "[CrashRules] 已加载 {} 条崩溃规则（version {}）",
                    set.rules.len(),
                    set.version
                );
                set
            }
            Err(e) => {
                crate::log_error!("[CrashRules] 规则解析失败: {}", e);
                RuleSet {
                    version: "0".to_string(),
                    rules: Vec::new(),
                }
            }
        }
    })
}

/// 当前规则集版本
pub fn rules_version() -> String {
    ruleset().version.clone()
}

/// 规则总数（供调试与自检）
pub fn rules_count() -> usize {
    ruleset().rules.len()
}

/// 用规则集分析日志，只返回已确认条目
pub fn analyze(
    log_text: &str,
    mod_index: Option<&HashMap<String, String>>,
) -> Vec<CrashAnalysisItem> {
    analyze_with_pending(log_text, mod_index).0
}

/// 用规则集分析日志，返回（已确认条目, 待确认条目）
///
/// 已确认 = 至少有一条日志原文证据；待确认 = 规则命中但拿不到证据行。
pub fn analyze_with_pending(
    log_text: &str,
    mod_index: Option<&HashMap<String, String>>,
) -> (Vec<CrashAnalysisItem>, Vec<CrashAnalysisItem>) {
    if log_text.trim().is_empty() {
        return (Vec::new(), Vec::new());
    }
    let index = LogIndex::new(log_text);
    let mut confirmed: Vec<CrashAnalysisItem> = Vec::new();
    let mut pending: Vec<CrashAnalysisItem> = Vec::new();

    for rule in &ruleset().rules {
        if !passes_gates(rule, &index) || !matches(rule, &index) {
            continue;
        }
        let item = build_item(rule, &index, mod_index);
        if item.evidence.is_empty() {
            pending.push(item);
        } else {
            confirmed.push(item);
        }
    }

    let (special_confirmed, special_pending) = special::run(&index, mod_index);
    confirmed.extend(special_confirmed);
    pending.extend(special_pending);

    let mut confirmed = merge_by_cause(confirmed);
    let mut pending = merge_by_cause(pending);
    sort_items(&mut confirmed);
    sort_items(&mut pending);
    (confirmed, pending)
}

/// 规则门槛：行数区间与「必须出现 / 必须不出现」关键词
fn passes_gates(rule: &CrashRule, index: &LogIndex) -> bool {
    let gates = &rule.gates;
    if let Some(min) = gates.min_lines {
        if index.line_count() < min {
            return false;
        }
    }
    if let Some(max) = gates.max_lines {
        if index.line_count() > max {
            return false;
        }
    }
    if !gates.require_present.iter().all(|kw| index.contains(kw)) {
        return false;
    }
    if gates.require_absent.iter().any(|kw| index.contains(kw)) {
        return false;
    }
    true
}

/// 关键词 / 正则匹配（`all` / `any` / `regex` / `none`）
fn matches(rule: &CrashRule, index: &LogIndex) -> bool {
    let spec = &rule.match_spec;
    if spec.all.is_empty() && spec.any.is_empty() && spec.regex.is_empty() {
        return false;
    }
    if !spec.all.iter().all(|kw| index.contains(kw)) {
        return false;
    }
    if !spec.any.is_empty() && !spec.any.iter().any(|kw| index.contains(kw)) {
        return false;
    }
    if !spec.regex.is_empty()
        && !spec.regex.iter().any(|pattern| {
            compiled(pattern)
                .map(|re| re.is_match(&index.text_lower()))
                .unwrap_or(false)
        })
    {
        return false;
    }
    !spec.none.iter().any(|kw| index.contains(kw))
}

/// 构造分析条目（含证据与置信度）
fn build_item(
    rule: &CrashRule,
    index: &LogIndex,
    mod_index: Option<&HashMap<String, String>>,
) -> CrashAnalysisItem {
    let mut evidence: Vec<CrashEvidence> = Vec::new();
    let mut title = rule.title.clone();

    if rule.steps.is_empty() {
        let mut needles = rule.evidence.any.clone();
        let patterns = rule.evidence.regex.clone();
        if needles.is_empty() && patterns.is_empty() {
            if let Some(first) = rule
                .match_spec
                .all
                .first()
                .or_else(|| rule.match_spec.any.first())
            {
                needles.push(first.clone());
            }
        }
        evidence = index.collect(&needles, &patterns, MAX_EVIDENCE_PER_ITEM);
    } else {
        // 因果链：逐环取证，缺环就跳过该环节（不编造）
        let mut labels: Vec<String> = Vec::new();
        for step in &rule.steps {
            if let Some(found) = index.collect(&step.any, &[], 1).into_iter().next() {
                labels.push(step.label.clone());
                evidence.push(found);
            }
        }
        if !labels.is_empty() {
            title = labels.join(" → ");
        }
    }

    let cause = if rule.cause.is_empty() {
        rule.id.clone()
    } else {
        rule.cause.clone()
    };
    let detail = evidence
        .first()
        .map(|e| e.raw_line.clone())
        .unwrap_or_default();
    let missing_evidence = if evidence.is_empty() {
        format!(
            "规则命中关键词「{}」，但日志里找不到可引用的具体行，无法复核",
            rule.match_spec
                .all
                .first()
                .or_else(|| rule.match_spec.any.first())
                .cloned()
                .unwrap_or_else(|| rule.id.clone())
        )
    } else {
        String::new()
    };
    let suspects = if rule.hint_mod {
        find_suspect_mods(index, mod_index)
    } else {
        Vec::new()
    };

    CrashAnalysisItem {
        cause,
        category: rule.category.clone(),
        severity: rule.severity.clone(),
        title,
        detail,
        suggestion: rule.suggestion.clone(),
        impact: rule.impact.clone(),
        confidence: rule.confidence.clone(),
        evidence,
        missing_evidence,
        rule_id: rule.id.clone(),
        actions: rule.actions.clone(),
        refs: rule.refs.clone(),
        suspects,
    }
}

/// 按 cause 合并：证据取最早 + 最晚各一条，置信度与严重级别取更高者
fn merge_by_cause(items: Vec<CrashAnalysisItem>) -> Vec<CrashAnalysisItem> {
    let mut merged: Vec<CrashAnalysisItem> = Vec::new();
    for item in items {
        match merged
            .iter_mut()
            .find(|existing| existing.cause == item.cause)
        {
            Some(existing) => {
                let mut evidence: Vec<CrashEvidence> = existing
                    .evidence
                    .iter()
                    .cloned()
                    .chain(item.evidence.iter().cloned())
                    .collect();
                evidence.sort_by_key(|e| e.line_no);
                evidence.dedup_by_key(|e| e.line_no);
                if evidence.len() > 2 {
                    let first = evidence.remove(0);
                    let last = evidence.pop();
                    evidence = vec![first];
                    if let Some(last) = last {
                        evidence.push(last);
                    }
                }
                existing.detail = evidence
                    .first()
                    .map(|e| e.raw_line.clone())
                    .unwrap_or_default();
                existing.evidence = evidence;
                if confidence_rank(&item.confidence) > confidence_rank(&existing.confidence) {
                    existing.confidence = item.confidence;
                }
                if severity_rank(&item.severity) < severity_rank(&existing.severity) {
                    existing.severity = item.severity;
                }
                if existing.missing_evidence.is_empty() {
                    existing.missing_evidence = item.missing_evidence;
                }
            }
            None => merged.push(item),
        }
    }
    merged
}

/// 排序：错误优先 → 高置信优先 → 证据行号靠前优先
fn sort_items(items: &mut [CrashAnalysisItem]) {
    items.sort_by(|a, b| {
        severity_rank(&a.severity)
            .cmp(&severity_rank(&b.severity))
            .then(confidence_rank(&b.confidence).cmp(&confidence_rank(&a.confidence)))
            .then(a.first_line_no().cmp(&b.first_line_no()))
    });
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    }
}

fn confidence_rank(confidence: &str) -> u8 {
    match confidence {
        "high" => 2,
        "medium" => 1,
        _ => 0,
    }
}

/// 从 mod 索引里找出日志中出现的 mod（最多 3 个）
fn find_suspect_mods(index: &LogIndex, mod_index: Option<&HashMap<String, String>>) -> Vec<String> {
    let Some(table) = mod_index else {
        return Vec::new();
    };
    let mut found: Vec<String> = Vec::new();
    for (modid, display) in table {
        if modid.len() < 3 || found.len() >= 3 {
            continue;
        }
        if index.contains(modid) {
            found.push(display.clone());
        }
    }
    found.sort();
    found
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod engine_test;

#[cfg(test)]
#[path = "regression_test.rs"]
mod regression_test;
