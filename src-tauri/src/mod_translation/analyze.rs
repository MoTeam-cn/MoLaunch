//! 模组翻译：JAR 分析（语言源发现 + 加载器探测）

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::lang;
use super::types::{JarInspection, LanguageKind, LanguageSource, Loader};

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
            let target_path = format!("assets/{ns_name}/lang/zh_cn.{ext}");
            let kind = if ext == "json" {
                LanguageKind::Json
            } else {
                LanguageKind::KeyValue
            };
            sources.push(LanguageSource {
                kind,
                namespace: ns_name.clone(),
                source_path: source_rel,
                target_path,
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
    for relative in super::jar::collect_files(workspace).unwrap_or_default() {
        let lower = relative.to_ascii_lowercase();
        if standard.contains(&lower) {
            continue;
        }
        let has_en_us = lower
            .split('/')
            .any(|segment| segment == "en_us" || segment.starts_with("en_us."));
        if !has_en_us {
            continue;
        }
        let path = workspace.join(&relative);
        if lower.ends_with(".json") {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(entries) = lang::collect_structured_strings(&content) {
                let mut target = relative.clone();
                if let Some(pos) = lower.rfind("en_us") {
                    target.replace_range(pos..pos + 5, "zh_cn");
                }
                sources.push(LanguageSource {
                    kind: LanguageKind::StructuredJson,
                    namespace: namespace_of(&relative),
                    source_path: relative.clone(),
                    target_path: target,
                    entries,
                    existing_target: BTreeMap::new(),
                });
            }
        } else if lower.ends_with(".txt") || lower.ends_with(".md") {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
            let entries: BTreeMap<String, String> = lines
                .iter()
                .enumerate()
                .filter(|(_, line)| !line.trim().is_empty())
                .map(|(i, line)| (i.to_string(), line.clone()))
                .collect();
            if entries.is_empty() {
                continue;
            }
            let mut target = relative.clone();
            if let Some(pos) = lower.rfind("en_us") {
                target.replace_range(pos..pos + 5, "zh_cn");
            }
            sources.push(LanguageSource {
                kind: LanguageKind::FreeText,
                namespace: namespace_of(&relative),
                source_path: relative.clone(),
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

/// 探测加载器：fabric.mod.json / META-INF/mods.toml / META-INF/neoforge.mods.toml / mcmod.info
pub fn detect_loader(workspace: &Path) -> (Loader, Vec<String>, Vec<String>) {
    if workspace.join("fabric.mod.json").is_file() {
        let mod_id = read_fabric_mod_id(&workspace.join("fabric.mod.json"));
        return (Loader::Fabric, mod_id.into_iter().collect(), Vec::new());
    }
    if workspace.join("META-INF/neoforge.mods.toml").is_file() {
        let ids = read_mods_toml_ids(&workspace.join("META-INF/neoforge.mods.toml"));
        return (Loader::NeoForge, ids, Vec::new());
    }
    if workspace.join("META-INF/mods.toml").is_file() {
        let ids = read_mods_toml_ids(&workspace.join("META-INF/mods.toml"));
        return (Loader::Forge, ids, Vec::new());
    }
    (Loader::Unknown, Vec::new(), Vec::new())
}

fn read_fabric_mod_id(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    value
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// 从 mods.toml 提取 modId（TOML 简易解析：`modId = "xxx"`）
fn read_mods_toml_ids(path: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("modId=") {
            let id = rest.trim().trim_matches('"').to_string();
            if !id.is_empty() {
                ids.push(id);
            }
        }
    }
    ids
}

/// 计算文件 SHA-256（用于工作区命名/去重）
pub fn file_hash(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("unable to read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// 汇总 JAR 分析结果（signed 由解包阶段传入）
pub fn inspect_jar(workspace: &Path, input_path: &Path, signed: bool) -> JarInspection {
    let standard = find_standard_sources(workspace);
    let standard_set: HashSet<String> = standard
        .iter()
        .map(|s| s.source_path.to_ascii_lowercase())
        .collect();
    let mut language_sources = standard;
    language_sources.extend(find_other_sources(workspace, &standard_set));
    let (loader, mod_ids, _) = detect_loader(workspace);
    let language_entries = language_sources.iter().map(|s| s.entries.len()).sum();
    let mut warnings = Vec::new();
    if language_sources.is_empty() {
        warnings.push("未找到 en_us 语言文件或含 en_us 路径的文本".to_string());
    }
    if signed {
        warnings.push("JAR 含签名文件，翻译重打包后签名将失效".to_string());
    }
    JarInspection {
        input_path: input_path.to_path_buf(),
        original_filename: input_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        loader,
        mod_ids,
        signed,
        language_sources,
        language_entries,
        warnings,
    }
}
