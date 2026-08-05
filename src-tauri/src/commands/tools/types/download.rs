use serde::{Deserialize, Serialize};

/// 外部下载请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadFileParams {
    pub url: String,
    pub file_name: String,
    /// 自定义请求 User-Agent（为空时使用默认 UA）
    #[serde(default)]
    pub user_agent: Option<String>,
    /// 文件级并发线程数（覆盖全局下载配置）
    #[serde(default)]
    pub max_threads: Option<u32>,
    /// 单文件分片数（覆盖全局下载配置，0/1 = 单流）
    #[serde(default)]
    pub chunk_count: Option<u32>,
    /// 全局限速 bytes/秒（0 = 不限速，覆盖全局下载配置）
    #[serde(default)]
    pub max_speed: Option<u64>,
}

/// 删除已下载文件请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDownloadParams {
    pub file_name: String,
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
