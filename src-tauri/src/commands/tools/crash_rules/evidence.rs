//! 日志行索引与取证工具
//!
//! 一切结论都必须落到具体行：本模块只负责「按关键词/正则找到行号 + 取该行原文」，
//! 不做任何改写，保证前端展示的证据能与日志原文逐字对上。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use regex::Regex;

use crate::commands::tools::types::CrashEvidence;

/// 单条证据最多保留的字符数（避免超长堆栈行把界面撑爆）
const MAX_EVIDENCE_CHARS: usize = 400;

static REGEX_CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();

fn regex_cache() -> &'static Mutex<HashMap<String, Regex>> {
    REGEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 编译并缓存正则（大小写不敏感）；非法正则记警告后返回 None
pub fn compiled(pattern: &str) -> Option<Regex> {
    let mut cache = match regex_cache().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(regex) = cache.get(pattern) {
        return Some(regex.clone());
    }
    match Regex::new(&format!("(?i){}", pattern)) {
        Ok(regex) => {
            cache.insert(pattern.to_string(), regex.clone());
            Some(regex)
        }
        Err(e) => {
            crate::log_warn!("[CrashRules] 正则无效: {} ({})", pattern, e);
            None
        }
    }
}

/// 日志行索引（原文 + 小写副本），供规则引擎与特殊规则共用
pub struct LogIndex {
    lines: Vec<String>,
    lower: Vec<String>,
    text_lower: String,
}

impl LogIndex {
    pub fn new(text: &str) -> Self {
        let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
        let lower: Vec<String> = lines.iter().map(|line| line.to_lowercase()).collect();
        Self {
            lines,
            lower,
            text_lower: text.to_lowercase(),
        }
    }

    /// 日志总行数
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// 全文是否包含关键词（大小写不敏感）
    pub fn contains(&self, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        !needle.is_empty() && self.text_lower.contains(&needle)
    }

    /// 全文小写副本（供正则匹配）
    pub fn text_lower(&self) -> &str {
        &self.text_lower
    }

    /// 关键词在全文出现的次数
    pub fn count(&self, needle: &str) -> usize {
        let needle = needle.to_lowercase();
        if needle.is_empty() {
            return 0;
        }
        self.text_lower.matches(&needle).count()
    }

    /// 首个命中关键词的行号（从 1 开始）
    pub fn first_line(&self, needle: &str) -> Option<usize> {
        let needle = needle.to_lowercase();
        self.lower
            .iter()
            .position(|line| line.contains(&needle))
            .map(|index| index + 1)
    }

    /// 首个命中正则的行号（从 1 开始）
    pub fn first_regex_line(&self, pattern: &str) -> Option<usize> {
        let regex = compiled(pattern)?;
        self.lower
            .iter()
            .position(|line| regex.is_match(line))
            .map(|index| index + 1)
    }

    /// 所有命中关键词的行号（升序）
    pub fn all_lines(&self, needle: &str) -> Vec<usize> {
        let needle = needle.to_lowercase();
        self.lower
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains(&needle))
            .map(|(index, _)| index + 1)
            .collect()
    }

    /// 全部原始行（未 trim）
    pub fn raw_lines(&self) -> &[String] {
        &self.lines
    }

    /// 取某行的证据（原文去首尾空白后截断，仍是日志子串）
    pub fn evidence(&self, line_no: usize) -> Option<CrashEvidence> {
        let raw = self.lines.get(line_no.checked_sub(1)?)?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        Some(CrashEvidence {
            line_no,
            raw_line: crate::utils::format::truncate_chars(trimmed, MAX_EVIDENCE_CHARS),
        })
    }

    /// 按关键词 + 正则收集证据行（去重、按行号升序、最多 limit 条）
    pub fn collect(
        &self,
        needles: &[String],
        patterns: &[String],
        limit: usize,
    ) -> Vec<CrashEvidence> {
        let mut found: Vec<usize> = Vec::new();
        for needle in needles {
            if let Some(line_no) = self.first_line(needle) {
                found.push(line_no);
            }
        }
        for pattern in patterns {
            if let Some(line_no) = self.first_regex_line(pattern) {
                found.push(line_no);
            }
        }
        found.sort_unstable();
        found.dedup();
        found
            .into_iter()
            .filter_map(|line_no| self.evidence(line_no))
            .take(limit)
            .collect()
    }

    /// 以 center 行为中心，在 ±span 行内找首个命中关键词的行（含 center 自身）
    pub fn window(&self, center: usize, span: usize, needles: &[String]) -> Option<CrashEvidence> {
        if self.lines.is_empty() {
            return None;
        }
        let start = center.saturating_sub(span).max(1);
        let end = (center + span).min(self.lines.len());
        let lowered: Vec<String> = needles.iter().map(|n| n.to_lowercase()).collect();
        for line_no in start..=end {
            let Some(line) = self.lower.get(line_no - 1) else {
                continue;
            };
            if lowered.iter().any(|needle| line.contains(needle)) {
                if let Some(evidence) = self.evidence(line_no) {
                    return Some(evidence);
                }
            }
        }
        None
    }
}

#[cfg(test)]
#[path = "evidence_test.rs"]
mod evidence_test;
