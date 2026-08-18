//! 模组翻译：质量回修兜底（复验 → AI 修复方案 → 原子写回）

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::Ordering;

use sha2::{Digest, Sha256};

use crate::{ai_core, log_warn};

use super::lang;
use super::ledger::{WorkGraph, WorkKind, WorkStatus};
use super::quality::{audit_invariants, audit_semantic, AuditSeverity};
use super::types::{LanguageKind, LanguageSource, ProgressFn};

#[path = "repair_ai.rs"]
mod repair_ai;
#[path = "repair_apply.rs"]
mod repair_apply;

#[cfg(test)]
#[path = "repair_test.rs"]
mod repair_test;

#[cfg(test)]
pub(super) use repair_ai::{parse_actions_response, validate_response};
#[cfg(test)]
pub(super) use repair_apply::apply_actions;

const MAX_REPAIR_PASSES: usize = 4;
const MAX_REPAIR_BATCH: usize = 24;

#[derive(Debug, Clone)]
pub struct RepairIssue {
    pub id: String,
    pub kind: String,
    pub target_path: Option<String>,
    pub key: Option<String>,
    pub source: String,
    pub current: Option<String>,
    pub messages: Vec<String>,
    pub actionable: bool,
}

#[derive(Debug, Clone)]
pub struct RepairAction {
    pub action: String,
    pub issue_id: String,
    pub translation: Option<String>,
    pub reason: Option<String>,
}

pub fn collect_issues(
    workspace: &Path,
    sources: &[&LanguageSource],
    work_graph: &WorkGraph,
) -> Vec<RepairIssue> {
    let mut issues = Vec::new();
    for source in sources {
        let target = read_target_map(workspace, source);
        let mut by_key: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for audit in audit_invariants(&source.entries, &target)
            .into_iter()
            .chain(audit_semantic(&source.entries, &target))
        {
            if audit.severity == AuditSeverity::Error
                && !is_superseded(work_graph, &source.target_path, &audit.key)
            {
                by_key.entry(audit.key).or_default().push(audit.message);
            }
        }
        for (key, messages) in by_key {
            issues.push(RepairIssue {
                id: issue_id("language", &format!("{}#{}", source.target_path, key)),
                kind: "language".to_string(),
                target_path: Some(source.target_path.clone()),
                key: Some(key.clone()),
                source: source.entries.get(&key).cloned().unwrap_or_default(),
                current: target.get(&key).cloned(),
                messages,
                actionable: true,
            });
        }
    }
    issues.sort_by(|a, b| a.id.cmp(&b.id));
    issues
}

fn read_target_map(workspace: &Path, source: &LanguageSource) -> BTreeMap<String, String> {
    let Ok(content) = std::fs::read_to_string(workspace.join(&source.target_path)) else {
        return BTreeMap::new();
    };
    match source.kind {
        LanguageKind::Json => lang::read_json_lang(&content).unwrap_or_default(),
        LanguageKind::KeyValue => lang::parse_keyvalue(&content).0.into_iter().collect(),
        LanguageKind::StructuredJson => {
            lang::collect_structured_strings(&content).unwrap_or_default()
        }
        LanguageKind::FreeText => {
            lang::read_localized_target(workspace, Path::new(&source.target_path))
                .into_iter()
                .filter_map(|(p, t)| {
                    Some((
                        p.strip_prefix("/lines/")?
                            .parse::<usize>()
                            .ok()?
                            .to_string(),
                        t,
                    ))
                })
                .collect()
        }
    }
}

pub(super) fn issue_id(kind: &str, source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

fn is_superseded(work_graph: &WorkGraph, target_path: &str, key: &str) -> bool {
    let source = format!("{target_path}#{key}");
    work_graph.items.values().any(|item| {
        item.kind == WorkKind::Language
            && item.source == source
            && item.status == WorkStatus::Superseded
    })
}

pub async fn run_repair_passes(
    workspace: &Path,
    sources: &[&LanguageSource],
    work_graph: &mut WorkGraph,
    config: &ai_core::AiConfig,
    model: &str,
    cancel: &std::sync::atomic::AtomicBool,
    on_progress: &ProgressFn,
) -> Result<bool, String> {
    for pass in 0..MAX_REPAIR_PASSES {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        let pass_progress = pass as f64 / MAX_REPAIR_PASSES as f64 * 100.0;
        on_progress(
            pass_progress,
            &format!("质量复验第 {}/{} 轮", pass + 1, MAX_REPAIR_PASSES),
            None,
        );
        let issues = collect_issues(workspace, sources, work_graph);
        if issues.is_empty() {
            return Ok(true);
        }
        let mut groups: BTreeMap<String, Vec<RepairIssue>> = BTreeMap::new();
        for issue in issues {
            let path = issue.target_path.clone().unwrap_or_default();
            groups.entry(path).or_default().push(issue);
        }
        for (target_path, group) in groups {
            if cancel.load(Ordering::Relaxed) {
                return Err("任务已取消".to_string());
            }
            let source = sources
                .iter()
                .find(|s| s.target_path == target_path)
                .ok_or_else(|| format!("未知语言目标：{target_path}"))?;
            for batch in group.chunks(MAX_REPAIR_BATCH) {
                // 回修是兜底：批次失败仅跳过，不阻塞打包
                match repair_ai::request_actions(
                    batch,
                    config,
                    model,
                    on_progress,
                    pass_progress,
                    cancel,
                )
                .await
                {
                    Ok(actions) => {
                        if let Err(e) =
                            repair_apply::apply_actions(workspace, source, &actions, work_graph)
                        {
                            log_warn!("[ModTranslation] 质量回修写回失败，跳过该批次: {e}");
                        }
                    }
                    Err(e) => log_warn!("[ModTranslation] 质量回修批次失败，跳过: {e}"),
                }
            }
        }
    }
    let remaining = collect_issues(workspace, sources, work_graph);
    Ok(remaining.is_empty())
}
