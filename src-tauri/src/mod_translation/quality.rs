//! 模组翻译：占位符保护、机械 / 语义审计、待译判定、工作量权重（规格 §5.4）
//!
//! 全部实现走字符扫描，不依赖正则：占位符语法由规格表格固定，
//! 术语表与透传规则用「词边界短语匹配」表达。

use std::collections::{BTreeMap, HashSet};

use super::types::has_chinese;

/// 提取文本中的保护占位符（排序后返回，便于直接比较集合）
pub fn extract_protected_tokens(text: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    tokens.extend(scan_printf(text));
    tokens.extend(scan_braces(text));
    tokens.extend(scan_format_codes(text));
    tokens.extend(scan_escapes(text));
    tokens.extend(scan_controls(text));
    tokens.sort();
    tokens
}

fn is_printf_flag(ch: char) -> bool {
    matches!(
        ch,
        '-' | '+' | '#' | '0' | ' ' | ',' | '(' | ')' | '<' | '\'' | '1'..='9'
    )
}

/// printf 占位符：`%[flags][.width]conversion`
fn scan_printf(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '%' {
            index += 1;
            continue;
        }
        let start = index;
        let mut cursor = index + 1;
        while cursor < chars.len() && (is_printf_flag(chars[cursor]) || chars[cursor] == '.') {
            cursor += 1;
        }
        if cursor < chars.len() && chars[cursor] == DOLLAR_SIGN {
            cursor += 1;
        }
        if cursor >= chars.len() {
            break;
        }
        let conversion = chars[cursor];
        if !conversion.is_ascii_alphabetic() && conversion != '%' {
            index = start + 1;
            continue;
        }
        cursor += 1;
        if cursor < chars.len() && chars[cursor].is_ascii_alphanumeric() {
            cursor += 1;
        }
        found.push(chars[start..cursor].iter().collect());
        index = cursor;
    }
    found
}

/// 位置参数前缀字符
const DOLLAR_SIGN: char = '$';

/// 花括号占位符：`{{name}}` / 美元花括号 / `{0}` / `{0:format}`
fn scan_braces(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let start = index;
        let token: Option<usize> = match chars[index] {
            '{' if chars.get(index + 1) == Some(&'{') => {
                closing_from(&chars, index + 2, "}}").map(|end| end + 2)
            }
            '{' if chars.get(index + 1).is_some_and(|c| c.is_ascii_digit()) => {
                closing_from(&chars, index + 1, "}").map(|end| end + 1)
            }
            '$' if chars.get(index + 1) == Some(&'{') => {
                closing_from(&chars, index + 2, "}").map(|end| end + 1)
            }
            _ => None,
        };
        match token {
            Some(end) => {
                found.push(chars[start..end].iter().collect());
                index = end;
            }
            None => index += 1,
        }
    }
    found
}

