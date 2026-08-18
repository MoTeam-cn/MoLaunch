//! 模组翻译：占位符保护、机械/语义审计、待译判定、工作量权重

use std::collections::{BTreeMap, HashSet};

use once_cell::sync::Lazy;
use regex::Regex;

use super::types::has_chinese;

/// 提取文本中的保护占位符集合（printf、${}、{n}/{{x}}、§格式码、\n 转义、控制字符），排序返回
pub fn extract_protected_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for regex in [
        &PRINTF_RE,
        &BRACE_RE,
        &FORMAT_CODE_RE,
        &ESCAPE_RE,
        &CONTROL_RE,
    ] {
        for cap in regex.captures_iter(text) {
            if let Some(m) = cap.get(0) {
                tokens.push(m.as_str().to_string());
            }
        }
    }
    tokens.sort();
    tokens
}

static PRINTF_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"%[-+#0-9]*\.?[0-9]*[a-zA-Z%$][a-zA-Z0-9$]*|%[a-zA-Z]").expect("PRINTF_RE")
});
static BRACE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\{[^}]*\}\}|\$\{[^}]*\}|\{[0-9]+(?::[^}]*)?\}").expect("BRACE_RE"));
static FORMAT_CODE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"§[0-9a-fk-orx]").expect("FORMAT_CODE_RE"));
static ESCAPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\n|\\t|\\r").expect("ESCAPE_RE"));
static CONTROL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u{0001}-\u{0008}\u{000b}\u{000c}\u{000e}-\u{001f}]").expect("CONTROL_RE")
});
static PASSTHROUGH_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?://\S+|\/[a-z0-9_.:-]+(?:\s+[a-z0-9_.:<>{}\[\]-]+)*|[a-z0-9_.-]+:[a-z0-9_./-]+)$")
        .expect("PASSTHROUGH_RE")
});
static CREDIT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^.{2,80}\s[-–—]\s.{2,120}$").expect("CREDIT_RE"));
static WEIGHT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"%\d*\$?[a-z]|\{\w+\}|\$\{[^}]+\}|§[0-9a-fk-or]").expect("WEIGHT_RE"));

/// 期望占位符集合与实际不一致时返回错误描述
pub fn validate_protected_tokens(source: &str, translation: &str) -> Option<String> {
    let expected = extract_protected_tokens(source);
    let actual = extract_protected_tokens(translation);
    if expected == actual {
        None
    } else {
        Some(format!(
            "占位符不一致：期望 [{}]，实际 [{}]",
            expected.join(", "),
            actual.join(", ")
        ))
    }
}

/// 按源文本是否含转义字面量，统一 \n/\r/\t 的转义/还原方向
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

/// 机械不变量审计：缺译（Error）、占位符不一致（Error）、译文无中文（Error）、多余键（Warning）
pub fn audit_invariants(
    source: &BTreeMap<String, String>,
    target: &BTreeMap<String, String>,
) -> Vec<AuditIssue> {
    let mut issues = Vec::new();
    for (key, english) in source {
        match target.get(key) {
            Some(chinese) if !chinese.trim().is_empty() => {
                if let Some(error) = validate_protected_tokens(english, chinese) {
                    issues.push(AuditIssue {
                        severity: AuditSeverity::Error,
                        key: key.clone(),
                        message: error,
                    });
                }
                if requires_work(key, english, Some(chinese)) && !has_chinese(chinese) {
                    issues.push(AuditIssue {
                        severity: AuditSeverity::Error,
                        key: key.clone(),
                        message: "译文不含简体中文；若必须保留原文，需要显式标记 keep-source"
                            .to_string(),
                    });
                }
            }
            _ => issues.push(AuditIssue {
                severity: AuditSeverity::Error,
                key: key.clone(),
                message: "缺少中文译文".to_string(),
            }),
        }
    }
    for key in target.keys() {
        if !source.contains_key(key) {
            issues.push(AuditIssue {
                severity: AuditSeverity::Warning,
                key: key.clone(),
                message: "中文文件包含源语言中不存在的额外条目".to_string(),
            });
        }
    }
    issues
}

/// 精简官方术语表：(正则, 必须含中文, 提示语)
static OFFICIAL_TERMS: Lazy<Vec<(Regex, &'static str, &'static str)>> = Lazy::new(|| {
    [
        (
            r"\bIron Bars\b",
            "铁栏杆",
            "Iron Bars 应沿用官方简中术语“铁栏杆”",
        ),
        (
            r"\bEnd(?:er)? Brick\b",
            "末地石砖",
            "End Brick 应沿用官方材料名“末地石砖”",
        ),
        (r"\bCrimson\b", "绯红", "Crimson 应沿用官方译名“绯红”"),
        (r"\bWarped\b", "诡异", "Warped 应沿用官方译名“诡异”"),
        (
            r"\bPale Oak\b",
            "苍白橡木",
            "Pale Oak 应沿用官方译名“苍白橡木”",
        ),
        (
            r"\bDark Oak\b",
            "深色橡木",
            "Dark Oak 应沿用官方译名“深色橡木”",
        ),
        (
            r"\bOak Planks\b",
            "橡木木板",
            "Oak Planks 应沿用完整材料名“橡木木板”",
        ),
    ]
    .iter()
    .map(|(pattern, required, label)| {
        (
            Regex::new(pattern).expect("OFFICIAL_TERMS"),
            *required,
            *label,
        )
    })
    .collect()
});

