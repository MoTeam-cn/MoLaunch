//! 模组翻译：语言翻译路由（memory 命中 → fast/deep 双通道批量翻译 → 校验 → 写回）
//!
//! 批次内先查翻译记忆，命中的直接验收通过；未命中的按 fast（少轮）/ deep（多轮）
//! 两个通道逐轮请求，每轮只重试上一轮被拒的条目，超过总尝试上限即放弃。

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

/// 单条累计尝试上限（fast + deep 合计）
const MAX_ITEM_ATTEMPTS: usize = 6;
/// fast 通道轮数上限
const FAST_MAX_ATTEMPTS: usize = 2;
/// deep 通道轮数上限
const DEEP_MAX_ATTEMPTS: usize = 3;
/// 批次大小安全区间
const BATCH_RANGE: std::ops::RangeInclusive<usize> = 1..=100;

/// 语言翻译路由：过滤待译条目 → 分批（memory 命中短路）→ 双通道 → 逐 source 写回
#[allow(clippy::too_many_arguments)]
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
        .filter(|source| source.required_count() > 0)
        .collect();
    let total: usize = sources.iter().map(|source| source.required_count()).sum();
    if total == 0 {
        on_progress(100.0, "没有需要翻译的条目", None);
        return Ok(());
    }

    let batch_size = if BATCH_RANGE.contains(&batch_size) {
        batch_size
    } else {
        batch_size.clamp(*BATCH_RANGE.start(), *BATCH_RANGE.end())
    };
    let mut translated = 0usize;

    for source in sources {
        if cancel.load(Ordering::Relaxed) {
            return Err("任务已取消".to_string());
        }
        let entries = pending_entries(source);
        let mut accepted: BTreeMap<String, String> = BTreeMap::new();

        for chunk in entries.chunks(batch_size) {
            if cancel.load(Ordering::Relaxed) {
                return Err("任务已取消".to_string());
            }
            let base = 100.0 * (translated as f64 / total as f64);
            let cap = 100.0 * ((translated + chunk.len()).min(total) as f64 / total as f64);
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
                base,
                cap,
            )
            .await?;
            translated += batch.len();
            accepted.extend(batch);
            on_progress(cap, &format!("翻译中：{translated}/{total} 条目"), None);
        }

        if accepted.is_empty() {
            continue;
        }
        write_back(workspace, source, &accepted)?;
        for (key, translation) in &accepted {
            if let Some(text) = source.entries.get(key) {
                memory.record(
                    &inspection.mod_ids,
                    &source.namespace,
                    text,
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

    on_progress(
        100.0 * (translated as f64 / total as f64),
        "语言翻译完成",
        None,
    );
    Ok(())
}

/// 需要翻译的条目（跳过空文本、透传内容、已有译文）
fn pending_entries(source: &LanguageSource) -> Vec<(String, String)> {
    source
        .entries
        .iter()
        .filter(|(key, text)| {
            quality::requires_work(
                key,
                text,
                source.existing_target.get(*key).map(String::as_str),
            )
        })
        .map(|(key, text)| (key.clone(), text.clone()))
        .collect()
}

/// 一次批量翻译的上下文（避免长参数列表）
struct BatchCtx<'a> {
    inspection: &'a JarInspection,
    source: &'a LanguageSource,
    config: &'a ai_core::AiConfig,
    model: &'a str,
    cancel: &'a AtomicBool,
    on_progress: &'a ProgressFn,
    base_progress: f64,
    cap_progress: f64,
}

impl BatchCtx<'_> {
    /// 走 fast → deep 两个通道，返回本批通过的译文
    async fn translate(
        &self,
        pending: &[(String, String)],
        ids: &BTreeMap<String, String>,
        work_graph: &mut WorkGraph,
    ) -> Result<BTreeMap<String, String>, String> {
        let mut accepted: BTreeMap<String, String> = BTreeMap::new();
        let mut queue: Vec<(String, String)> = pending.to_vec();

        for (action, max_rounds) in [
            ("fast_translate", FAST_MAX_ATTEMPTS),
            ("deep_translate", DEEP_MAX_ATTEMPTS),
        ] {
            let mut round = 0;
            while round < max_rounds && !queue.is_empty() {
                round += 1;
                if self.cancel.load(Ordering::Relaxed) {
                    return Err("任务已取消".to_string());
                }

                queue.retain(|(key, _)| {
                    work_graph.model_attempt_count(&ids[key]) < MAX_ITEM_ATTEMPTS
                });
                if queue.is_empty() {
                    break;
                }

                let content = match self.request(action, round, max_rounds, &queue).await {
                    Ok(content) => content,
                    Err(error) => {
                        log_warn!("[ModTranslation] AI 批量翻译调用失败: {error}");
                        continue;
                    }
                };
                let parsed = prompt::parse_translations_response(&content)?;
                let by_key: BTreeMap<&str, &str> = parsed
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str()))
                    .collect();

                let mut rejected: Vec<(String, String)> = Vec::new();
                for (key, source_text) in &queue {
                    let id = &ids[key];
                    let raw = by_key.get(key.as_str()).copied().unwrap_or("");
                    match evaluate_translation(source_text, raw) {
                        Ok(translation) => {
                            work_graph.record_attempt(
                                id,
                                action.to_string(),
                                "ok".to_string(),
                                None,
                            );
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
                            rejected.push((key.clone(), source_text.clone()));
                        }
                    }
                }
                queue = rejected;
            }
        }

        Ok(accepted)
    }

    /// 单轮请求：取消与进度平滑都参与 select
    async fn request(
        &self,
        action: &str,
        round: usize,
        max_rounds: usize,
        pending: &[(String, String)],
    ) -> Result<String, String> {
        let retry = (round > 1).then_some(RetryInfo {
            attempt: round as u32,
            total: max_rounds as u32,
        });
        let label = if round > 1 {
            format!("{action} 第 {round}/{max_rounds} 次重试")
        } else {
            "翻译中".to_string()
        };
        let user_prompt = prompt::build_translation_user_prompt(
            self.inspection.loader,
            &self.inspection.mod_ids,
            &self.source.namespace,
            pending,
        );

        tokio::select! {
            result = ai_core::chat_json(
                self.config,
                PromptKind::ModTranslation,
                user_prompt,
                Some(self.model),
                Some(super::progress::AI_TIMEOUT_SECS),
            ) => result.map_err(|error| error.to_string()),
            _ = super::progress::wait_cancel(self.cancel) => {
                return Err("任务已取消".to_string())
            }
            _ = super::progress::smooth_progress(
                "language",
                self.base_progress,
                self.cap_progress,
                self.cancel,
                self.on_progress,
                move |_p: f64| label.clone(),
                retry,
            ) => return Err("任务已取消".to_string()),
        }
    }
}

