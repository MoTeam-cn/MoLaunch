//! 证据提取（Detector）
//!
//! 多个独立检测器并行从各来源文本提取 `Evidence`，互不依赖、无短路。
//! 各检测器职责单一：关键字规则遍历 / 类名提取 / Mod 名提取 / 极短输出判定。

use super::super::types::CrashCategory;
use super::collect::CollectedSources;
use super::rules::{SourceKind, KEYWORD_RULES};
use super::util::extract_mod_from_keyword;

/// 检测到的崩溃证据
#[derive(Debug, Clone)]
pub struct Evidence {
    /// 置信度 0.0-1.0
    pub confidence: f64,
    /// 崩溃分类
    pub category: CrashCategory,
    /// 证据来源
    pub source: SourceKind,
    /// 原因摘要
    pub reason: String,
    /// 建议解决方案
    pub suggestion: String,
    /// 可能相关的 Mod（None 表示非 Mod 类证据）
    pub extracted_mod: Option<String>,
}

/// 检测器接口
pub trait Detector {
    /// 检测器标识（日志输出用）
    fn name(&self) -> &'static str;
    /// 从各来源文本提取证据
    fn detect(&self, sources: &CollectedSources) -> Vec<Evidence>;
}

/// 通用关键字检测器：遍历规则表，在对应来源中做小写不敏感包含匹配
pub struct KeywordDetector;

impl Detector for KeywordDetector {
    fn name(&self) -> &'static str {
        "keyword"
    }

    fn detect(&self, sources: &CollectedSources) -> Vec<Evidence> {
        // 每个来源只小写化一次
        let runtime_l = sources.runtime_log.to_lowercase();
        let crash_l = sources.crash_report_text.to_lowercase();
        let hs_err_l = sources.hs_err_text.to_lowercase();

        let mut out = Vec::new();
        for rule in KEYWORD_RULES {
            let lower = match rule.source {
                SourceKind::RuntimeLog => &runtime_l,
                SourceKind::CrashReport => &crash_l,
                SourceKind::HsErr => &hs_err_l,
            };
            if lower.is_empty() {
                continue;
            }
            let matched = rule.patterns.iter().any(|p| lower.contains(*p))
                && rule.and_patterns.iter().all(|p| lower.contains(*p));
            if !matched {
                continue;
            }
            crate::log_info!("[CrashAnalyzer] 关键字规则命中: {}", rule.id);
            out.push(Evidence {
                confidence: rule.confidence,
                category: rule.category.clone(),
                source: rule.source,
                reason: rule.reason.to_string(),
                suggestion: rule.suggestion.to_string(),
                extracted_mod: None,
            });
        }
        out
    }
}

/// 类缺失检测器：从运行时日志的 ClassNotFoundException 中提取类名并归类
pub struct ClassNotFoundDetector;

impl Detector for ClassNotFoundDetector {
    fn name(&self) -> &'static str {
        "class_not_found"
    }

    fn detect(&self, sources: &CollectedSources) -> Vec<Evidence> {
        let text = &sources.runtime_log;
        if text.is_empty() || !text.to_lowercase().contains("classnotfoundexception") {
            return Vec::new();
        }

        let missing_class = extract_class_name(text, "classnotfoundexception");
        let class_hint = missing_class.unwrap_or_else(|| "未知类".to_string());
        let class_l = class_hint.to_lowercase();

        // 与原逻辑保持一致的分类判定顺序；分类统一用 Unknown（不改变对外分类）
        let (reason, suggestion) = if class_l.contains("fabric") || class_l.contains("knot") {
            (
                "Fabric 加载器核心库缺失".to_string(),
                "游戏未能找到 Fabric Loader 的核心类，这通常是版本安装不完整所致。\n请重新安装该版本的 Fabric 加载器，或重建版本。".to_string(),
            )
        } else if class_l.contains("forge")
            || class_l.contains("fml")
            || class_l.contains("modlauncher")
        {
            (
                "Forge 加载器核心库缺失".to_string(),
                "游戏未能找到 Forge 的核心库类，这通常是版本安装不完整所致。\n请重新安装该版本的 Forge，或重建版本。".to_string(),
            )
        } else if class_l.contains("neoforge") {
            (
                "NeoForge 加载器核心库缺失".to_string(),
                "游戏未能找到 NeoForge 的核心库类，这通常是版本安装不完整所致。\n请重新安装该版本的 NeoForge，或重建版本。".to_string(),
            )
        } else {
            (
                "关键 Java 类缺失".to_string(),
                format!(
                    "Java 未能加载类 {}，可能是版本安装不完整或 classpath 配置有误。\n建议重新安装该版本后再试。",
                    class_hint
                ),
            )
        };

        vec![Evidence {
            confidence: 0.90,
            category: CrashCategory::Unknown,
            source: SourceKind::RuntimeLog,
            reason,
            suggestion,
            extracted_mod: None,
        }]
    }
}

