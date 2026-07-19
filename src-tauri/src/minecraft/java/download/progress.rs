//! Java 下载进度事件

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use super::constants::JAVA_DOWNLOAD_PROGRESS_EVENT;

/// 下载进度事件 payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaDownloadProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub message: String,
}

/// 推送进度事件
///
/// `app` 为 None 时静默跳过，便于在无 AppHandle 的场景下复用底层逻辑。
pub fn emit(
    app: Option<&AppHandle>,
    stage: &str,
    current: usize,
    total: usize,
    bytes_downloaded: u64,
    bytes_total: u64,
    message: &str,
) {
    if let Some(handle) = app {
        let _ = handle.emit(
            JAVA_DOWNLOAD_PROGRESS_EVENT,
            JavaDownloadProgress {
                stage: stage.to_string(),
                current,
                total,
                bytes_downloaded,
                bytes_total,
                message: message.to_string(),
            },
        );
    }
}
