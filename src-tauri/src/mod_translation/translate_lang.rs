//! 模组翻译：语言翻译路由（memory 命中 → fast/deep 双通道批量翻译 → 校验 → 写回）

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ai_core::{self, PromptKind};
use crate::{log_info, log_warn};

use super::lang;
use super::ledger::{WorkGraph, WorkKind};
use super::memory::TranslationMemory;
use super::prompt;
use super::quality;
use super::types::{JarInspection, LanguageKind, LanguageSource, ProgressFn, RetryInfo};

/// 单条模型尝试累计上限（fast + deep 通道合计）
const MAX_ITEM_ATTEMPTS: usize = 6;
/// fast 通道单条最大尝试轮数
const FAST_MAX_ATTEMPTS: usize = 2;
/// deep 通道单条最大尝试轮数
const DEEP_MAX_ATTEMPTS: usize = 3;

/// 语言翻译路由：过滤待译条目 → 分批（memory 命中短路）→ 双通道 → 逐 source 写回
pub async fn run_language_route(
    workspace: &Path,
    inspection: &JarInspection,
    config: &ai_core::AiConfig,
    model: &str,
    batch_size: usize,
    memory: &mut TranslationMemory,
    work_graph: &mut WorkGraph,
    cancel: &AtomicBool,
    on_progress: &ProgressFn,
) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("任务已取消".to_string());
    }
    let sources: Vec<&LanguageSource> = inspection
        .language_sources
        .iter()
        .filter(|s| s.required_count() > 0)
        .collect();
    let total: usize = sources.iter().map(|s| s.required_count()).sum();
    if total == 0 {
        on_progress(100.0, "没有需要翻译的条目", None);
        return Ok(());
    }
    let batch_size = batch_size.clamp(1, 100);
    let mut translated = 0usize;

    for source in &sources {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        let entries: Vec<(String, String)> = source
            .entries
            .iter()
            .filter(|(key, src)| {
                quality::requires_work(
                    key,
                    src,
                    source.existing_target.get(*key).map(String::as_str),
                )
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let mut accepted = BTreeMap::new();
        for chunk in entries.chunks(batch_size) {
            if cancel.load(Ordering::Relaxed) {
                return Err("任务已取消".to_string());
            }
            let progress = 100.0 * (translated as f64 / total as f64);
            let batch_cap = 100.0 * ((translated + chunk.len()).min(total) as f64 / total as f64);
            let batch = translate_batch(
                inspection,
                source,
                chunk,
                config,
                model,
                memory,
                work_graph,
                cancel,
                on_progress,
                progress,
                batch_cap,
            )
            .await?;
            let done = batch.len();
            accepted.extend(batch);
            translated += done;
            on_progress(
                batch_cap,
                &format!("翻译中：{translated}/{total} 条目"),
                None,
            );
        }
        if !accepted.is_empty() {
            write_back(workspace, source, &accepted)?;
            for (key, translation) in &accepted {
                if let Some(src) = source.entries.get(key) {
                    memory.record(
                        &inspection.mod_ids,
                        &source.namespace,
                        src,
                        translation.clone(),
                    );
                }
            }
            log_info!(
                "[ModTranslation] 写回 {}（{} 条）",
                source.target_path,
                accepted.len()
            );
        }
    }
    on_progress(100.0, "语言翻译完成", None);
    Ok(())
}

/// 翻译一批条目：memory 命中直取；未命中走 fast（≤2 轮）/ deep（≤3 轮）双通道
async fn translate_batch(
    inspection: &JarInspection,
    source: &LanguageSource,
    batch: &[(String, String)],
    config: &ai_core::AiConfig,
    model: &str,
    memory: &mut TranslationMemory,
    work_graph: &mut WorkGraph,
    cancel: &AtomicBool,
    on_progress: &ProgressFn,
    base_progress: f64,
    batch_cap: f64,
) -> Result<BTreeMap<String, String>, String> {
    let mut accepted: BTreeMap<String, String> = BTreeMap::new();
    let mut pending: Vec<(String, String)> = Vec::new();
    let mut ids: BTreeMap<String, String> = BTreeMap::new();
    for (key, src) in batch {
        let item_id = work_graph.upsert(
            WorkKind::Language,
            "提供自然且格式完整的简体中文".to_string(),
            format!("{}#{}", source.target_path, key),
            1.0,
        );
        ids.insert(key.clone(), item_id);
        match memory.lookup(&inspection.mod_ids, &source.namespace, src) {
            Some(hit) => {
                work_graph.reconcile(&ids[key], true, "翻译记忆命中");
                accepted.insert(key.clone(), hit);
            }
            None => pending.push((key.clone(), src.clone())),
        }
    }

    for (action, max_rounds) in [
        ("fast_translate", FAST_MAX_ATTEMPTS),
        ("deep_translate", DEEP_MAX_ATTEMPTS),
    ] {
        let mut round = 0;
        while round < max_rounds && !pending.is_empty() {
            round += 1;
            if cancel.load(Ordering::Relaxed) {
                return Err("任务已取消".to_string());
            }
            let retry = (round > 1).then(|| RetryInfo {
                attempt: round as u32,
                total: max_rounds as u32,
            });
            let msg = move |_p: f64| {
                if round > 1 {
                    format!("{action} 第 {round}/{max_rounds} 次重试")
                } else {
                    "翻译中".to_string()
                }
            };
            pending
                .retain(|(key, _)| work_graph.model_attempt_count(&ids[key]) < MAX_ITEM_ATTEMPTS);
            if pending.is_empty() {
                break;
            }
            let user_prompt = prompt::build_translation_user_prompt(
                inspection.loader,
                &inspection.mod_ids,
                &source.namespace,
                &pending,
            );
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
                    log_warn!("[ModTranslation] AI 批量翻译调用失败: {e}");
                    continue;
                }
            };
            let parsed = prompt::parse_translations_response(&content)?;
            let by_key: BTreeMap<&str, &str> = parsed
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let mut rejected: Vec<(String, String)> = Vec::new();
            for (key, src) in &pending {
                let id = &ids[key];
                let raw = by_key.get(key.as_str()).copied().unwrap_or("");
                match evaluate_translation(src, raw) {
                    Ok(translation) => {
                        work_graph.record_attempt(id, action.to_string(), "ok".to_string(), None);
                        work_graph.reconcile(id, true, "译文已通过校验");
                        accepted.insert(key.clone(), translation);
                    }
                    Err(error) => {
                        work_graph.record_attempt(
                            id,
                            action.to_string(),
                            "rejected".to_string(),
                            Some(error.clone()),
                        );
                        work_graph.reconcile(id, false, &error);
                        rejected.push((key.clone(), src.clone()));
                    }
                }
            }
            pending = rejected;
        }
    }
    Ok(accepted)
}