/// 语义审计：官方术语强制（Error）、同英文多译法（Warning）
pub fn audit_semantic(
    source: &BTreeMap<String, String>,
    target: &BTreeMap<String, String>,
) -> Vec<AuditIssue> {
    let mut issues = Vec::new();
    let mut by_english: BTreeMap<&str, Vec<(&String, &String)>> = BTreeMap::new();
    for (key, english) in source {
        let Some(chinese) = target.get(key) else {
            continue;
        };
        if chinese.trim().is_empty() {
            continue;
        }
        for (pattern, required, label) in OFFICIAL_TERMS.iter() {
            if pattern.is_match(english) && !chinese.contains(required) {
                issues.push(AuditIssue {
                    severity: AuditSeverity::Error,
                    key: key.clone(),
                    message: label.to_string(),
                });
            }
        }
        by_english
            .entry(english.as_str())
            .or_default()
            .push((key, chinese));
    }
    for (_, group) in by_english {
        if group.len() <= 1 {
            continue;
        }
        let variants: HashSet<&String> = group.iter().map(|(_, c)| *c).collect();
        if variants.len() > 1 {
            for (key, _) in group {
                issues.push(AuditIssue {
                    severity: AuditSeverity::Warning,
                    key: key.clone(),
                    message: format!(
                        "相同英文原文出现 {} 种译法，请确认是否需要统一",
                        variants.len()
                    ),
                });
            }
        }
    }
    issues
}

/// 待翻译条目判定（空/URL/ID/音乐唱片 credit 跳过；原文==译文需复核）
pub fn requires_work(key: &str, source_text: &str, existing_target: Option<&str>) -> bool {
    let original = source_text.trim();
    if original.is_empty() {
        return false;
    }
    let target = existing_target.map(str::trim).unwrap_or("");
    let protected_only = is_passthrough_entry(key, original);
    let credited_work = is_credit_entry(key, original);
    let identical_needs_review =
        !target.is_empty() && target == original && !protected_only && !credited_work;

    if !target.is_empty() && has_chinese(target) && !identical_needs_review {
        return false;
    }
    if protected_only || credited_work {
        return false;
    }
    if target.is_empty() {
        return true;
    }
    identical_needs_review || !has_chinese(target)
}

/// URL/命令/ID/credit 类条目直接透传
pub fn is_passthrough_entry(key: &str, source_text: &str) -> bool {
    let original = source_text.trim();
    PASSTHROUGH_RE.is_match(original) || is_credit_entry(key, original)
}

/// 音乐唱片/作者 credit 文本（"C418 - Cat" 形式）不翻译
fn is_credit_entry(key: &str, original: &str) -> bool {
    (key.contains("music_disc")
        || key.contains("soundtrack")
        || key.contains("credit")
        || key.contains("author")
        || key.contains("artist"))
        && CREDIT_RE.is_match(original)
}

/// 语言条目工作量权重（用于排序与进度，非定价）
pub fn language_work_weight(text: &str) -> f64 {
    let characters = text.chars().count();
    let protected = WEIGHT_RE.find_iter(text).count();
    (1.0 + (characters as f64 / 80.0).min(3.0) + protected as f64 * 0.35).round()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printf_placeholders_must_survive() {
        assert!(validate_protected_tokens("Spawn %d zombies", "生成 %d 只僵尸").is_none());
        assert!(validate_protected_tokens("Spawn %d zombies", "生成 %s 只僵尸").is_some());
        assert!(
            validate_protected_tokens("%1$s took %2$d damage", "%1$s 受到 %2$d 点伤害").is_none()
        );
    }

    #[test]
    fn format_codes_and_escapes_must_survive() {
        assert!(validate_protected_tokens("§aGreen text", "§a绿色文字").is_none());
        assert!(validate_protected_tokens("line\\nbreak", "换行\\n测试").is_none());
        assert!(validate_protected_tokens("line\\nbreak", "换行测试").is_some());
    }

    #[test]
    fn model_escapes_normalized_to_source_direction() {
        assert_eq!(
            normalize_model_translation("Rendering disabled: %s", "渲染已禁用：%s\\n请重新启用"),
            "渲染已禁用：%s\n请重新启用"
        );
        assert_eq!(
            normalize_model_translation("line\\nbreak", "第一行\n第二行"),
            "第一行\\n第二行"
        );
    }

    #[test]
    fn invariants_catch_missing_and_placeholder_breaks() {
        let source = BTreeMap::from([("a".to_string(), "Spawn %d".to_string())]);
        let target = BTreeMap::from([("a".to_string(), "生成".to_string())]);
        let issues = audit_invariants(&source, &target);
        assert!(issues
            .iter()
            .any(|i| i.severity == AuditSeverity::Error && i.key == "a"));
    }

    #[test]
    fn official_terms_and_consistent_translation() {
        let source = BTreeMap::from([
            ("k1".to_string(), "Iron Bars".to_string()),
            ("k2".to_string(), "Wrench".to_string()),
            ("k3".to_string(), "Wrench".to_string()),
        ]);
        let target = BTreeMap::from([
            ("k1".to_string(), "铁条".to_string()),
            ("k2".to_string(), "扳手".to_string()),
            ("k3".to_string(), "螺丝刀".to_string()),
        ]);
        let issues = audit_semantic(&source, &target);
        assert!(issues.iter().any(|i| i.message.contains("铁栏杆")));
        assert!(issues.iter().any(|i| i.message.contains("种译法")));
    }
}
