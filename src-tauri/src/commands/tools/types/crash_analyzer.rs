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
    /// 已确认的崩溃分析条目（每条都带证据行）
    pub analyses: Vec<CrashAnalysisItem>,
    /// 待确认条目：规则命中了关键词，但找不到可引用的日志行，或证据不足
    #[serde(default)]
    pub pending: Vec<CrashAnalysisItem>,
    /// 日志总行数（前端用于展示与「输出极短」判定说明）
    #[serde(default)]
    pub line_count: usize,
}

/// 一条日志证据：行号 + 原文（原文是日志的子串，不做任何改写）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashEvidence {
    /// 行号（从 1 开始）
    pub line_no: usize,
    /// 该行原文（已去掉首尾空白，内容与日志一致）
    pub raw_line: String,
}

/// 单个崩溃分析条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashAnalysisItem {
    /// 归因键：同一 cause 只输出一条（用于去重与合并证据）
    pub cause: String,
    /// 分类：java_version / missing_mod / memory / driver / mod_conflict / world / server / plugin / other
    pub category: String,
    /// 严重级别：error / warning / info
    pub severity: String,
    /// 标题
    pub title: String,
    /// 首条证据原文（保留旧字段，等价于 evidence[0].raw_line）
    pub detail: String,
    /// 中文修复建议
    pub suggestion: String,
    /// 影响说明
    #[serde(default)]
    pub impact: String,
    /// 置信度：high / medium / low
    #[serde(default)]
    pub confidence: String,
    /// 证据行（≥1 条；为空表示未确认，只进 pending）
    #[serde(default)]
    pub evidence: Vec<CrashEvidence>,
    /// 置信度不足时说明还缺什么证据
    #[serde(default)]
    pub missing_evidence: String,
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

impl CrashAnalysisItem {
    /// 首条证据行号（排序用；无证据返回 usize::MAX）
    pub fn first_line_no(&self) -> usize {
        self.evidence
            .first()
            .map(|e| e.line_no)
            .unwrap_or(usize::MAX)
    }
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
