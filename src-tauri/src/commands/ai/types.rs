//! AI 相关 IPC 类型

use serde::{Deserialize, Serialize};

/// 崩溃日志 AI 分析参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeCrashParams {
    /// 运行时日志全文
    pub runtime_log: String,
    /// 错误级别日志行
    #[serde(default)]
    pub error_lines: Vec<String>,
    /// 崩溃报告文本
    #[serde(default)]
    pub crash_report: String,
    /// hs_err 日志文本
    #[serde(default)]
    pub hs_err: String,
    /// 可选显式指定模型；为空时使用默认模型
    #[serde(default)]
    pub model: Option<String>,
}

/// AI 分析结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAnalysisResult {
    /// 模型回复（Markdown）
    pub content: String,
    /// 使用的模型名
    pub model: String,
    /// 耗时毫秒
    pub elapsed_ms: u64,
}

/// 连接状态
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatusResult {
    /// 是否可用
    pub available: bool,
    /// 服务地址
    pub base_url: String,
    /// 默认模型名
    pub model: String,
}

/// 服务探测参数（前端表单当前值，避免未保存时探测旧配置）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProbeParams {
    /// 服务地址
    pub base_url: String,
    /// API Key
    #[serde(default)]
    pub api_key: String,
    /// 超时秒数
    pub timeout_secs: u64,
}
