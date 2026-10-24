use serde::{Deserialize, Serialize};

/// 崩溃日志分析请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashAnalyzeParams {
    /// 崩溃日志原文
    pub log_text: String,
    /// 版本 ID（可选）：提供时会读取该版本的 mods 目录，用于反查可疑 mod
    #[serde(default)]
    pub version_id: Option<String>,
}

/// 崩溃日志分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashAnalyzeResult {
    /// 识别出的崩溃分析条目
    pub analyses: Vec<CrashAnalysisItem>,
}

/// 单个崩溃分析条目
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashAnalysisItem {
    /// 分类：java_version / missing_mod / memory / driver / mod_conflict / other
    pub category: String,
    /// 严重级别：error / warning / info
    pub severity: String,
    /// 标题
    pub title: String,
    /// 匹配到的相关行片段
    pub detail: String,
    /// 中文修复建议
    pub suggestion: String,
    /// 命中的规则 id（规则引擎产出时填充，兜底逻辑为空）
    #[serde(default)]
    pub rule_id: String,
    /// 可执行的修复动作（由前端按 type 执行）
    #[serde(default)]
    pub actions: Vec<super::super::crash_rules::schema::CrashAction>,
    /// 参考链接
    #[serde(default)]
    pub refs: Vec<String>,
    /// 疑似相关的 mod 显示名（来自 mods 目录索引）
    #[serde(default)]
    pub suspects: Vec<String>,
}
/// 规则误报反馈参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashRuleFeedbackParams {
    /// 被标记误报的规则 id
    pub rule_id: String,
    /// 分类（可选，便于日志检索）
    #[serde(default)]
    pub category: Option<String>,
    /// 用户补充说明（可选）
    #[serde(default)]
    pub note: Option<String>,
}