/// 指定 Mod 抛异常检测器：从运行时日志提取 "Caught exception from X" 的 Mod 名
pub struct CaughtExceptionDetector;

impl Detector for CaughtExceptionDetector {
    fn name(&self) -> &'static str {
        "caught_exception"
    }

    fn detect(&self, sources: &CollectedSources) -> Vec<Evidence> {
        extract_mod_from_keyword(&sources.runtime_log, "caught exception from ").map_or_else(
            Vec::new,
            |mod_name| {
                vec![Evidence {
                    confidence: 0.95,
                    category: CrashCategory::Mod,
                    source: SourceKind::RuntimeLog,
                    reason: format!("Mod '{}' 抛出异常导致游戏退出", mod_name),
                    suggestion: format!(
                        "崩溃与 Mod {} 有关。\n你可以尝试暂时禁用该 Mod，再启动游戏确认是否恢复正常。",
                        mod_name
                    ),
                    extracted_mod: Some(mod_name),
                }]
            },
        )
    }
}

/// 极短输出检测器：游戏输出极少即退出（常见于 Java 路径或启动参数有误）
pub struct ShortOutputDetector;

impl Detector for ShortOutputDetector {
    fn name(&self) -> &'static str {
        "short_output"
    }

    fn detect(&self, sources: &CollectedSources) -> Vec<Evidence> {
        let log = &sources.runtime_log;
        if !log.is_empty() && log.len() < 100 && !log.contains("at net.") && !log.contains("INFO]")
        {
            vec![Evidence {
                confidence: 0.50,
                category: CrashCategory::Unknown,
                source: SourceKind::RuntimeLog,
                reason: "游戏输出过短（启动即退出）".to_string(),
                suggestion: "游戏输出的内容极少便已退出。\n常见原因是 Java 路径或启动参数有误，请检查启动设置后重试。"
                    .to_string(),
                extracted_mod: None,
            }]
        } else {
            Vec::new()
        }
    }
}

/// 从日志中提取 ClassNotFoundException 中的类名
///
/// 匹配模式：`ClassNotFoundException: net.fabricmc.loader.impl.launch.knot.KnotClient`
/// 或 `java.lang.ClassNotFoundException: xxx`
fn extract_class_name(log: &str, keyword: &str) -> Option<String> {
    let log_l = log.to_lowercase();
    let keyword_l = keyword.to_lowercase();
    if let Some(pos) = log_l.find(&keyword_l) {
        // 从关键字后找类名（可能跨行）
        let rest = &log[pos + keyword_l.len()..];
        // 跳过冒号和空格
        let rest = rest.trim_start_matches(':').trim_start();
        // 类名格式：xxx.xxx.xxx（只含字母、数字、点、$、_）
        let class_end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '.' || c == '$' || c == '_'))
            .unwrap_or(rest.len().min(200));
        let class_name = &rest[..class_end];
        if !class_name.is_empty() {
            return Some(class_name.to_string());
        }
    }
    None
}

#[cfg(test)]
#[path = "detector_tests.rs"]
mod tests;
