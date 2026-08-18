//! 模组翻译：JAR 分析（语言源发现 + 加载器探测 + class 候选 + 报价）

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::class;
use super::lang;
use super::types::{JarInspection, LanguageKind, LanguageSource, Loader, Quote, ResourceCoverage};

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
    for relative in super::jar::collect_files(workspace).unwrap_or_default() {
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

/// 探测元数据：加载器 + modId + 项目名 + 版本（内部复用 detect_loader）
pub fn detect_metadata(workspace: &Path) -> (Loader, Vec<String>, Vec<String>, Option<String>) {
    let (loader, mod_ids, _) = detect_loader(workspace);
    match loader {
        Loader::Fabric => {
            let Ok(content) = std::fs::read_to_string(workspace.join("fabric.mod.json")) else {
                return (loader, mod_ids, Vec::new(), None);
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
                return (loader, mod_ids, Vec::new(), None);
            };
            let project_names = value
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .into_iter()
                .collect();
            let version = value
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (loader, mod_ids, project_names, version)
        }
        Loader::NeoForge | Loader::Forge => {
            let path = if loader == Loader::NeoForge {
                workspace.join("META-INF/neoforge.mods.toml")
            } else {
                workspace.join("META-INF/mods.toml")
            };
            let Ok(content) = std::fs::read_to_string(path) else {
                return (loader, mod_ids, Vec::new(), None);
            };
            let project_names = extract_toml_values(&content, "displayName");
            let version = extract_toml_values(&content, "version").into_iter().next();
            (loader, mod_ids, project_names, version)
        }
        Loader::Unknown => (loader, mod_ids, Vec::new(), None),
    }
}

/// 正则提取 TOML 键值（支持双引号与单引号）
fn extract_toml_values(text: &str, key: &str) -> Vec<String> {
    let pattern = format!(r#"\b{key}\s*=\s*(?:"([^"]+)"|'([^']+)')"#);
    let Ok(regex) = regex::Regex::new(&pattern) else {
        return Vec::new();
    };
    regex
        .captures_iter(text)
        .filter_map(|caps| {
            caps.get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string())
        })
        .collect()
}

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
    for relative in super::jar::collect_files(workspace).unwrap_or_default() {
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
        warnings,
    }
}
