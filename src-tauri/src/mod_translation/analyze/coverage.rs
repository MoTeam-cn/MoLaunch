//! 覆盖诊断、已有中文检测、文件哈希与 token 报价

use std::collections::{HashMap, HashSet};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::super::jar;
use super::super::lang;
use super::super::types::{ExistingChinese, LanguageKind, LanguageSource, Quote, ResourceCoverage};

/// 计算文件 SHA-256（用于工作区命名/去重）
pub fn file_hash(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("unable to read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// 资源覆盖诊断：每个工作区文件的处置结论
pub fn build_resource_coverage(
    workspace: &Path,
    sources: &[LanguageSource],
) -> Vec<ResourceCoverage> {
    let source_by_path: HashMap<String, &LanguageSource> = sources
        .iter()
        .map(|s| (s.source_path.to_ascii_lowercase(), s))
        .collect();
    let target_paths: HashSet<String> = sources
        .iter()
        .map(|s| s.target_path.to_ascii_lowercase())
        .collect();
    let mut coverage = Vec::new();
    for relative in jar::collect_files(workspace).unwrap_or_default() {
        let lower = relative.to_ascii_lowercase();
        let (disposition, media_type, target, candidates, reason) =
            if let Some(source) = source_by_path.get(&lower) {
                (
                    if source.kind == LanguageKind::StructuredJson {
                        "structured_source"
                    } else {
                        "standard_language"
                    },
                    if lower.ends_with(".json") {
                        "json"
                    } else {
                        "text"
                    },
                    Some(source.target_path.clone()),
                    source.entries.len() as u64,
                    "已进入翻译工作图",
                )
            } else if target_paths.contains(&lower) {
                ("generated_target", "text", None, 0, "现有中文镜像")
            } else if lower.ends_with(".class") {
                (
                    "class_review",
                    "class",
                    None,
                    0,
                    "由 Class 常量扫描独立审计",
                )
            } else {
                let extension = lower.rsplit('.').next().unwrap_or("");
                let is_text = matches!(
                    extension,
                    "json" | "lang" | "properties" | "txt" | "md" | "toml" | "cfg" | "xml"
                );
                let has_en_us = lower
                    .split('/')
                    .any(|segment| segment == "en_us" || segment.starts_with("en_us."));
                if is_text && has_en_us {
                    (
                        "unknown",
                        if extension == "json" { "json" } else { "text" },
                        None,
                        0,
                        "存在自然语言迹象但无镜像",
                    )
                } else {
                    (
                        "protected",
                        if is_text { "text" } else { "binary" },
                        None,
                        0,
                        if is_text {
                            "未发现高置信玩家文本"
                        } else {
                            "非文本资源保持原样"
                        },
                    )
                }
            };
        coverage.push(ResourceCoverage {
            path: relative,
            media_type: media_type.to_string(),
            disposition: disposition.to_string(),
            target_path: target,
            text_candidates: candidates,
            reason: reason.to_string(),
        });
    }
    coverage
}

/// token 报价预估（分析阶段展示，供用户评估成本）
///
/// 输入 token = 每批固定开销（system prompt + user 模板，编译期内嵌读取）
/// + 条目字符 token；输出 token = 译文字符 token + class 译文 token。
pub fn quote_translation_metrics(
    language_entries: usize,
    language_chars: usize,
    class_candidates: usize,
    class_chars: usize,
) -> Quote {
    let language_batches = language_entries.div_ceil(40);
    let class_batches = class_candidates.div_ceil(16);
    // 每批请求固定携带 system prompt 与 user 模板（resources 编译期内嵌，运行时零 IO）
    let system_prompt_chars = crate::resources::read_resource("prompts/mod_translation.md")
        .map(|s| s.chars().count())
        .unwrap_or(0);
    let user_template_chars = crate::resources::read_resource("prompts/mod_translation_user.md")
        .map(|s| s.chars().count())
        .unwrap_or(0);
    let per_batch_overhead = (system_prompt_chars + user_template_chars) as f64 * 0.35;
    let points = 10
        + (language_batches + class_batches) as u64 * 2
        + ((language_chars + class_chars) as u64).div_ceil(1_000);
    let estimated_input_tokens = (language_chars as f64 * 0.35
        + language_batches as f64 * (600.0 + per_batch_overhead))
        as u64
        + (class_chars as f64 * 0.3 + class_batches as f64 * (500.0 + per_batch_overhead)) as u64;
    let estimated_output_tokens =
        (language_chars as f64 * 0.6) as u64 + class_candidates as u64 * 40;
    let estimated_tokens = ((estimated_input_tokens + estimated_output_tokens) as f64 * 1.2) as u64;
    let estimated_calls = language_batches as u64 + class_batches as u64;
    Quote {
        estimated_input_tokens,
        estimated_output_tokens,
        estimated_tokens,
        estimated_calls,
        language_batches: language_batches as u64,
        class_batches: class_batches as u64,
        points,
        characters: (language_chars + class_chars) as u64,
        entries: (language_entries + class_candidates) as u64,
    }
}

/// 检测 JAR 内已有的中文语言文件（zh_cn / zh_tw 等），供预检提示覆盖风险
pub fn find_existing_chinese(workspace: &Path) -> Vec<ExistingChinese> {
    let mut result = Vec::new();
    for relative in jar::collect_files(workspace).unwrap_or_default() {
        let lower = relative.to_ascii_lowercase();
        let locale = if lower.contains("zh_cn") || lower.contains("zh-cn") {
            "zh_cn"
        } else if lower.contains("zh_tw") || lower.contains("zh-tw") {
            "zh_tw"
        } else {
            continue;
        };
        if !(lower.ends_with(".json") || lower.ends_with(".lang") || lower.ends_with(".properties"))
        {
            continue;
        }
        let path = workspace.join(&relative);
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let entries = if lower.ends_with(".json") {
            lang::read_json_lang(&content).map(|m| m.len()).unwrap_or(0)
        } else {
            lang::parse_keyvalue(&content).0.len()
        };
        result.push(ExistingChinese {
            path: relative,
            locale: locale.to_string(),
            entries,
        });
    }
    result
}