/// 翻译一批条目：memory 命中直取；未命中走双通道
#[allow(clippy::too_many_arguments)]
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
    let mut ids: BTreeMap<String, String> = BTreeMap::new();
    let mut pending: Vec<(String, String)> = Vec::new();

    for (key, source_text) in batch {
        let item_id = work_graph.upsert(
            WorkKind::Language,
            "提供自然且格式完整的简体中文".to_string(),
            format!("{}#{}", source.target_path, key),
            1.0,
        );
        match memory.lookup(&inspection.mod_ids, &source.namespace, source_text) {
            Some(hit) => {
                work_graph.reconcile(&item_id, true, "翻译记忆命中");
                accepted.insert(key.clone(), hit);
            }
            None => pending.push((key.clone(), source_text.clone())),
        }
        ids.insert(key.clone(), item_id);
    }

    if pending.is_empty() {
        return Ok(accepted);
    }

    let ctx = BatchCtx {
        inspection,
        source,
        config,
        model,
        cancel,
        on_progress,
        base_progress,
        cap_progress: batch_cap,
    };
    let translated = ctx.translate(&pending, &ids, work_graph).await?;
    accepted.extend(translated);
    Ok(accepted)
}

/// 单条译文验收：先统一转义方向，再校验空译文与占位符
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

/// 按语言源类型写回目标文件（原子写）
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
            let lines: Vec<String> = original.lines().map(str::to_string).collect();
            lang::align_free_text(&lines, translations).join("\n")
        }
    };
    crate::utils::fs::write_atomic(&target_path, output.as_bytes())
}

#[cfg(test)]
#[path = "translate_lang_test.rs"]
mod tests;
