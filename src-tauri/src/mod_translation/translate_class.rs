//! 模组翻译：class 常量池文本翻译路由（确定性排除 → AI 判定 → 改写写回）

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;

use crate::ai_core::{self, PromptKind};
use crate::log_warn;

use super::class;
use super::ledger::{ClassDecision, ClassDecisionLedger};
use super::prompt;
use super::quality;
use super::types::{has_chinese, ClassCandidate, JarInspection, ProgressFn, RetryInfo};

const CLASS_BATCH_SIZE: usize = 16;
const MAX_BATCH_ATTEMPTS: usize = 3;

pub async fn run_class_route(
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
    let batch_count = (total + CLASS_BATCH_SIZE - 1) / CLASS_BATCH_SIZE;
    for (batch_idx, batch) in candidates.chunks(CLASS_BATCH_SIZE).enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        let base_progress = 100.0 * (handled as f64 / total as f64);
        let batch_cap = 100.0 * ((handled + batch.len()).min(total) as f64 / total as f64);
        let (mut last_error, mut decisions) = (None, Vec::new());
        for attempt in 0..MAX_BATCH_ATTEMPTS {
            if cancel.load(Ordering::Relaxed) {
                return Err("任务已取消".to_string());
            }
            let retry = (attempt > 0).then(|| RetryInfo {
                attempt: attempt as u32 + 1,
                total: MAX_BATCH_ATTEMPTS as u32,
            });
            let msg = move |p: f64| {
                if attempt > 0 {
                    format!("class 判定第 {}/{} 次重试", attempt + 1, MAX_BATCH_ATTEMPTS)
                } else {
                    format!(
                        "class 文本判定：批次 {}/{}（{handled}/{total} · {p:.0}%）",
                        batch_idx + 1,
                        batch_count
                    )
                }
            };
            let user_prompt = build_class_prompt(inspection, batch, last_error.as_deref());
            let content = match tokio::select! {
                result = ai_core::chat_json(
                    config,
                    PromptKind::ModTranslation,
                    user_prompt,
                    Some(model),
                    Some(super::AI_TIMEOUT_SECS),
                ) => result,
                _ = super::wait_cancel(cancel) => return Err("任务已取消".to_string()),
                _ = super::smooth_progress(
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
            match parse_and_validate_decisions(&content, batch) {
                Ok(value) => {
                    decisions = value;
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
        if decisions.is_empty() {
            log_warn!(
                "[ModTranslation] class 判定批次失败：{}",
                last_error.unwrap_or_default()
            );
            continue;
        }
        for decision in decisions {
            match apply_decision(workspace, batch, &decision, class_ledger) {
                Ok(()) => handled += 1,
                Err(e) => log_warn!("[ModTranslation] class 处置失败: {e}"),
            }
        }
        let progress = 100.0 * (handled as f64 / total as f64);
        on_progress(
            progress,
            &format!(
                "class 文本判定：批次 {}/{}（{handled}/{total}）",
                batch_idx + 1,
                batch_count
            ),
            None,
        );
    }
    on_progress(100.0, "class 文本翻译完成", None);
    Ok(())
}

fn resolve_deterministic_exclusions(
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

fn build_class_prompt(
    inspection: &JarInspection,
    batch: &[ClassCandidate],
    last_error: Option<&str>,
) -> String {
    let candidates: Vec<Value> = batch
        .iter()
        .map(|c| serde_json::json!({"id": c.id, "path": c.path, "text": c.text, "paths": c.paths}))
        .collect();
    let mut prompt_value = serde_json::json!({
        "task": "判断 Minecraft 模组 class 常量文本是否展示给玩家；translate 必须提供含简体中文的 translation 且占位符原样保留，exclude 必须提供 reason",
        "output": "只输出 JSON 对象：{\"decisions\":[{\"id\":\"候选id\",\"action\":\"translate\"|\"exclude\",\"translation\":\"译文\",\"reason\":\"理由\"}]}。id 必须原样复制 candidates 中的 id，每个候选恰好一个 decision，不得遗漏、不得新增。注意：本任务输出 decisions 数组，不是 translations 数组。",
        "loader": inspection.loader.as_str(),
        "modIds": inspection.mod_ids,
        "candidates": candidates,
    });
    if let Some(error) = last_error {
        prompt_value["retryNote"] = format!("上次判定校验失败：{error}，请重发合法 JSON").into();
    }
    prompt_value.to_string()
}

fn str_at<'a>(item: &'a Value, key: &str) -> &'a str {
    item.get(key).and_then(Value::as_str).unwrap_or("")
}

fn parse_and_validate_decisions(
    content: &str,
    candidates: &[ClassCandidate],
) -> Result<Vec<DecisionEntry>, String> {
    let stripped = prompt::strip_json_fences(content);
    let start = stripped.find('{').ok_or("AI 响应中未找到 JSON 对象")?;
    let end = stripped.rfind('}').ok_or("AI 响应中未找到 JSON 对象")?;
    let value: Value = serde_json::from_str(&stripped[start..=end])
        .map_err(|e| format!("解析 class 判定 JSON 失败: {e}"))?;
    let items = value
        .get("decisions")
        .and_then(Value::as_array)
        .ok_or("AI 响应缺少 decisions 数组")?;
    let expected: HashSet<&str> = candidates.iter().map(|c| c.id.as_str()).collect();
    let by_id: HashMap<_, _> = candidates.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let id = str_at(item, "id");
        if !expected.contains(id) || !seen.insert(id) {
            return Err(format!("模型返回未知或重复 class 候选：{id}"));
        }
        let action = str_at(item, "action");
        let reason = str_at(item, "reason");
        let candidate = by_id.get(id).ok_or_else(|| format!("候选缺失：{id}"))?;
        match action {
            "exclude" => {
                if reason.trim().is_empty() {
                    return Err(format!("class 候选 {id} 缺少判定理由"));
                }
                result.push(DecisionEntry {
                    id: id.to_string(),
                    action: "exclude".to_string(),
                    translation: None,
                    reason: Some(reason.to_string()),
                });
            }
            "translate" => {
                let raw = str_at(item, "translation");
                let translation = quality::normalize_model_translation(&candidate.text, raw);
                if !has_chinese(&translation) {
                    return Err(format!("class 候选 {id} 的译文不含简体中文"));
                }
                if let Some(error) =
                    quality::validate_protected_tokens(&candidate.text, &translation)
                {
                    return Err(format!("class 候选 {id}：{error}"));
                }
                result.push(DecisionEntry {
                    id: id.to_string(),
                    action: "translate".to_string(),
                    translation: Some(translation),
                    reason: Some(reason.to_string()),
                });
            }
            other => return Err(format!("class 候选 {id} 返回未知动作：{other}")),
        }
    }
    let (seen_n, expected_n) = (seen.len(), expected.len());
    if seen_n != expected_n {
        return Err(format!("模型只判定了 {seen_n}/{expected_n} 个候选"));
    }
    Ok(result)
}

fn apply_decision(
    workspace: &Path,
    candidates: &[ClassCandidate],
    decision: &DecisionEntry,
    ledger: &mut ClassDecisionLedger,
) -> Result<(), String> {
    let candidate = candidates
        .iter()
        .find(|c| c.id == decision.id)
        .ok_or_else(|| format!("未知候选：{}", decision.id))?;
    match decision.action.as_str() {
        "exclude" => {
            record_decision(
                ledger,
                &candidate.id,
                "exclude",
                None,
                decision.reason.as_deref(),
            );
            Ok(())
        }
        "translate" => {
            let translation = decision
                .translation
                .clone()
                .ok_or_else(|| format!("class 候选 {} 缺少译文", candidate.id))?;
            apply_class_translation(workspace, candidate, &translation, ledger)
        }
        other => Err(format!("未知动作：{other}")),
    }
}

fn apply_class_translation(
    workspace: &Path,
    candidate: &ClassCandidate,
    translation: &str,
    class_ledger: &mut ClassDecisionLedger,
) -> Result<(), String> {
    if candidate.text == translation {
        record_decision(
            class_ledger,
            &candidate.id,
            "exclude",
            None,
            Some("译文与原文一致"),
        );
        return Ok(());
    }
    let paths = match candidate.paths.as_slice() {
        [] => vec![candidate.path.clone()],
        _ => candidate.paths.clone(),
    };
    for relative in &paths {
        let target_path = workspace.join(relative);
        let bytes = std::fs::read(&target_path)
            .map_err(|e| format!("无法读取 {}: {e}", target_path.display()))?;
        let rewritten = class::replace_class_utf8(&bytes, &candidate.text, translation)?;
        if rewritten == bytes {
            return Err(format!("class 候选在声明路径中不存在：{relative}"));
        }
        let temporary = target_path.with_extension("class.tmp");
        std::fs::write(&temporary, &rewritten)
            .map_err(|e| format!("无法写入 {}: {e}", temporary.display()))?;
        std::fs::rename(&temporary, &target_path)
            .map_err(|e| format!("无法移动 {}: {e}", target_path.display()))?;
        if !class_ledger.replaced_files.contains(relative) {
            class_ledger.replaced_files.push(relative.clone());
        }
    }
    record_decision(
        class_ledger,
        &candidate.id,
        "translate",
        Some(translation),
        None,
    );
    class_ledger.replacement_count += paths.len();
    Ok(())
}

fn record_decision(
    ledger: &mut ClassDecisionLedger,
    id: &str,
    action: &str,
    translation: Option<&str>,
    reason: Option<&str>,
) {
    ledger.decisions.insert(
        id.to_string(),
        ClassDecision {
            action: action.to_string(),
            translation: translation.map(str::to_string),
            reason: reason.map(str::to_string),
        },
    );
}

struct DecisionEntry {
    id: String,
    action: String,
    translation: Option<String>,
    reason: Option<String>,
}

#[cfg(test)]
#[path = "translate_class_test.rs"]
mod tests;
