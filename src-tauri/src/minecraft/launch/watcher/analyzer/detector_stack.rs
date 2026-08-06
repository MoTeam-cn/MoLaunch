//! 堆栈证据检测器
//!
//! 从崩溃报告 / 运行时日志 / hs_err 的堆栈中提取关键字，过滤后推断可能的 Mod 名称。
//! 作为独立 Detector 参与并行检测，仅在存在 Mod 加载器时产出证据。

use super::super::types::CrashCategory;
use super::collect::CollectedSources;
use super::detector::{Detector, Evidence};
use super::rules::SourceKind;

/// 堆栈检测器：启发式提取堆栈中可能属于 Mod 的关键字
pub struct StackDetector;

impl Detector for StackDetector {
    fn name(&self) -> &'static str {
        "stack"
    }

    fn detect(&self, sources: &CollectedSources) -> Vec<Evidence> {
        // 仅当运行时日志存在 Mod 加载器痕迹时才进行堆栈分析
        let has_mod_loader = sources.runtime_log.contains("orge")
            || sources.runtime_log.contains("abric")
            || sources.runtime_log.contains("uilt")
            || sources.runtime_log.contains("iteloader")
            || sources.runtime_log.contains("ModLauncher")
            || sources.runtime_log.contains("fmlloader");
        if !has_mod_loader {
            return Vec::new();
        }

        let mut keywords = Vec::new();

        // 崩溃报告：取 System Details 之前的部分
        if !sources.crash_report_text.is_empty() {
            if let Some(details_end) = sources.crash_report_text.find("System Details") {
                keywords.extend(extract_stack_keywords(
                    &sources.crash_report_text[..details_end],
                ));
            }
        }

        // 运行时日志
        if !sources.runtime_log.is_empty() {
            keywords.extend(extract_stack_keywords(&sources.runtime_log));
        }

        // hs_err：取 THREAD 段（Registers: 之前）
        if !sources.hs_err_text.is_empty() {
            if let Some(thread_start) = sources.hs_err_text.find("T H R E A D") {
                let thread_section = if let Some(reg_start) =
                    sources.hs_err_text[thread_start..].find("Registers:")
                {
                    &sources.hs_err_text[thread_start..thread_start + reg_start]
                } else {
                    &sources.hs_err_text[thread_start..]
                };
                keywords.extend(extract_stack_keywords(thread_section));
            }
        }

        let mod_names = analyze_mod_name(&keywords);
        mod_names.map_or_else(Vec::new, |names| {
            let names_str = names.join(", ");
            vec![Evidence {
                confidence: 0.80,
                category: CrashCategory::Mod,
                source: SourceKind::RuntimeLog,
                reason: format!("堆栈分析发现可能的 Mod: {}", names_str),
                suggestion: format!(
                    "崩溃堆栈中出现了以下 Mod 的相关代码：{}\n可以尝试逐个禁用这些 Mod，观察游戏能否恢复正常。",
                    names_str
                ),
                extracted_mod: Some(names_str),
            }]
        })
    }
}

/// 从堆栈文本提取关键字
fn extract_stack_keywords(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let excluded_packages = [
        "java.",
        "javax.",
        "sun.",
        "com.sun.",
        "jdk.",
        "org.lwjgl",
        "net.minecraftforge",
        "paulscode.sound",
        "com.mojang",
        "net.minecraft",
        "cpw.mods",
        "com.google",
        "org.apache",
        "org.spongepowered",
        "net.fabricmc",
        "com.mumfrey",
        "com.electronwill.nightconfig",
        "it.unimi.dsi",
        "MojangTricksIntelDriversForPerformance",
    ];

    // 匹配 "at xxx.yyy.Zzz(" 格式的堆栈行
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("at ") {
            continue;
        }
        let rest = &trimmed[3..];
        if let Some(paren) = rest.find('(') {
            let class_path = &rest[..paren];
            // 排除非 Mod 包名
            let is_excluded = excluded_packages.iter().any(|p| class_path.starts_with(p));
            if !is_excluded && class_path.contains('.') {
                // 提取前 4 节作为关键字
                let parts: Vec<&str> = class_path.split('.').collect();
                for part in parts.iter().take(4.min(parts.len())) {
                    let word = *part;
                    if word.len() <= 2 || word.starts_with("func_") {
                        continue;
                    }
                    let word_l = word.to_lowercase();
                    // 排除通用词
                    if matches!(
                        word_l.as_str(),
                        "com"
                            | "org"
                            | "net"
                            | "asm"
                            | "fml"
                            | "mod"
                            | "forge"
                            | "fabric"
                            | "minecraft"
                            | "optifine"
                            | "internal"
                            | "common"
                            | "core"
                            | "api"
                            | "util"
                            | "lib"
                            | "client"
                            | "server"
                            | "event"
                            | "config"
                            | "block"
                            | "item"
                            | "entity"
                            | "render"
                            | "world"
                            | "game"
                            | "player"
                            | "tile"
                            | "gui"
                            | "screen"
                            | "packet"
                            | "network"
                            | "registry"
                            | "loader"
                            | "mixin"
                            | "concurrent"
                    ) {
                        continue;
                    }
                    results.push(word.to_string());
                }
            }
        }
    }

    results
}

/// 从关键字列表分析 Mod 名称
fn analyze_mod_name(keywords: &[String]) -> Option<Vec<String>> {
    if keywords.is_empty() || keywords.len() > 10 {
        // 无关键字或关键词过多（可能匹配出错）时放弃
        return None;
    }
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<String> = keywords
        .iter()
        .filter(|k| seen.insert((*k).clone()))
        .cloned()
        .collect();
    if unique.is_empty() {
        None
    } else {
        Some(unique)
    }
}
