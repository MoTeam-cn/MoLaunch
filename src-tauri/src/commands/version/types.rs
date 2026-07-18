use serde::{Deserialize, Serialize};

/// Version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    pub version_type: String,
    pub release_time: i64, // Unix时间戳
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Version list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionListResult {
    pub versions: Vec<VersionInfo>,
    pub latest_release: String,
    pub latest_snapshot: String,
    pub source_name: String,
}

/// Download progress snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStageSnapshot {
    pub name: String,
    pub progress: f64,
    pub weight: f64,
    pub status: String,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub files_downloaded: usize,
    pub files_total: usize,
    /// 所属分组（用于前端按 "MC本体安装" 等分组折叠展开）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// 是否已暂停（由前端 pause 按钮控制，用于显示暂停状态）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_paused: Option<bool>,
}

/// Download progress snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressSnapshot {
    pub stages: Vec<DownloadStageSnapshot>,
    pub current_stage_index: usize,
    pub global_speed: u64,
    pub global_bytes_downloaded: u64,
    pub global_bytes_total: u64,
    pub is_active: bool,
    pub is_complete: bool,
    pub error_code: i32,
}
