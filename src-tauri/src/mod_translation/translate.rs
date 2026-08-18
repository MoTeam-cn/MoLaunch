//! 模组翻译：翻译编排（分批 AI 翻译 → 校验 → 写回）

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ai_core::{self, PromptKind};
use crate::log_info;

use super::lang;
use super::prompt;
use super::types::{has_chinese, JarInspection, LanguageKind, LanguageSource, ProgressFn};

/// 单批最大重试次数（含首次调用共 2 次）
const MAX_BATCH_ATTEMPTS: usize = 2;

/// 取消哨兵错误消息（上层据此判定为用户取消而非失败）
pub const CANCEL_MSG: &str = "任务已取消";

/// 执行翻译：按 source 分批调 AI，逐 source 写回，进度经回调上报
pub async fn translate_sources(
    workspace: &Path,
    inspection: &JarInspection,
    config: &ai_core::AiConfig,
    model: &str,
    batch_size: usize,
    cancel: &AtomicBool,
    on_progress: &ProgressFn,
) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        return Err(CANCEL_MSG.to_string());
    }
    let sources: Vec<&LanguageSource> = inspection
        .language_sources
        .iter()
        .filter(|s| s.required_count() > 0)
        .collect();
    let total = sources.iter().map(|s| s.required_count()).sum::<usize>();
    if total == 0 {
        on_progress(100.0, "没有需要翻译的条目");
        return Ok(());
    }

    let batch_size = batch_size.clamp(1, 100);
    let mut translated = 0usize;

    for source in &sources {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCEL_MSG.to_string());
        }
        let entries: Vec<(String, String)> = source
            .entries
            .iter()
            .filter(|(key, src)| {
                let existing = source.existing_target.get(*key).map(String::as_str);
                !existing.is_some_and(|t| !t.trim().is_empty() && has_chinese(t))
                    && !src.trim().is_empty()
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let translations = translate_entries(
            inspection,
            source,
            &entries,
            config,
            model,
            batch_size,
            cancel,
            &mut |done| {
                translated += done;
                let progress = 10.0 + 80.0 * (translated as f64 / total as f64);
                on_progress(progress, &format!("翻译中：{}/{} 条目", translated, total));
            },
        )
        .await?;

        if !translations.is_empty() {
            write_back(workspace, source, &translations)?;
            log_info!(
                "[ModTranslation] 写回 {}（{} 条）",
                source.target_path,
                translations.len()
            );
        }
    }

    on_progress(90.0, "翻译完成，正在打包");
    Ok(())
}

/// 翻译单个 source 的全部条目（分批顺序调用）
async fn translate_entries(
    inspection: &JarInspection,
    source: &LanguageSource,
    entries: &[(String, String)],
    config: &ai_core::AiConfig,
    model: &str,
    batch_size: usize,
    cancel: &AtomicBool,
    on_batch_done: &mut (dyn FnMut(usize) + Send),
) -> Result<BTreeMap<String, String>, String> {
    let mut result = BTreeMap::new();
    for chunk in entries.chunks(batch_size) {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCEL_MSG.to_string());
        }
        let before = result.len();
        let mut pending: Vec<(String, String)> = chunk.to_vec();
        let mut attempt = 0;
        while !pending.is_empty() && attempt < MAX_BATCH_ATTEMPTS {
            attempt += 1;
            let user_prompt = prompt::build_translation_user_prompt(
                inspection.loader,
                &inspection.mod_ids,
                &source.namespace,
                &pending,
            );
            let content =
                ai_core::chat_json(config, PromptKind::ModTranslation, user_prompt, Some(model))
                    .await
                    .map_err(|e| format!("AI 批量翻译调用失败: {e}"))?;

            let parsed = prompt::parse_translations_response(&content)?;
            let mut accepted = BTreeMap::new();
            let mut rejected = Vec::new();
            let wanted: std::collections::BTreeSet<&str> =
                pending.iter().map(|(k, _)| k.as_str()).collect();
            for (key, translation) in parsed {
                if !wanted.contains(key.as_str()) {
                    continue;
                }
                if let Some((_, src)) = pending.iter().find(|(k, _)| k == &key) {
                    if prompt::validate_translation(src, &translation) {
                        accepted.insert(key.clone(), translation);
                    }
                }
            }
            // 未命中或被拒的条目进入重试（最后一次尝试后不再计数，保持进度为已译条目数）
            for (key, src) in &pending {
                if !accepted.contains_key(key) {
                    rejected.push((key.clone(), src.clone()));
                }
            }
            result.extend(accepted);
            pending = rejected;
        }
        on_batch_done(result.len() - before);
    }
    Ok(result)
}

/// 按语言源类型写回目标文件
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
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("unable to create target directory: {e}"))?;
    }
    let original = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("unable to read {}: {e}", source_path.display()))?;

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
    std::fs::write(&target_path, output)
        .map_err(|e| format!("unable to write {}: {e}", target_path.display()))
}