/// 单条译文验收：normalize 后校验空译文与占位符，通过返回译文
fn evaluate_translation(source: &str, raw: &str) -> Result<String, String> {
    let translation = quality::normalize_model_translation(source, raw);
    if translation.trim().is_empty() {
        return Err("没有返回译文".to_string());
    }
    if let Some(error) = quality::validate_protected_tokens(source, &translation) {
        return Err(error);
    }
    Ok(translation)
}

/// 按语言源类型写回目标文件（临时文件 + rename 原子写）
fn write_back(
    workspace: &Path,
    source: &LanguageSource,
    translations: &BTreeMap<String, String>,
) -> Result<(), String> {
    if translations.is_empty() {
        return Ok(());
    }
    let source_path = workspace.join(&source.source_path);
    let target_path = workspace.join(&source.target_path);
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("无法创建目标目录: {e}"))?;
    }
    let original = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("无法读取 {}: {e}", source_path.display()))?;
    let output = match source.kind {
        LanguageKind::Json => lang::write_json_lang(&original, translations)?,
        LanguageKind::KeyValue => {
            let (_, lines) = lang::parse_keyvalue(&original);
            lang::write_keyvalue(&lines, translations)
        }
        LanguageKind::StructuredJson => lang::apply_structured_strings(&original, translations)?,
        LanguageKind::FreeText => {
            let lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
            lang::align_free_text(&lines, translations).join("\n")
        }
    };
    let temporary = target_path.with_extension("tmp");
    std::fs::write(&temporary, output)
        .map_err(|e| format!("无法写入 {}: {e}", temporary.display()))?;
    std::fs::rename(&temporary, &target_path)
        .map_err(|e| format!("无法移动 {}: {e}", target_path.display()))
}

#[cfg(test)]
#[path = "translate_lang_test.rs"]
mod tests;
