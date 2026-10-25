//! 语言源发现：标准 lang 文件 + 路径含 `en_us` 的结构化 JSON / 自由文本（规格 §2）

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use super::super::jar;
use super::super::lang;
use super::super::types::{LanguageKind, LanguageSource};

/// 标准语言文件扩展名 → 解析方式
const STANDARD_EXTENSIONS: &[(&str, LanguageKind)] = &[
    ("json", LanguageKind::Json),
    ("lang", LanguageKind::KeyValue),
    ("properties", LanguageKind::KeyValue),
];

/// 自由文本扩展名
const FREE_TEXT_EXTENSIONS: &[&str] = &["txt", "md"];

fn read_entries(kind: LanguageKind, path: &Path) -> BTreeMap<String, String> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    match kind {
        LanguageKind::Json => lang::read_json_lang(&content).unwrap_or_default(),
        LanguageKind::KeyValue => lang::parse_keyvalue(&content).0.into_iter().collect(),
        _ => BTreeMap::new(),
    }
}

/// 发现 `assets/<ns>/lang/en_us.{json,lang,properties}`
pub fn find_standard_sources(workspace: &Path) -> Vec<LanguageSource> {
    let assets = workspace.join("assets");
    if !assets.is_dir() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&assets) else {
        return Vec::new();
    };

    let mut sources = Vec::new();
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let namespace = entry.file_name().to_string_lossy().to_string();
        let lang_dir = entry.path().join("lang");
        if !lang_dir.is_dir() {
            continue;
        }
        for (extension, kind) in STANDARD_EXTENSIONS {
            let file = lang_dir.join(format!("en_us.{extension}"));
            if !file.is_file() {
                continue;
            }
            sources.push(LanguageSource {
                kind: *kind,
                namespace: namespace.clone(),
                source_path: format!("assets/{namespace}/lang/en_us.{extension}"),
                target_path: format!("assets/{namespace}/lang/zh_cn.{extension}"),
                entries: read_entries(*kind, &file),
                existing_target: BTreeMap::new(),
            });
        }
    }
    sources
}

/// 路径段是否为 `en_us`（或 `en_us.*`）
fn is_english_segment(segment: &str) -> bool {
    segment == "en_us" || segment.starts_with("en_us.")
}

fn has_english_segment(relative: &str) -> bool {
    relative
        .to_ascii_lowercase()
        .split('/')
        .any(is_english_segment)
}

/// 把路径里的 `en_us` 换成 `zh_cn` 作为目标路径
fn target_path_of(relative: &str) -> String {
    let lower = relative.to_ascii_lowercase();
    match lower.rfind("en_us") {
        Some(position) => {
            let mut target = relative.to_string();
            target.replace_range(position..position + 5, "zh_cn");
            target
        }
        None => relative.to_string(),
    }
}

fn namespace_of(relative: &str) -> String {
    relative.split('/').next().unwrap_or("unknown").to_string()
}

/// 发现路径含 `en_us` 的结构化 JSON 与自由文本（排除标准 lang 源）
pub fn find_other_sources(workspace: &Path, standard: &HashSet<String>) -> Vec<LanguageSource> {
    let mut sources = Vec::new();

    for relative in jar::collect_files(workspace).unwrap_or_default() {
        let lower = relative.to_ascii_lowercase();
        if standard.contains(&lower) || !has_english_segment(&relative) {
            continue;
        }

        let path = workspace.join(&relative);
        let namespace = namespace_of(&relative);
        let target_path = target_path_of(&relative);

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
                target_path,
                entries,
                existing_target: BTreeMap::new(),
            });
            continue;
        }

        let is_free_text = FREE_TEXT_EXTENSIONS
            .iter()
            .any(|extension| lower.ends_with(&format!(".{extension}")));
        if !is_free_text {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let entries: BTreeMap<String, String> = content
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(index, line)| (index.to_string(), line.to_string()))
            .collect();
        if entries.is_empty() {
            continue;
        }
        sources.push(LanguageSource {
            kind: LanguageKind::FreeText,
            namespace,
            source_path: relative,
            target_path,
            entries,
            existing_target: BTreeMap::new(),
        });
    }

    sources
}
