//! class 翻译路由：确定性排除 → AI 判定批次 → 未覆盖兜底

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ai_core::{self, PromptKind};
use crate::log_warn;

use super::super::class;
use super::super::ledger::ClassDecisionLedger;
use super::super::progress;
use super::super::types::{ClassCandidate, JarInspection, ProgressFn, RetryInfo};
use super::apply::{apply_decision, record_decision};
use super::prompt::{build_class_prompt, parse_and_validate_decisions};

const CLASS_BATCH_SIZE: usize = 16;
const MAX_BATCH_ATTEMPTS: usize = 3;

pub(crate) async fn run_class_route(
    workspace: &Path,
    inspection: &JarInspection,
    config: &ai_core::AiConfig,
    model: &str,
    class_ledger: &mut ClassDecisionLedger,
    cancel: &AtomicBool,
    on_progress: &ProgressFn,
) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("任务已取消".to_string());
    }
    resolve_deterministic_exclusions(inspection, class_ledger);
    let candidates = class_ledger.unresolved(&inspection.class_candidates);
    let total = candidates.len();
    if total == 0 {
        on_progress(100.0, "class 文本无待判定候选", None);
        return Ok(());
    }
    let mut handled = 0usize;
    for batch in candidates.chunks(CLASS_BATCH_SIZE) {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        let batch_refs: Vec<&ClassCandidate> = batch.iter().collect();
        let base_progress = 100.0 * (handled as f64 / total as f64);
        let batch_cap = 100.0 * ((handled + batch.len()).min(total) as f64 / total as f64);
        let mut last_error = None;
        for attempt in 0..MAX_BATCH_ATTEMPTS {
            if cancel.load(Ordering::Relaxed) {
                return Err("任务已取消".to_string());
            }
            let retry = (attempt > 0).then(|| RetryInfo {
                attempt: attempt as u32 + 1,
                total: MAX_BATCH_ATTEMPTS as u32,
            });
            let msg = move |_p: f64| {
                if attempt > 0 {
                    format!("class 判定第 {}/{} 次重试", attempt + 1, MAX_BATCH_ATTEMPTS)
                } else {
                    "class 文本判定中".to_string()
                }
            };
            let user_prompt = build_class_prompt(inspection, &batch_refs, last_error.as_deref());
            let content = match tokio::select! {
                result = ai_core::chat_json(
                    config,
                    PromptKind::ModTranslation,
                    user_prompt,
                    Some(model),
                    Some(progress::AI_TIMEOUT_SECS),
                ) => result,
                _ = progress::wait_cancel(cancel) => return Err("任务已取消".to_string()),
                _ = progress::smooth_progress(
                    "class",
                    base_progress,
                    batch_cap,
                    cancel,
                    on_progress,
                    msg,
                    retry,
                ) => return Err("任务已取消".to_string()),
            } {
                Ok(content) => content,
                Err(e) => {
                    let msg = format!("AI class 判定调用失败: {e}");
                    log_warn!("[ModTranslation] {msg}");
                    last_error = Some(msg);
                    if attempt + 1 == MAX_BATCH_ATTEMPTS {
                        break;
                    }
                    continue;
                }
            };
            match parse_and_validate_decisions(&content, &batch_refs) {
                Ok((valid, uncovered)) => {
                    if valid.is_empty() && uncovered.is_empty() {
                        log_warn!("[ModTranslation] class 判定批次全部无效，跳过");
                    }
                    for decision in &valid {
                        match apply_decision(workspace, &batch_refs, decision, class_ledger) {
                            Ok(()) => handled += 1,
                            Err(e) => log_warn!("[ModTranslation] class 处置失败: {e}"),
                        }
                    }
                    if !uncovered.is_empty() {
                        handled += retry_uncovered(
                            workspace,
                            inspection,
                            &uncovered,
                            config,
                            model,
                            class_ledger,
                            cancel,
                        )
                        .await;
                    }
                    break;
                }
                Err(e) => {
                    let msg = format!("class 判定解析失败: {e}");
                    log_warn!("[ModTranslation] {msg}");
                    last_error = Some(msg);
                    if attempt + 1 == MAX_BATCH_ATTEMPTS {
                        break;
                    }
                }
            }
        }
        let progress = 100.0 * (handled as f64 / total as f64);
        on_progress(
            progress,
            &format!("class 文本判定：{handled}/{total}"),
            None,
        );
    }
    let final_progress = 100.0 * (handled as f64 / total as f64);
    on_progress(final_progress, "class 文本翻译完成", None);
    Ok(())
}

pub(crate) fn resolve_deterministic_exclusions(
    inspection: &JarInspection,
    class_ledger: &mut ClassDecisionLedger,
) {
    for candidate in &inspection.class_candidates {
        let Some(reason) =
            class::deterministic_class_exclusion_reason(&candidate.path, &candidate.text)
        else {
            continue;
        };
        record_decision(class_ledger, &candidate.id, "exclude", None, Some(reason));
    }
}

/// 未覆盖候选单独请求一次，失败即跳过（不整批重试）
async fn retry_uncovered(
    workspace: &Path,
    inspection: &JarInspection,
    uncovered: &[&ClassCandidate],
    config: &ai_core::AiConfig,
    model: &str,
    class_ledger: &mut ClassDecisionLedger,
    cancel: &AtomicBool,
) -> usize {
    if uncovered.is_empty() || cancel.load(Ordering::Relaxed) {
        return 0;
    }
    let user_prompt = build_class_prompt(inspection, uncovered, None);
    let content = match ai_core::chat_json(
        config,
        PromptKind::ModTranslation,
        user_prompt,
        Some(model),
        Some(progress::AI_TIMEOUT_SECS),
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            log_warn!("[ModTranslation] class 未覆盖候选单独请求失败: {e}");
            return 0;
        }
    };
    match parse_and_validate_decisions(&content, uncovered) {
        Ok((valid, _)) => {
            let mut handled = 0;
            for decision in &valid {
                match apply_decision(workspace, uncovered, decision, class_ledger) {
                    Ok(()) => handled += 1,
                    Err(e) => log_warn!("[ModTranslation] class 处置失败: {e}"),
                }
            }
            handled
        }
        Err(e) => {
            log_warn!("[ModTranslation] class 未覆盖候选解析失败: {e}");
            0
        }
    }
}
