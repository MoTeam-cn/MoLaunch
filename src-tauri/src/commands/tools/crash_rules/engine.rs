//! 崩溃规则引擎
//!
//! 判定顺序与 PCL2 的三层思路一致：先精确关键词（`all`/`any`/`none`），
//! 再正则（`regex`）。命中即产出分析条目，未命中交给调用方的兜底逻辑。

use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;

use super::schema::{CrashRule, RuleSet};
use crate::commands::tools::types::CrashAnalysisItem;

/// 编译后的正则缓存（key = 原始 pattern）
static REGEX_CACHE: OnceLock<std::sync::Mutex<HashMap<String, Regex>>> = OnceLock::new();

fn regex_cache() -> &'static std::sync::Mutex<HashMap<String, Regex>> {
    REGEX_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// 解析内置规则集（首次调用时解析，失败则返回空规则集）
fn ruleset() -> &'static RuleSet {
    static RULES: OnceLock<RuleSet> = OnceLock::new();
    RULES.get_or_init(|| {
        let raw = crate::resources::read_resource("crash_rules.json").unwrap_or_default();
        match serde_json::from_str::<RuleSet>(&raw) {
            Ok(set) => {
                crate::log_info!(
                    "[CrashRules] 已加载 {} 条崩溃规则（version {}）",
                    set.rules.len(),
                    set.version
                );
                set
            }
            Err(e) => {
                crate::log_error!("[CrashRules] 规则解析失败: {}", e);
                RuleSet {
                    version: "0".to_string(),
                    rules: Vec::new(),
                }
            }
        }
    })
}

/// 当前规则集版本
pub fn rules_version() -> String {
    ruleset().version.clone()
}

/// 规则总数（供调试与自检）
pub fn rules_count() -> usize {
    ruleset().rules.len()
}

/// 用规则集分析日志；`mod_index` 为 modid → 显示名，用于反查可疑 mod
pub fn analyze(
    log_text: &str,
    mod_index: Option<&HashMap<String, String>>,
) -> Vec<CrashAnalysisItem> {
    let mut items: Vec<CrashAnalysisItem> = Vec::new();
    if log_text.trim().is_empty() {
        return items;
    }
    let lower = log_text.to_lowercase();
    let lines: Vec<&str> = log_text.lines().collect();

    for rule in &ruleset().rules {
        if !matches(rule, &lower) {
            continue;
        }
        let suspects = if rule.hint_mod {
            find_suspect_mods(&lower, mod_index)
        } else {
            Vec::new()
        };
        items.push(build_item(rule, &lines, suspects));
    }

    // 严重级别高的排前面，同级保持规则文件顺序
    items.sort_by_key(|item| severity_rank(&item.severity));
    items
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    }
}

fn matches(rule: &CrashRule, lower: &str) -> bool {
    let spec = &rule.match_spec;
    if spec.all.is_empty() && spec.any.is_empty() && spec.regex.is_empty() {
        return false;
    }
    if !spec.all.iter().all(|kw| lower.contains(&kw.to_lowercase())) {
        return false;
    }
    if !spec.any.is_empty() && !spec.any.iter().any(|kw| lower.contains(&kw.to_lowercase())) {
        return false;
    }
    if !spec.regex.is_empty() && !spec.regex.iter().any(|pattern| regex_hit(pattern, lower)) {
        return false;
    }
    if spec
        .none
        .iter()
        .any(|kw| lower.contains(&kw.to_lowercase()))
    {
        return false;
    }
    true
}

fn regex_hit(pattern: &str, text: &str) -> bool {
    let mut cache = match regex_cache().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let regex = cache.entry(pattern.to_string()).or_insert_with(|| {
        match Regex::new(&format!("(?i){}", pattern)) {
            Ok(regex) => regex,
            Err(_) => Regex::new("$^").expect("空正则"),
        }
    });
    regex.is_match(text)
}

/// 从 mod 索引里找出日志中出现的 mod（最多 3 个）
fn find_suspect_mods(lower: &str, mod_index: Option<&HashMap<String, String>>) -> Vec<String> {
    let Some(index) = mod_index else {
        return Vec::new();
    };
    let mut found: Vec<String> = Vec::new();
    for (modid, display) in index {
        if modid.len() < 3 || found.len() >= 3 {
            continue;
        }
        if lower.contains(modid) {
            found.push(display.clone());
        }
    }
    found.sort();
    found
}

/// 在日志里定位与规则首个关键词对应的行，作为展示细节
fn build_item(rule: &CrashRule, lines: &[&str], suspects: Vec<String>) -> CrashAnalysisItem {
    let mut detail = String::new();
    if let Some(needle) = rule
        .match_spec
        .all
        .first()
        .or_else(|| rule.match_spec.any.first())
    {
        let needle_lower = needle.to_lowercase();
        for line in lines {
            if line.to_lowercase().contains(&needle_lower) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    detail = crate::utils::format::truncate_chars(trimmed, 300);
                    break;
                }
            }
        }
    }

    CrashAnalysisItem {
        category: rule.category.clone(),
        severity: rule.severity.clone(),
        title: rule.title.clone(),
        detail,
        suggestion: rule.suggestion.clone(),
        rule_id: rule.id.clone(),
        actions: rule.actions.clone(),
        refs: rule.refs.clone(),
        suspects,
    }
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod engine_test;
