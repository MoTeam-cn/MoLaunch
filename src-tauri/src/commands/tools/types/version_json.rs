use serde::{Deserialize, Serialize};

/// 版本 JSON 读取请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionJsonReadParams {
    /// 版本 ID
    pub version_id: String,
}

/// 版本 JSON 读取结果
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionJsonReadResult {
    /// 文件内容
    pub content: String,
    /// 文件路径
    pub path: String,
}

/// 版本 JSON 保存请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionJsonSaveParams {
    /// 版本 ID
    pub version_id: String,
    /// JSON 内容
    pub content: String,
}

/// 版本 JSON 保存结果
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionJsonSaveResult {
    /// 是否成功
    pub success: bool,
}
