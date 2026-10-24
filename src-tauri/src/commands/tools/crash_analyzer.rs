//! 崩溃日志分析
//!
//! 流程：规则引擎（证据驱动）→ 兜底关键词 → 合并去重。
//! 所有结论都必须带至少一条日志原文证据，拿不到证据的进「待确认」。

use crate::log_info;
use crate::state::AppState;

use super::crash_rules::{self, evidence::LogIndex};
use super::types::{
    CrashAnalysisItem, CrashAnalyzeParams, CrashAnalyzeResult, CrashEvidence,
    CrashRuleFeedbackParams,
};

/// 分析崩溃日志文本，识别常见崩溃模式（仅返回已确认条目）
pub fn analyze_log_text(log_text: &str) -> Vec<CrashAnalysisItem> {
    analyze_log_text_with_mods(log_text, None)
}

/// 分析崩溃日志，附带 mod 索引用于反查可疑 mod
///
/// 规则引擎命中即返回；完全没命中时才回退到内置关键词兜底，保证不至于空手而归。
pub fn analyze_log_text_with_mods(
    log_text: &str,
    mod_index: Option<&std::collections::HashMap<String, String>>,
) -> Vec<CrashAnalysisItem> {
    analyze_log_text_full(log_text, mod_index).analyses
}

/// 完整分析：已确认条目 + 待确认条目 + 日志行数
pub fn analyze_log_text_full(
    log_text: &str,
    mod_index: Option<&std::collections::HashMap<String, String>>,
) -> CrashAnalyzeResult {
    let line_count = log_text.lines().count();
    if log_text.trim().is_empty() {
        return CrashAnalyzeResult {
            analyses: Vec::new(),
            pending: Vec::new(),
            line_count,
        };
    }

    let (mut analyses, pending) = crash_rules::analyze_with_pending(log_text, mod_index);
    if analyses.is_empty() && pending.is_empty() {
        analyses = legacy_analyze(log_text);
    }
    log_info!(
        "[CrashAnalyzer] 规则引擎 v{} 产出 {} 条结论（待确认 {} 条，日志 {} 行）",
        crash_rules::rules_version(),
        analyses.len(),
        pending.len(),
        line_count
    );
    CrashAnalyzeResult {
        analyses,
        pending,
        line_count,
    }
}

/// 关键词兜底分析（规则库完全没命中时使用）
fn legacy_analyze(log_text: &str) -> Vec<CrashAnalysisItem> {
    let index = LogIndex::new(log_text);
    let mut analyses: Vec<CrashAnalysisItem> = Vec::new();

    // 1. Java 版本不匹配
    if let Some(item) = legacy_item(
        &index,
        &[
            "unsupported class file major version",
            "has been compiled by a more recent version",
            "unsupportedclassversionerror",
        ],
        "legacy-java-version",
        "java_version",
        "error",
        "Java 版本不匹配",
        "当前 Java 版本与 mod / 游戏所需版本不一致，游戏在类加载阶段就中断。",
        "在启动器设置里切换到匹配的 Java 版本；较新的 mod 通常需要更高版本。",
    ) {
        analyses.push(item);
    }

    // 2. 缺失 mod / 类缺失
    if let Some(item) = legacy_item(
        &index,
        &["noclassdeffounderror", "nosuchmethoderror"],
        "legacy-missing-mod",
        "missing_mod",
        "error",
        "缺失 Mod 或 Mod 文件损坏",
        "游戏引用的类 / 方法找不到，通常是缺少前置 mod 或 mod 文件损坏。",
        "检查是否漏装前置 mod；依赖齐全则重新下载相关 mod。",
    ) {
        analyses.push(item);
    }

    // 3. 内存不足
    if let Some(item) = legacy_item(
        &index,
        &["outofmemoryerror", "outofmemory"],
        "legacy-memory",
        "memory",
        "error",
        "内存不足",
        "JVM 内存不足，游戏随时会被强杀或卡死。",
        "在版本设置里增大内存分配，并确认物理内存充足。",
    ) {
        analyses.push(item);
    }

    // 4. 显卡驱动
    if let Some(item) = legacy_item(
        &index,
        &["glfw", "opengl", "pixel format"],
        "legacy-driver",
        "driver",
        "warning",
        "显卡驱动 / 图形库问题",
        "图形相关错误，可能是驱动过旧或不兼容，表现为启动即崩或画面异常。",
        "更新显卡驱动到最新版本，或尝试切换使用的显卡。",
    ) {
        analyses.push(item);
    }

    analyses
}

