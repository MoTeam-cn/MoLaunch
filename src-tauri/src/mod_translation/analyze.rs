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
    let standard_set: HashSet<String> = standard
        .iter()
        .map(|s| s.source_path.to_ascii_lowercase())
        .collect();
    let mut language_sources = standard;
    language_sources.extend(find_other_sources(workspace, &standard_set));
    let (loader, mod_ids, project_names, version) = detect_metadata(workspace);
    let language_entries = language_sources.iter().map(|s| s.entries.len()).sum();
    let required_entries = language_sources.iter().map(|s| s.required_count()).sum();
    let language_chars: usize = language_sources
        .iter()
        .flat_map(|s| s.entries.values())
        .map(|v| v.chars().count())
        .sum();
    let class_candidates = class::discover_class_candidates(workspace);
    let class_chars: usize = class_candidates
        .iter()
        .map(|c| c.text.chars().count())
        .sum();
    let coverage = build_resource_coverage(workspace, &language_sources);
    let existing_chinese = find_existing_chinese(workspace);
    let quote = quote_translation_metrics(
        required_entries,
        language_chars,
        class_candidates.len(),
        class_chars,
    );
    let mut warnings = Vec::new();
    if language_sources.is_empty() {
        warnings.push("未找到 en_us 语言文件或含 en_us 路径的文本".to_string());
    }
    if signed {
        warnings.push("JAR 含签名文件，翻译重打包后签名将失效".to_string());
    }
    if !class_candidates.is_empty() {
        warnings.push(format!(
            "发现 {} 个 class 常量池文本候选",
            class_candidates.len()
        ));
    }
    if coverage.iter().any(|c| c.disposition == "unknown") {
        warnings.push("存在未覆盖的文本资源".to_string());
    }
    JarInspection {
        input_path: input_path.to_path_buf(),
        original_filename: input_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
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
