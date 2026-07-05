//! Download types

use serde::{Deserialize, Serialize};

/// 下载任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub urls: Vec<String>,
    pub local_path: String,
    pub expected_size: i64,
    pub expected_hash: Option<String>,
}

/// 下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub task_id: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed: u64,
    pub status: DownloadStatus,
    pub error: Option<String>,
}

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Waiting,
    Connecting,
    Downloading,
    Verifying,
    Completed,
    Failed,
    Skipped,
}

/// 全局下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub skipped_files: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub current_speed: u64,
    pub is_active: bool,
}

impl Default for GlobalProgress {
    fn default() -> Self {
        Self {
            total_files: 0,
            completed_files: 0,
            failed_files: 0,
            skipped_files: 0,
            total_bytes: 0,
            downloaded_bytes: 0,
            current_speed: 0,
            is_active: false,
        }
    }
}
