//! 第二级堆栈分析
//!
//! 从崩溃报告、运行时日志、hs_err 的堆栈信息中提取关键字，
//! 过滤后推断可能的 Mod 名称。

use super::super::types::{CrashCategory, CrashInfo};
use std::path::Path;

/// 第二级堆栈分析
pub(super) fn analyze_stack(
    log_mc: &str,
    log_crash: &str,
    log_hs: &str,
    error_lines: &[String],
    crash_report_path: Option<&Path>,
) -> Option<CrashInfo> {
    // 从各源提取堆栈关键字
    let mut keywords = Vec::new();

    // 从崩溃报告提取
    if !log_crash.is_empty() {
        if let Some(details_end) = log_crash.find("System Details") {
            keywords.extend(extract_stack_keywords(&log_crash[..details_end]));
        }
    }

    // 从运行时日志的 FATAL 行提取
    if !log_mc.is_empty() {
        keywords.extend(extract_stack_keywords(log_mc));
    }

    // 从 hs_err 的 THREAD 段提取
    if !log_hs.is_empty() {
        if let Some(thread_start) = log_hs.find("T H R E A D") {
            let thread_section = if let Some(reg_start) = log_hs[thread_start..].find("Registers:")
            {
                &log_hs[thread_start..thread_start + reg_start]
            } else {
                &log_hs[thread_start..]
            };
            keywords.extend(extract_stack_keywords(thread_section));
        }
    }

    if keywords.is_empty() {
        return None;
    }

    // 过滤并提取 Mod 名称
    let mod_names = analyze_mod_name(&keywords);
    if let Some(ref names) = mod_names {
        let names_str = names.join(", ");
        return Some(CrashInfo {
            reason: format!("堆栈分析发现可能的 Mod: {}", names_str),
            category: CrashCategory::Mod,
            log_lines: error_lines.to_vec(),
            suggestion: format!("以下 Mod 可能导致了游戏崩溃：{}\n你可以尝试依次禁用上述 Mod，然后观察游戏是否还会崩溃。", names_str),
            problematic_mod: Some(names_str),
            crash_report_path: crash_report_path.map(|p| p.to_string_lossy().to_string()),
            log_tail: Vec::new(),
        });
    }

    None
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
        "oolloo.",
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
    if keywords.is_empty() {
        return None;
    }
    if keywords.len() > 10 {
        // 关键词过多，可能是匹配出错
        return None;
    }
    let unique: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        keywords
            .iter()
            .filter(|k| seen.insert((*k).clone()))
            .cloned()
            .collect()
    };
    if unique.is_empty() {
        None
    } else {
        Some(unique)
    }
}
