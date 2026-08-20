//! class 候选发现（跨文件聚合）与确定性排除

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::super::jar;
use super::super::types::ClassCandidate;
use super::classify::classify_class_text;
use super::pool::class_string_constants;

/// 遍历工作区 .class 文件，按文本跨文件聚合候选
pub fn discover_class_candidates(workspace: &Path) -> Vec<ClassCandidate> {
    let mut grouped: HashMap<String, (String, BTreeSet<String>)> = HashMap::new();
    for relative in jar::collect_files(workspace).unwrap_or_default() {
        if !relative.to_ascii_lowercase().ends_with(".class") {
            continue;
        }
        let Ok(bytes) = std::fs::read(workspace.join(&relative)) else {
            continue;
        };
        for text in class_string_constants(&bytes) {
            if !classify_class_text(&text).0 {
                continue;
            }
            let entry = grouped
                .entry(text.clone())
                .or_insert_with(|| (relative.clone(), BTreeSet::new()));
            entry.1.insert(relative.clone());
        }
    }
    let mut candidates: Vec<ClassCandidate> = grouped
        .into_iter()
        .map(|(text, (path, paths))| {
            let paths = paths.into_iter().collect::<Vec<_>>();
            ClassCandidate {
                id: candidate_id(&path, &text),
                path: path.clone(),
                paths: paths.clone(),
                occurrences: paths.len(),
                text,
            }
        })
        .collect();
    candidates.sort_by(|a, b| a.id.cmp(&b.id));
    candidates
}

fn candidate_id(path: &str, text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    hasher.update(b"\0");
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())[..24].to_string()
}

/// 确定性排除理由：java 类名 / 正则 / 内部诊断；UI 宿主路径保留
pub fn deterministic_class_exclusion_reason(path: &str, text: &str) -> Option<&'static str> {
    let value = text.trim();
    if regex::Regex::new(r"^(?:[a-z_][A-Za-z0-9_$]*\.)+[A-Z_$][A-Za-z0-9_$]*$")
        .unwrap()
        .is_match(value)
    {
        return Some("java_class_name");
    }
    let looks_like_regex = value.starts_with('^')
        && value.ends_with('$')
        && regex::Regex::new(r"(?:\\[.dDsSwW]|\[[^\]]+\]|\{\d+(?:,\d*)?\}|\(\?:?)")
            .unwrap()
            .is_match(value);
    if looks_like_regex {
        return Some("regular_expression");
    }
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    if regex::Regex::new(r"(?:^|/)(?:gui|screen|widget|tooltip|chat|menu|config)(?:/|$)")
        .unwrap()
        .is_match(&normalized)
    {
        return None;
    }
    let diagnostic_host = regex::Regex::new(
        r"(?:^|/)(?:file|graphics|server|palette|world|misc|region|pool|core)(?:/|$)",
    )
    .unwrap()
    .is_match(&normalized);
    let diagnostic_text =
        regex::Regex::new(r"(?i)\b(?:ioexception|failed to|retrying|unknown status)\b")
            .unwrap()
            .is_match(value);
    if diagnostic_host && diagnostic_text {
        return Some("internal_diagnostic");
    }
    None
}
