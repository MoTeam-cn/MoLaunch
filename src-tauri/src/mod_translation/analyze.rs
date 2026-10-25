//! 模组翻译：JAR 分析（语言源发现 + 加载器探测 + class 候选 + 报价）

mod coverage;
mod loader;
mod sources;

pub use coverage::{
    build_resource_coverage, file_hash, find_existing_chinese, quote_translation_metrics,
};
pub use loader::{detect_loader, detect_metadata};
pub use sources::{find_other_sources, find_standard_sources};

use std::collections::HashSet;
use std::path::Path;

use super::class;
use super::types::JarInspection;

/// 汇总 JAR 分析结果（signed 由解包阶段传入）
pub fn inspect_jar(workspace: &Path, input_path: &Path, signed: bool) -> JarInspection {
    let standard = find_standard_sources(workspace);
    let standard_paths: HashSet<String> = standard
        .iter()
        .map(|source| source.source_path.to_ascii_lowercase())
        .collect();

    let mut language_sources = standard;
    language_sources.extend(find_other_sources(workspace, &standard_paths));

    let (loader, mod_ids, project_names, version) = detect_metadata(workspace);

    let language_entries: usize = language_sources.iter().map(|s| s.entries.len()).sum();
    let required_entries: usize = language_sources.iter().map(|s| s.required_count()).sum();
    let language_chars: usize = language_sources
        .iter()
        .flat_map(|source| source.entries.values())
        .map(|value| value.chars().count())
        .sum();

    let class_candidates = class::discover_class_candidates(workspace);
    let class_chars: usize = class_candidates
        .iter()
        .map(|candidate| candidate.text.chars().count())
        .sum();

    let coverage = build_resource_coverage(workspace, &language_sources);
    let existing_chinese = find_existing_chinese(workspace);
    let quote = quote_translation_metrics(
        required_entries,
        language_chars,
        class_candidates.len(),
        class_chars,
    );

    let warnings = collect_warnings(
        language_sources.is_empty(),
        signed,
        class_candidates.len(),
        coverage.iter().any(|entry| entry.disposition == "unknown"),
    );

    JarInspection {
        input_path: input_path.to_path_buf(),
        original_filename: input_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        loader,
        mod_ids,
        project_names,
        version,
        signed,
        language_sources,
        language_entries,
        class_candidates,
        coverage,
        quote,
        mod_name: None,
        existing_chinese,
        warnings,
    }
}

/// 分析阶段的提示信息
fn collect_warnings(
    no_language_source: bool,
    signed: bool,
    class_candidates: usize,
    uncovered_text: bool,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if no_language_source {
        warnings.push("未找到 en_us 语言文件或含 en_us 路径的文本".to_string());
    }
    if signed {
        warnings.push("JAR 含签名文件，翻译重打包后签名将失效".to_string());
    }
    if class_candidates > 0 {
        warnings.push(format!("发现 {class_candidates} 个 class 常量池文本候选"));
    }
    if uncovered_text {
        warnings.push("存在未覆盖的文本资源".to_string());
    }
    warnings
}
