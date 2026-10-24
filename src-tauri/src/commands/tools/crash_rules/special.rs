//! 需要上下文判断的取证规则
//!
//! 这几类判断只看单个关键词会误报（例如 NoClassDefFoundError 既可能是缺前置、
//! 也可能是 mod 文件损坏），必须结合邻行与 mod 列表才能定性，因此放在代码里而不是 JSON。

use std::collections::HashMap;

use super::evidence::LogIndex;
use crate::commands::tools::types::{CrashAnalysisItem, CrashEvidence};

/// 依赖声明的检索窗口（命中行前后各 30 行）
const DEP_WINDOW: usize = 30;

/// 依赖声明关键词（Fabric / Forge / Quilt 的措辞）
const DEP_NEEDLES: &[&str] = &[
    "is missing dependency",
    "missing dependency",
    "missing mod",
    "requires mod",
    "requires ",
];

/// 执行全部特殊规则，返回（已确认, 待确认）
pub fn run(
    index: &LogIndex,
    mod_index: Option<&HashMap<String, String>>,
) -> (Vec<CrashAnalysisItem>, Vec<CrashAnalysisItem>) {
    let mut confirmed: Vec<CrashAnalysisItem> = Vec::new();
    let mut pending: Vec<CrashAnalysisItem> = Vec::new();

    if let Some(item) = missing_dependency(index) {
        if item.evidence.is_empty() {
            pending.push(item);
        } else {
            confirmed.push(item);
        }
    }
    confirmed.extend(plugin_load_failures(index));
    if let Some(item) = duplicate_mods(index) {
        confirmed.push(item);
    }

    // 可疑 mod 反查只补充在「缺前置 / 类缺失」上，避免到处挂 suspects
    if let Some(table) = mod_index {
        for item in confirmed.iter_mut() {
            if item.cause != "missing_dependency" && item.cause != "class_missing" {
                continue;
            }
            let mut suspects: Vec<String> = Vec::new();
            for (modid, display) in table {
                if modid.len() >= 3 && suspects.len() < 3 && index.contains(modid) {
                    suspects.push(display.clone());
                }
            }
            suspects.sort();
            item.suspects = suspects;
        }
    }

    (confirmed, pending)
}

/// 插件加载失败：按「缺类 / Java 版本过低 / 类文件缺失」三种子因分别输出
///
/// 插件（Bukkit / Spigot）报错只有一行 `Failed to load plugin 'X'`，真正原因在其后几行，
/// 所以必须顺着往下找异常类型，否则只能得到「加载失败」这种没有诊断价值的结论。
fn plugin_load_failures(index: &LogIndex) -> Vec<CrashAnalysisItem> {
    let mut items: Vec<CrashAnalysisItem> = Vec::new();
    for line_no in index.all_lines("failed to load plugin '") {
        let Some(plugin) = plugin_name(&index.raw_lines()[line_no - 1]) else {
            continue;
        };
        let mut evidence: Vec<CrashEvidence> = Vec::new();
        if let Some(head) = index.evidence(line_no) {
            evidence.push(head);
        }
        // 只看该插件行之后 4 行，且不越过下一个插件的报错行
        let cause_line = plugin_cause_line(index, line_no);
        let Some(cause_line) = cause_line else {
            continue;
        };
        let lowered = cause_line.raw_line.to_lowercase();
        let (cause, title, suggestion, confidence) =
            if lowered.contains("unsupportedclassversionerror") {
                (
                    "plugin_java_version",
                    format!("插件 {} 需要更高版本的 Java", plugin),
                    "把服务端 Java 升级到插件要求的版本（看日志里的 class 版本号）。",
                    "high",
                )
            } else if lowered.contains("noclassdeffounderror") {
                (
                    "plugin_missing_class",
                    format!("插件 {} 缺少依赖类", plugin),
                    "补装该插件的前置插件 / 库，或确认服务端核心版本与插件匹配。",
                    "medium",
                )
            } else {
                (
                    "plugin_class_missing",
                    format!("插件 {} 的类文件缺失（jar 不完整）", plugin),
                    "重新下载该插件；若手动改过 jar 内容，改回官方版本。",
                    "medium",
                )
            };
        evidence.push(cause_line);
        items.push(item(
            cause,
            "plugin",
            "error",
            title,
            confidence,
            evidence,
            String::new(),
            "该插件被禁用，相关命令 / 功能不可用。",
            suggestion,
            "plugin-load-failed",
        ));
    }
    items
}

/// 取插件失败的真正原因行：在插件行之后 4 行内，遇到下一个插件行即停止
fn plugin_cause_line(index: &LogIndex, line_no: usize) -> Option<CrashEvidence> {
    let needles = [
        "noclassdeffounderror",
        "unsupportedclassversionerror",
        "classnotfoundexception",
    ];
    for offset in 1..=4 {
        let no = line_no + offset;
        let Some(raw) = index.raw_lines().get(no - 1) else {
            break;
        };
        let lowered = raw.to_lowercase();
        if lowered.contains("failed to load plugin '") {
            break;
        }
        if needles.iter().any(|needle| lowered.contains(needle)) {
            return index.evidence(no);
        }
    }
    None
}

