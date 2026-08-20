//! 语言源发现：标准 lang 文件与含 en_us 路径的结构化/自由文本

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use super::super::jar;
use super::super::lang;
use super::super::types::{LanguageKind, LanguageSource};

/// 发现标准语言文件 `assets/<ns>/lang/en_us.{json,lang,properties}`
pub fn find_standard_sources(workspace: &Path) -> Vec<LanguageSource> {
    let mut sources = Vec::new();
    let assets = workspace.join("assets");
    if !assets.is_dir() {
        return sources;
    }
    let Ok(namespaces) = std::fs::read_dir(&assets) else {
        return sources;
    };
    for ns in namespaces.flatten() {
        let ns_path = ns.path();
        if !ns_path.is_dir() {
            continue;
        }
        let ns_name = ns.file_name().to_string_lossy().to_string();
        let lang_dir = ns_path.join("lang");
        if !lang_dir.is_dir() {
            continue;
        }
        for ext in ["json", "lang", "properties"] {
            let source_rel = format!("assets/{ns_name}/lang/en_us.{ext}");
            let source_path = lang_dir.join(format!("en_us.{ext}"));
            if !source_path.is_file() {
                continue;
            }
            let kind = if ext == "json" {
                LanguageKind::Json
            } else {
                LanguageKind::KeyValue
            };
            sources.push(LanguageSource {
                kind,
                namespace: ns_name.clone(),
                source_path: source_rel,
                target_path: format!("assets/{ns_name}/lang/zh_cn.{ext}"),
                entries: read_source_entries(kind, &source_path),
                existing_target: BTreeMap::new(),
            });
        }
    }
    sources
}

fn read_source_entries(kind: LanguageKind, path: &Path) -> BTreeMap<String, String> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    match kind {
        LanguageKind::Json => lang::read_json_lang(&content).unwrap_or_default(),
        LanguageKind::KeyValue => lang::parse_keyvalue(&content).0.into_iter().collect(),
        _ => BTreeMap::new(),
    }
}

/// 发现路径含 `en_us` 的结构化 JSON / 自由文本（排除标准 lang 源，避免重复捕获）
pub fn find_other_sources(workspace: &Path, standard: &HashSet<String>) -> Vec<LanguageSource> {
    let mut sources = Vec::new();
    for relative in jar::collect_files(workspace).unwrap_or_default() {
        let lower = relative.to_ascii_lowercase();
        if standard.contains(&lower)
            || !lower
                .split('/')
                .any(|segment| segment == "en_us" || segment.starts_with("en_us."))
        {
            continue;
        }
        let path = workspace.join(&relative);
        let mut target = relative.clone();
        if let Some(pos) = lower.rfind("en_us") {
            target.replace_range(pos..pos + 5, "zh_cn");
        }
        let namespace = namespace_of(&relative);
        if lower.ends_with(".json") {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(entries) = lang::collect_structured_strings(&content) else {
                continue;
            };
            sources.push(LanguageSource {
                kind: LanguageKind::StructuredJson,
                namespace,
                source_path: relative,
                target_path: target,
                entries,
                existing_target: BTreeMap::new(),
            });
        } else if lower.ends_with(".txt") || lower.ends_with(".md") {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let entries: BTreeMap<String, String> = content
                .lines()
                .enumerate()
                .filter(|(_, line)| !line.trim().is_empty())
                .map(|(i, line)| (i.to_string(), line.to_string()))
                .collect();
            if entries.is_empty() {
                continue;
            }
            sources.push(LanguageSource {
                kind: LanguageKind::FreeText,
                namespace,
                source_path: relative,
                target_path: target,
                entries,
                existing_target: BTreeMap::new(),
            });
        }
    }
    sources
}

fn namespace_of(relative: &str) -> String {
    relative.split('/').next().unwrap_or("unknown").to_string()
}