/// 兜底条目构造：命中任一关键词才产出，证据取首个命中行
#[allow(clippy::too_many_arguments)]
fn legacy_item(
    index: &LogIndex,
    needles: &[&str],
    cause: &str,
    category: &str,
    severity: &str,
    title: &str,
    impact: &str,
    suggestion: &str,
) -> Option<CrashAnalysisItem> {
    let owned: Vec<String> = needles.iter().map(|s| s.to_string()).collect();
    let evidence: Vec<CrashEvidence> = index.collect(&owned, &[], 2);
    if evidence.is_empty() {
        return None;
    }
    Some(CrashAnalysisItem {
        cause: cause.to_string(),
        category: category.to_string(),
        severity: severity.to_string(),
        title: title.to_string(),
        detail: evidence
            .first()
            .map(|e| e.raw_line.clone())
            .unwrap_or_default(),
        suggestion: suggestion.to_string(),
        impact: impact.to_string(),
        confidence: "medium".to_string(),
        evidence,
        missing_evidence: String::new(),
        rule_id: String::new(),
        actions: Vec::new(),
        refs: Vec::new(),
        suspects: Vec::new(),
    })
}

/// 崩溃日志分析命令入口（包装纯函数，返回 JSON）
pub async fn analyze(
    state: &AppState,
    params: CrashAnalyzeParams,
) -> Result<serde_json::Value, String> {
    log_info!(
        "[CrashAnalyzer] 开始分析日志，长度 {} 字节",
        params.log_text.len()
    );
    // 提供版本 ID 时读取该版本的 mods 目录，用于反查可疑 mod（读不到就退化为纯文本分析）
    let mod_index = match params
        .version_id
        .as_ref()
        .filter(|id| !id.trim().is_empty())
    {
        Some(version_id) => {
            let game_dir = crate::state::resolve_game_dir_from_state(state).await;
            let mods_dir = game_dir.join("versions").join(version_id).join("mods");
            let index = crate::minecraft::mods::load_mod_index(&mods_dir);
            if index.is_empty() {
                None
            } else {
                Some(index)
            }
        }
        None => None,
    };

    let result = analyze_log_text_full(&params.log_text, mod_index.as_ref());
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 在日志中定位首个命中关键词（大小写不敏感）的行，返回该行及其前后各 `context` 行的
/// 上下文窗口（含行号标注），供 AI 分析使用。
///
/// - 命中：返回 `(起始行号, 窗口文本)`；行号从 1 起。
/// - 未命中：返回 `(None, "")`。
pub fn locate_keyword_context(
    log_text: &str,
    keyword: &str,
    context: usize,
) -> (Option<usize>, String) {
    let lines: Vec<&str> = log_text.lines().collect();
    let needle_l = keyword.to_lowercase();
    let mut hit_line: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() && line.to_lowercase().contains(&needle_l) {
            hit_line = Some(i);
            break;
        }
    }
    let Some(idx) = hit_line else {
        return (None, String::new());
    };
    let start = idx.saturating_sub(context);
    let end = (idx + context).min(lines.len().saturating_sub(1));
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate().take(end + 1).skip(start) {
        out.push_str(&format!("{:>6}| {}\n", i + 1, line));
    }
    (Some(idx + 1), out)
}

/// 记录一条「规则判断不对」的反馈
///
/// 只写本地启动器日志，不上报任何内容：用户反馈问题时把日志附上即可。
pub fn report_rule_feedback(params: &CrashRuleFeedbackParams) -> Result<serde_json::Value, String> {
    crate::log_warn!(
        "[CrashRules] 用户标记误报: rule={} category={:?} note={:?}",
        params.rule_id,
        params.category,
        params.note
    );
    Ok(serde_json::json!({ "recorded": true }))
}