/// 从 `Failed to load plugin 'X'` 里取出插件名
fn plugin_name(line: &str) -> Option<String> {
    let start = line.find('\'')? + 1;
    let rest = line.get(start..)?;
    let end = rest.find('\'')?;
    let name = rest.get(..end)?.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// NoClassDefFoundError 归因：先在 ±30 行内找依赖声明，找不到才降级为「类缺失」
fn missing_dependency(index: &LogIndex) -> Option<CrashAnalysisItem> {
    let needles: Vec<String> = DEP_NEEDLES.iter().map(|s| s.to_string()).collect();
    let error_lines = index.all_lines("noclassdeffounderror");

    for line_no in &error_lines {
        if let Some(dep) = index.window(*line_no, DEP_WINDOW, &needles) {
            let mut evidence = vec![dep.clone()];
            if let Some(err) = index.evidence(*line_no) {
                evidence.push(err);
            }
            let missing = extract_quoted(&dep.raw_line, "dependency");
            let owner = extract_quoted(&dep.raw_line, "mod");
            let title = match (&owner, &missing) {
                (Some(owner), Some(missing)) => format!("缺少前置 mod：{} 需要 {}", owner, missing),
                (None, Some(missing)) => format!("缺少前置 mod：{}", missing),
                _ => "缺少前置 mod".to_string(),
            };
            return Some(item(
                "missing_dependency",
                "missing_mod",
                "error",
                title,
                "high",
                evidence,
                String::new(),
                "游戏在加载阶段就中断，mod 无法初始化，通常直接回到启动器或卡在加载界面。",
                "日志里明确写了缺哪个前置，装上对应版本的前置 mod 即可；版本号要满足括号里的下限要求。",
                "mod-missing-dependency",
            ));
        }
    }

    // 没有任何依赖声明 → 只能给出低置信结论，并说明还缺什么证据
    let first = error_lines.first().copied()?;
    let evidence: Vec<CrashEvidence> = index.evidence(first).into_iter().collect();
    Some(item(
        "class_missing",
        "missing_mod",
        "warning",
        "类缺失（可能是 mod 文件损坏或版本不匹配）".to_string(),
        "low",
        evidence,
        "未找到「is missing dependency / requires」这类前置声明，无法确认是缺前置还是 mod 自身损坏".to_string(),
        "相关 mod 无法加载，功能缺失或直接崩溃。",
        "先确认该 mod 的依赖项是否齐全；若依赖齐全，多半是 jar 下载不完整或被解压过，重新下载该 mod。",
        "mod-class-missing",
    ))
}

/// 同名 mod 多版本并存（必须真找到两条不同版本的列表行才输出）
fn duplicate_mods(index: &LogIndex) -> Option<CrashAnalysisItem> {
    let mut versions: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    for (index_no, line) in index.raw_lines().iter().enumerate() {
        let Some((modid, version)) = parse_mod_list_line(line) else {
            continue;
        };
        versions
            .entry(modid)
            .or_default()
            .push((index_no + 1, version));
    }
    let mut duplicated: Vec<(String, Vec<(usize, String)>)> = versions
        .into_iter()
        .filter(|(_, entries)| {
            let mut distinct: Vec<&String> = entries.iter().map(|(_, v)| v).collect();
            distinct.sort();
            distinct.dedup();
            distinct.len() >= 2
        })
        .collect();
    if duplicated.is_empty() {
        return None;
    }
    duplicated.sort_by(|a, b| a.0.cmp(&b.0));
    let (modid, entries) = duplicated.remove(0);
    let evidence: Vec<CrashEvidence> = entries
        .iter()
        .take(2)
        .filter_map(|(line_no, _)| index.evidence(*line_no))
        .collect();
    if evidence.is_empty() {
        return None;
    }
    Some(item(
        "duplicate_mod",
        "mod_conflict",
        "warning",
        format!("同名 mod 安装了多个版本：{}", modid),
        "high",
        evidence,
        String::new(),
        "同一 mod 的多个版本会互相覆盖注册表条目，表现为随机崩溃或某些功能时好时坏。",
        "到 mods 目录里删掉旧版本，只保留一个；启动器日志里能直接看到重复的两行。",
        "mod-duplicate",
    ))
}

/// 解析 mod 列表行：`\t- modid 1.2.3` / `- modid 1.2.3`
fn parse_mod_list_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("- ")?;
    let mut parts = rest.split_whitespace();
    let modid = parts.next()?;
    let version = parts.next()?;
    if modid.is_empty() || version.is_empty() {
        return None;
    }
    if !version
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        return None;
    }
    Some((modid.to_string(), version.to_string()))
}

/// 从一行里取出引号内容；`hint` 用于挑选更贴近语义的那一处
fn extract_quoted(line: &str, hint: &str) -> Option<String> {
    let mut values: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut inside = false;
    for ch in line.chars() {
        if ch == '\'' {
            if inside {
                values.push(current.clone());
                current.clear();
            }
            inside = !inside;
            continue;
        }
        if inside {
            current.push(ch);
        }
    }
    if values.is_empty() {
        return None;
    }
    let lower_hint = hint.to_lowercase();
    if lower_hint == "dependency" {
        // 「Mod 'create' is missing dependency 'flywheel'」→ 取最后一个引号内容
        return values.last().cloned();
    }
    values.first().cloned()
}

#[allow(clippy::too_many_arguments)]
fn item(
    cause: &str,
    category: &str,
    severity: &str,
    title: String,
    confidence: &str,
    evidence: Vec<CrashEvidence>,
    missing_evidence: String,
    impact: &str,
    suggestion: &str,
    rule_id: &str,
) -> CrashAnalysisItem {
    let detail = evidence
        .first()
        .map(|e| e.raw_line.clone())
        .unwrap_or_default();
    CrashAnalysisItem {
        cause: cause.to_string(),
        category: category.to_string(),
        severity: severity.to_string(),
        title,
        detail,
        suggestion: suggestion.to_string(),
        impact: impact.to_string(),
        confidence: confidence.to_string(),
        evidence,
        missing_evidence,
        rule_id: rule_id.to_string(),
        actions: Vec::new(),
        refs: Vec::new(),
        suspects: Vec::new(),
    }
}

#[cfg(test)]
#[path = "special_test.rs"]
mod special_test;
