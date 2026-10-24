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
    /// 匹配条件（JSON 字段名为 `match`）
    #[serde(rename = "match", default)]
    pub match_spec: MatchSpec,
    #[serde(default)]
    pub actions: Vec<CrashAction>,
    /// 参考链接（Mojang bug、论坛帖等）
    #[serde(default)]
    pub refs: Vec<String>,
    /// 命中后尝试从日志反查相关 mod
    #[serde(default)]
    pub hint_mod: bool,
}

/// 规则集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub version: String,
    #[serde(default)]
    pub rules: Vec<CrashRule>,
}
