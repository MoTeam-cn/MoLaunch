//! 覆盖诊断、已有中文检测、文件哈希与 token 报价

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::super::jar;
use super::super::lang;
use super::super::types::{ExistingChinese, LanguageKind, LanguageSource, Quote, ResourceCoverage};

/// 计入「文本资源」的扩展名
const TEXT_EXTENSIONS: &[&str] = &[
    "json",
    "lang",
    "properties",
    "txt",
    "md",
    "toml",
    "cfg",
    "xml",
];

/// 计算文件 SHA-256
pub fn file_hash(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("unable to read {}: {e}", path.display()))?;
    Ok(crate::utils::hash::sha256_hex(&bytes))
}

/// 单文件处置结论
struct Disposition {
    kind: &'static str,
    media_type: &'static str,
    target_path: Option<String>,
    candidates: u64,
    reason: &'static str,
}

fn extension_of(lower: &str) -> &str {
    lower.rsplit('.').next().unwrap_or("")
}

fn is_text_extension(extension: &str) -> bool {
    TEXT_EXTENSIONS.contains(&extension)
}

fn has_english_segment(lower: &str) -> bool {
    lower
        .split('/')
        .any(|segment| segment == "en_us" || segment.starts_with("en_us."))
}

fn media_type_of(lower: &str) -> &'static str {
    if lower.ends_with(".json") {
        "json"
    } else if lower.ends_with(".class") {
        "class"
    } else if is_text_extension(extension_of(lower)) {
        "text"
    } else {
        "binary"
    }
}

/// 资源覆盖诊断：每个工作区文件的处置结论
pub fn build_resource_coverage(
    workspace: &Path,
    sources: &[LanguageSource],
) -> Vec<ResourceCoverage> {
    let source_by_path: HashMap<String, &LanguageSource> = sources
        .iter()
        .map(|source| (source.source_path.to_ascii_lowercase(), source))
        .collect();
    let target_paths: HashSet<String> = sources
        .iter()
        .map(|source| source.target_path.to_ascii_lowercase())
        .collect();

    jar::collect_files(workspace)
        .unwrap_or_default()
        .into_iter()
        .map(|relative| {
            let lower = relative.to_ascii_lowercase();
            let disposition = classify(&lower, &source_by_path, &target_paths);
            ResourceCoverage {
                path: relative,
                media_type: disposition.media_type.to_string(),
                disposition: disposition.kind.to_string(),
                target_path: disposition.target_path,
                text_candidates: disposition.candidates,
                reason: disposition.reason.to_string(),
            }
        })
        .collect()
}

fn classify(
    lower: &str,
    source_by_path: &HashMap<String, &LanguageSource>,
    target_paths: &HashSet<String>,
) -> Disposition {
    if let Some(source) = source_by_path.get(lower) {
        let kind = if source.kind == LanguageKind::StructuredJson {
            "structured_source"
        } else {
            "standard_language"
        };
        return Disposition {
            kind,
            media_type: media_type_of(lower),
            target_path: Some(source.target_path.clone()),
            candidates: source.entries.len() as u64,
            reason: "已进入翻译工作图",
        };
    }
    if target_paths.contains(lower) {
        return Disposition {
            kind: "generated_target",
            media_type: "text",
            target_path: None,
            candidates: 0,
            reason: "现有中文镜像",
        };
    }
    if lower.ends_with(".class") {
        return Disposition {
            kind: "class_review",
            media_type: "class",
            target_path: None,
            candidates: 0,
            reason: "由 Class 常量扫描独立审计",
        };
    }

    let extension = extension_of(lower);
    let text = is_text_extension(extension);
    if text && has_english_segment(lower) {
        return Disposition {
            kind: "unknown",
            media_type: media_type_of(lower),
            target_path: None,
            candidates: 0,
            reason: "存在自然语言迹象但无镜像",
        };
    }
    Disposition {
        kind: "protected",
        media_type: media_type_of(lower),
        target_path: None,
        candidates: 0,
        reason: if text {
            "未发现高置信玩家文本"
        } else {
            "非文本资源保持原样"
        },
    }
}

/// token 报价预估（分析阶段展示，供用户评估成本）
pub fn quote_translation_metrics(
    language_entries: usize,
    language_chars: usize,
    class_candidates: usize,
    class_chars: usize,
) -> Quote {
    let language_batches = language_entries.div_ceil(40);
    let class_batches = class_candidates.div_ceil(16);

    // 每批请求固定携带 system prompt 与 user 模板（编译期内嵌，运行时零 IO）
    let prompt_chars = |name: &str| {
        crate::resources::read_resource(name)
            .map(|text| text.chars().count())
            .unwrap_or(0)
    };
    let per_batch_overhead = (prompt_chars("prompts/mod_translation.md")
        + prompt_chars("prompts/mod_translation_user.md")) as f64
        * 0.35;

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

    Quote {
        estimated_input_tokens,
        estimated_output_tokens,
        estimated_tokens,
        estimated_calls: (language_batches + class_batches) as u64,
        language_batches: language_batches as u64,
        class_batches: class_batches as u64,
        points,
        characters: (language_chars + class_chars) as u64,
        entries: (language_entries + class_candidates) as u64,
    }
}

/// 检测 JAR 内已有的中文语言文件（预检覆盖风险）
pub fn find_existing_chinese(workspace: &Path) -> Vec<ExistingChinese> {
    let mut found = Vec::new();

    for relative in jar::collect_files(workspace).unwrap_or_default() {
        let lower = relative.to_ascii_lowercase();
        let locale = if lower.contains("zh_cn") || lower.contains("zh-cn") {
            "zh_cn"
        } else if lower.contains("zh_tw") || lower.contains("zh-tw") {
            "zh_tw"
        } else {
            continue;
        };
        let is_language_file =
            lower.ends_with(".json") || lower.ends_with(".lang") || lower.ends_with(".properties");
        if !is_language_file {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(workspace.join(&relative)) else {
            continue;
        };
        let entries = if lower.ends_with(".json") {
            lang::read_json_lang(&content)
                .map(|map| map.len())
                .unwrap_or(0)
        } else {
            lang::parse_keyvalue(&content).0.len()
        };
        found.push(ExistingChinese {
            path: relative,
            locale: locale.to_string(),
            entries,
        });
    }

    found
}
