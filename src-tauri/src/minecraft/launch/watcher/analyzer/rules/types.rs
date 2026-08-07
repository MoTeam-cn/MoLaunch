//! 声明式崩溃检测规则表（数据驱动）
//!
//! 规则与检测逻辑分离：每条规则声明匹配来源、关键字模式、置信度、分类与结论文案。
//! 检测器（detector.rs）遍历本表产出证据，评分器（scorer.rs）按置信度聚合结论。
//! 与早期"顺序短路 if 链"的实现形态不同：此处规则是数据而非流程，新增规则只需追加条目。

use super::super::super::types::CrashCategory;

/// 证据来源
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// 运行时日志
    RuntimeLog,
    /// 崩溃报告（crash-reports/*.txt）
    CrashReport,
    /// JVM 崩溃日志（hs_err_pid*.log）
    HsErr,
}

impl SourceKind {
    pub fn name(self) -> &'static str {
        match self {
            SourceKind::RuntimeLog => "runtime_log",
            SourceKind::CrashReport => "crash_report",
            SourceKind::HsErr => "hs_err",
        }
    }
}

/// 关键字规则：在指定来源中命中 patterns（任一）且满足 and_patterns（全部）时触发
pub struct KeywordRule {
    /// 规则标识（日志输出用）
    pub id: &'static str,
    /// 匹配来源
    pub source: SourceKind,
    /// 命中关键字（小写不敏感，任一命中即触发）
    pub patterns: &'static [&'static str],
    /// 附加条件（小写不敏感，全部命中才触发；空表表示不要求）
    pub and_patterns: &'static [&'static str],
    /// 崩溃分类
    pub category: CrashCategory,
    /// 置信度 0.0-1.0
    pub confidence: f64,
    /// 崩溃原因摘要
    pub reason: &'static str,
    /// 建议解决方案
    pub suggestion: &'static str,
}
