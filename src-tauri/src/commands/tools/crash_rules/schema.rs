//! 崩溃规则数据结构（对应 resources/crash_rules.json）

use serde::{Deserialize, Serialize};

/// 规则匹配条件
///
/// - `all`：全部关键词都必须出现
/// - `any`：至少出现一个
/// - `regex`：至少命中一条正则
/// - `none`：一个都不能出现（用于排除误报）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchSpec {
    #[serde(default)]
    pub all: Vec<String>,
    #[serde(default)]
    pub any: Vec<String>,
    #[serde(default)]
    pub regex: Vec<String>,
    #[serde(default)]
    pub none: Vec<String>,
}

/// 取证条件：命中这些关键词/正则的行会作为证据输出
///
/// 为空时回退到 `match.all` / `match.any` 的首个关键词，保证老规则也有证据行。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceSpec {
    #[serde(default)]
    pub any: Vec<String>,
    #[serde(default)]
    pub regex: Vec<String>,
}

/// 规则开关：不满足门槛时规则整体不参与判定（用于消除误报）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleGates {
    /// 日志行数下限（不足则禁用，例如「输出极短」要求 ≤200 行）
    #[serde(default)]
    pub min_lines: Option<usize>,
    /// 日志行数上限（超过则禁用）
    #[serde(default)]
    pub max_lines: Option<usize>,
    /// 必须出现的关键词（全部）
    #[serde(default)]
    pub require_present: Vec<String>,
    /// 必须**不**出现的关键词（任一出现则禁用）
    #[serde(default)]
    pub require_absent: Vec<String>,
}

/// 因果链中的一环
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChainStep {
    /// 该环节的说明（会以「 → 」串成标题）
    pub label: String,
    /// 命中任一关键词即视为该环节成立，并取其行作为证据
    #[serde(default)]
    pub any: Vec<String>,
}

/// 修复动作：由前端按 `type` 执行
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CrashAction {
    /// 打开外部链接
    OpenUrl { url: String },
    /// 打开当前版本的 mods 目录
    OpenModsDir,
    /// 跳转到版本设置的 Java 选择
    OpenSettingsJava,
    /// 跳转到版本设置的内存分配
    OpenSettingsMemory,
    /// 直接把 Java 大版本设为指定值
    SetJavaVersion { major: u32 },
    /// 直接把内存分配设为指定值（MB）
    SetMemory { mb: u32 },
}

/// 单条崩溃规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRule {
    pub id: String,
    pub category: String,
    pub severity: String,
    pub title: String,
    pub suggestion: String,
    /// 归因键：同一 cause 只输出一条（缺省用 id）
    #[serde(default)]
    pub cause: String,
    /// 置信度：high / medium / low（缺省 high）
    #[serde(default = "default_confidence")]
    pub confidence: String,
    /// 影响说明
    #[serde(default)]
    pub impact: String,
    /// 匹配条件（JSON 字段名为 `match`）
    #[serde(rename = "match", default)]
    pub match_spec: MatchSpec,
    /// 取证条件
    #[serde(default)]
    pub evidence: EvidenceSpec,
    /// 规则门槛
    #[serde(default)]
    pub gates: RuleGates,
    /// 因果链各环节（非空时标题由各环节 label 串成，输出 1 条而不是 N 条）
    #[serde(default)]
    pub steps: Vec<ChainStep>,
    #[serde(default)]
    pub actions: Vec<CrashAction>,
    /// 参考链接（Mojang bug、论坛帖等）
    #[serde(default)]
    pub refs: Vec<String>,
    /// 命中后尝试从日志反查相关 mod
    #[serde(default)]
    pub hint_mod: bool,
}

fn default_confidence() -> String {
    "high".to_string()
}

/// 规则集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub version: String,
    #[serde(default)]
    pub rules: Vec<CrashRule>,
}
