//! 模组翻译：质量回修兜底（复验 → AI 修复方案 → 原子写回）
//!
//! 回修最多跑 `MAX_REPAIR_PASSES` 轮，每轮按「目标文件 + 批次」推进：
//! 批次失败只记录并跳过，最终以最后一轮复验是否清空问题作为返回值。

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
const STAGE: &str = "repair";

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

/// 一轮复验：按目标文件收集 Error 级问题，合并同一键的多条提示
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
            if audit.severity != AuditSeverity::Error {
                continue;
            }
            if is_superseded(work_graph, &source.target_path, &audit.key) {
                continue;
            }
            by_key.entry(audit.key).or_default().push(audit.message);
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
                .filter_map(|(pointer, text)| {
                    let index = pointer.strip_prefix("/lines/")?.parse::<usize>().ok()?;
                    Some((index.to_string(), text))
                })
                .collect()
        }
    }
}

/// 问题 id：sha256(kind ␀ 源) 前 16 位
pub(super) fn issue_id(kind: &str, source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(source.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    digest[..16].to_string()
}

fn is_superseded(work_graph: &WorkGraph, target_path: &str, key: &str) -> bool {
    let source = format!("{target_path}#{key}");
    work_graph.items.values().any(|item| {
        item.kind == WorkKind::Language
            && item.source == source
            && item.status == WorkStatus::Superseded
    })
}

/// 按目标文件分组并按批次切分
fn batches_by_target(issues: Vec<RepairIssue>) -> Vec<(String, Vec<RepairIssue>)> {
    let mut groups: BTreeMap<String, Vec<RepairIssue>> = BTreeMap::new();
    for issue in issues {
        let path = issue.target_path.clone().unwrap_or_default();
        groups.entry(path).or_default().push(issue);
    }
    groups
        .into_iter()
        .flat_map(|(path, group)| {
            group
                .chunks(MAX_REPAIR_BATCH)
                .map(|batch| (path.clone(), batch.to_vec()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// 收集问题期间让进度平滑爬升，避免审计耗时看起来卡住
async fn collect_with_progress(
    workspace: &Path,
    sources: &[&LanguageSource],
    work_graph: &WorkGraph,
    pass: usize,
    progress: f64,
    pass_end: f64,
    on_progress: Arc<ProgressFn>,
) -> Vec<RepairIssue> {
    let stop = Arc::new(AtomicBool::new(false));
    let ticker = tokio::spawn({
        let on_progress = on_progress.clone();
        let stop = stop.clone();
        async move {
            super::progress::smooth_progress(
                STAGE,
                progress,
                pass_end,
                &stop,
                on_progress.as_ref(),
                move |_p: f64| format!("质量复验第 {}/{} 轮", pass + 1, MAX_REPAIR_PASSES),
                None,
            )
            .await;
        }
    });
    let issues = collect_issues(workspace, sources, work_graph);
    stop.store(true, Ordering::Relaxed);
    let _ = ticker.await;
    issues
}

pub async fn run_repair_passes(
    workspace: &Path,
    sources: &[&LanguageSource],
    work_graph: &mut WorkGraph,
    config: &ai_core::AiConfig,
    model: &str,
    cancel: Arc<AtomicBool>,
    on_progress: Arc<ProgressFn>,
) -> Result<bool, String> {
    let mut progress = 0.0;

    for pass in 0..MAX_REPAIR_PASSES {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        let pass_end = (pass + 1) as f64 / MAX_REPAIR_PASSES as f64 * 100.0;
        let issues = collect_with_progress(
            workspace,
            sources,
            work_graph,
            pass,
            progress,
            pass_end,
            on_progress.clone(),
        )
        .await;

        if issues.is_empty() {
            on_progress(100.0, "质量复验完成", None);
            return Ok(true);
        }
        progress = super::progress::current_stage_progress(STAGE).max(progress);

        let batches = batches_by_target(issues);
        let total_batches = batches.len().max(1);
        for (target_path, batch) in &batches {
            if cancel.load(Ordering::Relaxed) {
                return Err("任务已取消".to_string());
            }
            let base = progress;
            let cap = progress + (pass_end - progress) / total_batches as f64;
            let Some(source) = sources.iter().find(|s| s.target_path == *target_path) else {
                return Err(format!("未知语言目标：{target_path}"));
            };

            match repair_ai::request_actions(
                batch,
                config,
                model,
                on_progress.as_ref(),
                base,
                cap,
                &cancel,
            )
            .await
            {
                Ok(actions) => {
                    if let Err(error) =
                        repair_apply::apply_actions(workspace, source, &actions, work_graph)
                    {
                        log_warn!("[ModTranslation] 质量回修写回失败，跳过该批次: {error}");
                        let current = super::progress::current_stage_progress(STAGE);
                        on_progress(
                            current.max(base),
                            &format!("质量回修写回失败，跳过: {error}"),
                            None,
                        );
                    }
                }
                Err(error) => {
                    log_warn!("[ModTranslation] 质量回修批次失败，跳过: {error}");
                    let current = super::progress::current_stage_progress(STAGE);
                    on_progress(
                        current.max(base),
                        &format!("质量回修批次失败，跳过: {error}"),
                        None,
                    );
                }
            }
            progress = super::progress::current_stage_progress(STAGE).max(progress);
        }
    }

    let remaining = collect_issues(workspace, sources, work_graph);
    on_progress(100.0, "质量复验完成", None);
    Ok(remaining.is_empty())
}
