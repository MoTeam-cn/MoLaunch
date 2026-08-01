use serde::{Deserialize, Serialize};

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
