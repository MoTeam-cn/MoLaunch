//! class 候选发现与确定性排除（规格 §3.3 §3.5 §3.6）
//!
//! 遍历工作区 `.class` 文件，把通过判定的文本按「文本 → 出现路径集合」聚合，
//! 再按 sha256(首个路径 + NUL + 文本) 前 24 位生成稳定 id，最后按 id 排序。

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::super::jar;
use super::super::types::ClassCandidate;
use super::classify::classify_class_text;
use super::pool::class_string_constants;

/// 聚合结果：文本 → (首个路径, 全部路径)
type Grouped = BTreeMap<String, (String, BTreeSet<String>)>;

pub fn discover_class_candidates(workspace: &Path) -> Vec<ClassCandidate> {
    let mut grouped: Grouped = BTreeMap::new();

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
                .entry(text)
                .or_insert_with(|| (relative.clone(), BTreeSet::new()));
            entry.1.insert(relative.clone());
        }
    }

    let mut candidates: Vec<ClassCandidate> = grouped
        .into_iter()
        .map(|(text, (path, paths))| {
            let paths: Vec<String> = paths.into_iter().collect();
            ClassCandidate {
                id: candidate_id(&path, &text),
                path: path.clone(),
                occurrences: paths.len(),
                paths,
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
    let digest = format!("{:x}", hasher.finalize());
    digest[..24].to_string()
}

/// 宿主路径里出现这些目录名时视为界面文案（不排除）
const UI_SEGMENTS: [&str; 7] = [
    "gui", "screen", "widget", "tooltip", "chat", "menu", "config",
];
/// 宿主路径里出现这些目录名时视为内部诊断宿主
const DIAGNOSTIC_SEGMENTS: [&str; 8] = [
    "file", "graphics", "server", "palette", "world", "misc", "region", "pool",
];
/// 诊断文案特征（小写匹配）
const DIAGNOSTIC_PHRASES: [&str; 4] = ["ioexception", "failed to", "retrying", "unknown status"];

/// 确定性排除理由：java 类名 / 正则 / 内部诊断；界面宿主路径一律保留
pub fn deterministic_class_exclusion_reason(path: &str, text: &str) -> Option<&'static str> {
    let value = text.trim();
    if is_java_class_name(value) {
        return Some("java_class_name");
    }
    if looks_like_regex(value) {
        return Some("regular_expression");
    }

    let segments: Vec<String> = path
        .replace('\\', "/")
        .to_ascii_lowercase()
        .split('/')
        .map(str::to_string)
        .collect();
    if segments
        .iter()
        .any(|segment| UI_SEGMENTS.contains(&segment.as_str()))
    {
        return None;
    }

    let diagnostic_host = segments
        .iter()
        .any(|segment| DIAGNOSTIC_SEGMENTS.contains(&segment.as_str()));
    if diagnostic_host {
        let lowered = value.to_ascii_lowercase();
        if DIAGNOSTIC_PHRASES
            .iter()
            .any(|phrase| lowered.contains(phrase))
        {
            return Some("internal_diagnostic");
        }
    }
    None
}

/// 全限定类名：`com.example.Foo` / `com.example.Foo$Bar`
fn is_java_class_name(value: &str) -> bool {
    let segments: Vec<&str> = value.split('.').collect();
    if segments.len() < 2 {
        return false;
    }
    let Some((last, head)) = segments.split_last() else {
        return false;
    };
    let tail_ok = last
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase() || c == '$')
        .unwrap_or(false)
        && last
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$'));
    tail_ok
        && head.iter().all(|segment| {
            segment
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase() || c == '_')
                .unwrap_or(false)
                && segment
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$'))
        })
}

/// 正则字面量：以 `^` 开头、`$` 结尾且含转义类/字符组/量词/分组
fn looks_like_regex(value: &str) -> bool {
    if !(value.starts_with('^') && value.ends_with('$')) {
        return false;
    }
    let mut chars = value.chars().peekable();
    while let Some(current) = chars.next() {
        match current {
            '\\' => {
                if let Some(next) = chars.peek() {
                    if "dDsSwW.".contains(*next) {
                        return true;
                    }
                }
            }
            '[' => return true,
            '(' => return true,
            '{' => return true,
            _ => {}
        }
    }
    false
}
