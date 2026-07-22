//! 工具模块的统一请求 / 响应类型定义
//!
//! - `ToolsRequest` 作为 `tools_manager` IPC 入口的统一请求体
//! - 各子模块的参数 / 响应类型集中声明在此，便于跨模块复用与序列化

use serde::{Deserialize, Serialize};

/// 统一请求体
///
/// `action` 决定分发到哪个子模块函数，`params` 由对应子模块自行反序列化。
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolsRequest {
    pub action: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

// ===== 外部下载相关 =====

/// 外部下载请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadFileParams {
    pub url: String,
    pub file_name: String,
}

/// 删除已下载文件请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDownloadParams {
    pub file_name: String,
}

/// 从 URL 获取文件名请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchFilenameParams {
    pub url: String,
}

/// 外部下载结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalDownloadResult {
    pub path: String,
    pub size: u64,
    pub file_name: String,
}

/// 已下载文件条目
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalDownloadEntry {
    pub name: String,
    pub size: u64,
    /// Unix 时间戳（秒）
    pub modified: u64,
}

/// 从 URL 获取文件名结果
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchFilenameResult {
    pub filename: String,
    pub file_size: u64,
}

// ===== 清理相关 =====

/// 单个可清理项
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupItem {
    pub path: String,
    pub display_name: String,
    pub category: String,
    pub size: u64,
    pub file_count: u64,
}

/// 清理扫描结果
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupScanResult {
    pub items: Vec<CleanupItem>,
    pub total_size: u64,
    pub total_files: u64,
}

/// 清理执行请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupExecuteParams {
    pub paths: Vec<String>,
}

/// 清理执行失败项
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupFailedItem {
    pub path: String,
    pub error: String,
}

/// 清理执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupExecuteResult {
    pub cleaned_size: u64,
    pub cleaned_files: u64,
    pub failed: Vec<CleanupFailedItem>,
}

// ===== 内存优化相关 =====

/// 内存优化请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryOptimizeParams {
    /// 优化模式："light"（轻量）或 "strong"（强力）
    pub mode: String,
}

/// 内存优化结果（所有字段单位：字节）
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryOptimizeResult {
    pub freed_bytes: u64,
    pub before_bytes: u64,
    pub after_bytes: u64,
    /// 本次优化使用的模式："light" / "strong"
    pub mode: String,
}
