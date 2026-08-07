//! 单文件下载结果构造与最终校验

use super::super::super::super::utils::file_checker::FileChecker;
use super::super::super::types::{DownloadProgress, DownloadStatus, DownloadTask};

pub(super) fn skipped(task: &DownloadTask) -> DownloadProgress {
    DownloadProgress {
        task_id: task.id.clone(),
        downloaded: 0,
        total: task.expected_size.max(0) as u64,
        speed: 0,
        status: DownloadStatus::Skipped,
        error: None,
    }
}

pub(super) fn completed(
    task: &DownloadTask,
    downloaded: u64,
    total: u64,
    speed: u64,
) -> DownloadProgress {
    DownloadProgress {
        task_id: task.id.clone(),
        downloaded,
        total,
        speed,
        status: DownloadStatus::Completed,
        error: None,
    }
}

pub(super) fn failed(task: &DownloadTask, error: String) -> DownloadProgress {
    DownloadProgress {
        task_id: task.id.clone(),
        downloaded: 0,
        total: 0,
        speed: 0,
        status: DownloadStatus::Failed,
        error: Some(error),
    }
}

pub(super) fn check_chunk(task: &DownloadTask, total: u64) -> Option<String> {
    FileChecker::new()
        .with_actual_size(total as i64)
        .with_hash(task.expected_hash.clone())
        .check(&task.local_path)
}

pub(super) fn check_stream(task: &DownloadTask, file_size: u64, downloaded: u64) -> Option<String> {
    FileChecker::new()
        .with_actual_size(if file_size > 0 {
            file_size as i64
        } else {
            downloaded as i64
        })
        .with_hash(task.expected_hash.clone())
        .check(&task.local_path)
}
