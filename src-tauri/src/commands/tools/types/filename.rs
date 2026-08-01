use serde::{Deserialize, Serialize};

/// 从 URL 获取文件名请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchFilenameParams {
    pub url: String,
}

/// 从 URL 获取文件名结果
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchFilenameResult {
    pub filename: String,
    pub file_size: u64,
}