/// 从 `from` 起找终止串，返回其起始下标
fn closing_from(chars: &[char], from: usize, closing: &str) -> Option<usize> {
    let pattern: Vec<char> = closing.chars().collect();
    let mut cursor = from;
    while cursor + pattern.len() <= chars.len() {
        if chars[cursor..cursor + pattern.len()] == pattern[..] {
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

/// 颜色 / 格式码：`§` + 单字符
fn scan_format_codes(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index + 1 < chars.len() {
        let is_code = chars[index] == '§'
            && (chars[index + 1].is_ascii_hexdigit()
                || matches!(chars[index + 1], 'k'..='o' | 'r' | 'x'));
        if is_code {
            found.push(chars[index..index + 2].iter().collect());
            index += 2;
            continue;
        }
        index += 1;
    }
    found
}

/// 转义字面量：换行 / 制表 / 回车
fn scan_escapes(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index + 1 < chars.len() {
        if chars[index] == '\\' && matches!(chars[index + 1], 'n' | 't' | 'r') {
            found.push(chars[index..index + 2].iter().collect());
            index += 2;
            continue;
        }
        index += 1;
    }
    found
}

/// 不可见控制字符
fn scan_controls(text: &str) -> Vec<String> {
    text.chars()
        .filter(|c| {
            matches!(
                *c,
                '\u{0001}'..='\u{0008}' | '\u{000b}' | '\u{000c}' | '\u{000e}'..='\u{001f}'
            )
        })
        .map(|c| c.to_string())
        .collect()
}

/// 占位符集合不一致时返回错误描述
pub fn validate_protected_tokens(source: &str, translation: &str) -> Option<String> {
    let expected = extract_protected_tokens(source);
    let actual = extract_protected_tokens(translation);
    if expected == actual {
        return None;
    }
    Some(format!(
        "占位符不一致：期望 [{}]，实际 [{}]",
        expected.join(", "),
        actual.join(", ")
    ))
}

/// 按源文本是否含转义字面量，统一换行转义的方向
pub fn normalize_model_translation(source: &str, translation: &str) -> String {
    let mut normalized = translation.to_string();
    for (escaped, control) in [("\\n", "\n"), ("\\r", "\r"), ("\\t", "\t")] {
        if source.contains(escaped) {
            if !normalized.contains(escaped) {
                normalized = normalized.replace(control, escaped);
            }
        } else {
            normalized = normalized.replace(escaped, control);
        }
    }
    normalized
}

/// 审计严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditSeverity {
    Error,
    Warning,
}

/// 单条审计结果
#[derive(Debug, Clone)]
pub struct AuditIssue {
    pub severity: AuditSeverity,
    pub key: String,
    pub message: String,
}

fn error(key: &str, message: impl Into<String>) -> AuditIssue {
    AuditIssue {
        severity: AuditSeverity::Error,
        key: key.to_string(),
        message: message.into(),
    }
}

fn warning(key: &str, message: impl Into<String>) -> AuditIssue {
    AuditIssue {
        severity: AuditSeverity::Warning,
        key: key.to_string(),
        message: message.into(),
    }
}

/// 机械不变量审计：缺译 / 占位符不一致 / 译文无中文（Error），多余键（Warning）
pub fn audit_invariants(
    source: &BTreeMap<String, String>,
    target: &BTreeMap<String, String>,
) -> Vec<AuditIssue> {
    let mut issues = Vec::new();
    for (key, english) in source {
        let translated = target.get(key).map(|value| value.trim()).unwrap_or("");
        if translated.is_empty() {
            issues.push(error(key, "缺少中文译文"));
            continue;
        }
        if let Some(problem) = validate_protected_tokens(english, translated) {
            issues.push(error(key, problem));
        }
        if requires_work(key, english, Some(translated)) && !has_chinese(translated) {
            issues.push(error(
                key,
                "译文不含简体中文；若必须保留原文，需要显式标记 keep-source",
            ));
        }
    }
    for key in target.keys() {
        if !source.contains_key(key) {
            issues.push(warning(key, "中文文件包含源语言中不存在的额外条目"));
        }
    }
    issues
}

/// 官方术语表：(可接受写法, 译文必须包含, 提示语)
const OFFICIAL_TERMS: &[(&[&str], &str, &str)] = &[
    (
        &["Iron Bars"],
        "铁栏杆",
        "Iron Bars 应沿用官方简中术语“铁栏杆”",
    ),
    (
        &["End Brick", "Ender Brick"],
        "末地石砖",
        "End Brick 应沿用官方材料名“末地石砖”",
    ),
    (&["Crimson"], "绯红", "Crimson 应沿用官方译名“绯红”"),
    (&["Warped"], "诡异", "Warped 应沿用官方译名“诡异”"),
    (
        &["Pale Oak"],
        "苍白橡木",
        "Pale Oak 应沿用官方译名“苍白橡木”",
    ),
    (
        &["Dark Oak"],
        "深色橡木",
        "Dark Oak 应沿用官方译名“深色橡木”",
    ),
    (
        &["Oak Planks"],
        "橡木木板",
        "Oak Planks 应沿用完整材料名“橡木木板”",
    ),
];

/// 词边界短语匹配
fn contains_phrase(text: &str, phrase: &str) -> bool {
    let mut from = 0;
    while let Some(found) = text[from..].find(phrase) {
        let start = from + found;
        let end = start + phrase.len();
        let before_ok = text[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        let after_ok = text[end..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        from = end;
    }
    false
}

/// 语义审计：官方术语强制（Error）、同英文多译法（Warning）
pub fn audit_semantic(
    source: &BTreeMap<String, String>,
    target: &BTreeMap<String, String>,
) -> Vec<AuditIssue> {
    let mut issues = Vec::new();
    let mut groups: BTreeMap<&str, Vec<(&String, &String)>> = BTreeMap::new();

    for (key, english) in source {
        let Some(chinese) = target.get(key) else {
            continue;
        };
        if chinese.trim().is_empty() {
            continue;
        }
        for (variants, required, label) in OFFICIAL_TERMS {
            let matched = variants
                .iter()
                .any(|variant| contains_phrase(english, variant));
            if matched && !chinese.contains(required) {
                issues.push(error(key, *label));
            }
        }
        groups
            .entry(english.as_str())
            .or_default()
            .push((key, chinese));
    }

    for group in groups.values() {
        if group.len() <= 1 {
            continue;
        }
        let variants: HashSet<&String> = group.iter().map(|(_, chinese)| *chinese).collect();
        if variants.len() <= 1 {
            continue;
        }
        for (key, _) in group {
            issues.push(warning(
                key,
                format!(
                    "相同英文原文出现 {} 种译法，请确认是否需要统一",
                    variants.len()
                ),
            ));
        }
    }
    issues
}

/// 透传判定：URL / 命令 / 命名空间 ID
fn is_passthrough_text(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    let is_url = ["http://", "https://"]
        .iter()
        .any(|prefix| lowered.starts_with(prefix))
        && !text.contains(char::is_whitespace);
    is_url || is_namespaced_id(text) || is_plain_command(text)
}

fn is_namespaced_id(text: &str) -> bool {
    let Some((namespace, path)) = text.split_once(':') else {
        return false;
    };
    let allowed = |c: char| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '/' | '-')
    };
    !namespace.is_empty()
        && !path.is_empty()
        && namespace.chars().all(allowed)
        && path.chars().all(allowed)
}

fn is_plain_command(text: &str) -> bool {
    if !text.starts_with('/') {
        return false;
    }
    text.split_whitespace().all(|token| {
        token.chars().all(|c| {
            c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || matches!(
                    c,
                    '_' | '.'
                        | ':'
                        | '-'
                        | '@'
                        | '~'
                        | '^'
                        | '<'
                        | '>'
                        | '{'
                        | '}'
                        | '['
                        | ']'
                        | '*'
                )
        })
    })
}

/// 音乐唱片 / 作者 credit 文本不翻译
fn is_credit_entry(key: &str, original: &str) -> bool {
    let key_hit = ["music_disc", "soundtrack", "credit", "author", "artist"]
        .iter()
        .any(|needle| key.contains(needle));
    key_hit && is_credit_shaped(original)
}

/// `前段 - 后段` 且两段长度在 2..80 / 2..120 之间
fn is_credit_shaped(text: &str) -> bool {
    let split = text
        .split_once(" - ")
        .or_else(|| text.split_once('–'))
        .or_else(|| text.split_once('—'));
    let Some((head, tail)) = split else {
        return false;
    };
    let head_len = head.trim().chars().count();
    let tail_len = tail.trim().chars().count();
    (2..=80).contains(&head_len) && (2..=120).contains(&tail_len)
}

/// URL / 命令 / ID / credit 类条目直接透传
pub fn is_passthrough_entry(key: &str, source_text: &str) -> bool {
    let original = source_text.trim();
    is_passthrough_text(original) || is_credit_entry(key, original)
}

/// 待翻译条目判定：空 / 透传跳过；原文==译文或译文无中文时需要复核
pub fn requires_work(key: &str, source_text: &str, existing_target: Option<&str>) -> bool {
    let original = source_text.trim();
    if original.is_empty() {
        return false;
    }
    let target = existing_target.map(str::trim).unwrap_or("");
    let passthrough = is_passthrough_entry(key, original);
    let identical_needs_review = !target.is_empty() && target == original && !passthrough;

    if !target.is_empty() && has_chinese(target) && !identical_needs_review {
        return false;
    }
    if passthrough {
        return false;
    }
    if target.is_empty() {
        return true;
    }
    identical_needs_review || !has_chinese(target)
}

/// 语言条目工作量权重（排序与进度用，非定价）
pub fn language_work_weight(text: &str) -> f64 {
    let characters = text.chars().count();
    let protected =
        scan_printf(text).len() + scan_braces(text).len() + scan_format_codes(text).len();
    (1.0 + (characters as f64 / 80.0).min(3.0) + protected as f64 * 0.35).round()
}

#[cfg(test)]
#[path = "quality_test.rs"]
mod tests;
