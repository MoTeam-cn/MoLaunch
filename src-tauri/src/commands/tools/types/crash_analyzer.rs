use serde::{Deserialize, Serialize};

/// 崩溃日志分析请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashAnalyzeParams {
    /// 崩溃日志原文
    pub log_text: String,
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
}
