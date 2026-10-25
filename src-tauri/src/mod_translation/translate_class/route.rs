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
use super::prompt::{build_class_prompt, parse_and_validate_decisions, DecisionEntry};

const CLASS_BATCH_SIZE: usize = 16;
const MAX_BATCH_ATTEMPTS: usize = 3;
const STAGE: &str = "class";

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
        let base = 100.0 * (handled as f64 / total as f64);
        let cap = 100.0 * ((handled + batch.len()).min(total) as f64 / total as f64);

        handled += judge_batch(
            workspace,
            inspection,
            &batch_refs,
            config,
            model,
            class_ledger,
            cancel,
            on_progress,
            base,
            cap,
        )
        .await;

        on_progress(
            100.0 * (handled as f64 / total as f64),
            &format!("class 文本判定：{handled}/{total}"),
            None,
        );
    }

    on_progress(
        100.0 * (handled as f64 / total as f64),
        "class 文本翻译完成",
        None,
    );
    Ok(())
}

/// 确定性排除：java 类名 / 正则 / 内部诊断直接记为 exclude
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

/// 判定一个批次：失败重试，未覆盖候选单独补一次；返回成功处置数
#[allow(clippy::too_many_arguments)]
async fn judge_batch(
    workspace: &Path,
    inspection: &JarInspection,
    batch: &[&ClassCandidate],
    config: &ai_core::AiConfig,
    model: &str,
    class_ledger: &mut ClassDecisionLedger,
    cancel: &AtomicBool,
    on_progress: &ProgressFn,
    base_progress: f64,
    batch_cap: f64,
) -> usize {
    let mut last_error: Option<String> = None;

    for attempt in 0..MAX_BATCH_ATTEMPTS {
        if cancel.load(Ordering::Relaxed) {
            return 0;
        }
        let retry = (attempt > 0).then(|| RetryInfo {
            attempt: attempt as u32 + 1,
            total: MAX_BATCH_ATTEMPTS as u32,
        });
        let label = if attempt > 0 {
            format!("class 判定第 {}/{} 次重试", attempt + 1, MAX_BATCH_ATTEMPTS)
        } else {
            "class 文本判定中".to_string()
        };

        let user_prompt = build_class_prompt(inspection, batch, last_error.as_deref());
        let content = match request(
            config,
            model,
            user_prompt,
            cancel,
            on_progress,
            base_progress,
            batch_cap,
            label,
            retry,
        )
        .await
        {
            Ok(content) => content,
            Err(error) => {
                let message = format!("AI class 判定调用失败: {error}");
                log_warn!("[ModTranslation] {message}");
                last_error = Some(message);
                if attempt + 1 == MAX_BATCH_ATTEMPTS {
                    return 0;
                }
                continue;
            }
        };

        match parse_and_validate_decisions(&content, batch) {
            Ok((decisions, uncovered)) => {
                if decisions.is_empty() && uncovered.is_empty() {
                    log_warn!("[ModTranslation] class 判定批次全部无效，跳过");
                }
                let mut handled = apply_decisions(workspace, batch, &decisions, class_ledger);
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
                return handled;
            }
            Err(error) => {
                let message = format!("class 判定解析失败: {error}");
                log_warn!("[ModTranslation] {message}");
                last_error = Some(message);
                if attempt + 1 == MAX_BATCH_ATTEMPTS {
                    return 0;
                }
            }
        }
    }
    0
}

/// 单轮请求：取消与进度平滑参与 select
#[allow(clippy::too_many_arguments)]
async fn request(
    config: &ai_core::AiConfig,
    model: &str,
    user_prompt: String,
    cancel: &AtomicBool,
    on_progress: &ProgressFn,
    base_progress: f64,
    cap_progress: f64,
    label: String,
    retry: Option<RetryInfo>,
) -> Result<String, String> {
    tokio::select! {
        result = ai_core::chat_json(
            config,
            PromptKind::ModTranslation,
            user_prompt,
            Some(model),
            Some(progress::AI_TIMEOUT_SECS),
        ) => result,
        _ = progress::wait_cancel(cancel) => Err("任务已取消".to_string()),
        _ = progress::smooth_progress(
            STAGE,
            base_progress,
            cap_progress,
            cancel,
            on_progress,
            move |_p: f64| label.clone(),
            retry,
        ) => Err("任务已取消".to_string()),
    }
}

fn apply_decisions(
    workspace: &Path,
    batch: &[&ClassCandidate],
    decisions: &[DecisionEntry],
    class_ledger: &mut ClassDecisionLedger,
) -> usize {
    let mut handled = 0;
    for decision in decisions {
        match apply_decision(workspace, batch, decision, class_ledger) {
            Ok(()) => handled += 1,
            Err(error) => log_warn!("[ModTranslation] class 处置失败: {error}"),
        }
    }
    handled
}

/// 未覆盖候选单独请求一次，失败即跳过（不整批重试）
#[allow(clippy::too_many_arguments)]
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
        Ok(content) => content,
        Err(error) => {
            log_warn!("[ModTranslation] class 未覆盖候选单独请求失败: {error}");
            return 0;
        }
    };
    match parse_and_validate_decisions(&content, uncovered) {
        Ok((decisions, _)) => apply_decisions(workspace, uncovered, &decisions, class_ledger),
        Err(error) => {
            log_warn!("[ModTranslation] class 未覆盖候选解析失败: {error}");
            0
        }
    }
}
