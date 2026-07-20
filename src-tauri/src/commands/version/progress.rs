use crate::state::{AppState, DownloadState, StageStatus};
use tauri::State;

use super::types::{DownloadProgressSnapshot, DownloadStageSnapshot};

/// Get download progress
#[tauri::command]
pub async fn get_download_progress(
    state: State<'_, AppState>,
) -> Result<DownloadProgressSnapshot, String> {
    let ds = state.download_state.lock().unwrap();
    let is_paused = state
        .download_pause_flag
        .load(std::sync::atomic::Ordering::Relaxed);
    Ok(DownloadProgressSnapshot {
        stages: ds
            .stages
            .iter()
            .map(|s| DownloadStageSnapshot {
                name: s.name.clone(),
                progress: s.progress,
                weight: s.weight,
                status: match s.status {
                    StageStatus::Waiting => "waiting".to_string(),
                    StageStatus::Loading => "loading".to_string(),
                    StageStatus::Finished => "finished".to_string(),
                    StageStatus::Failed => "failed".to_string(),
                },
                bytes_downloaded: s.bytes_downloaded,
                bytes_total: s.bytes_total,
                files_downloaded: s.files_downloaded,
                files_total: s.files_total,
                group: s.group.clone(),
                is_paused: if is_paused { Some(true) } else { None },
            })
            .collect(),
        current_stage_index: ds.current_stage_index,
        global_speed: ds.global_speed,
        global_bytes_downloaded: ds.global_bytes_downloaded,
        global_bytes_total: ds.global_bytes_total,
        is_active: ds.is_active,
        is_complete: ds.is_complete,
        error_code: ds.error_code,
        version_name: ds.version_name.clone(),
    })
}

/// Check if downloading
#[tauri::command]
pub async fn is_downloading(state: State<'_, AppState>) -> Result<bool, String> {
    let ds = state.download_state.lock().unwrap();
    Ok(ds.is_active)
}

/// Reset download progress
#[tauri::command]
pub async fn reset_download_progress(state: State<'_, AppState>) -> Result<(), String> {
    let mut ds = state.download_state.lock().unwrap();
    *ds = DownloadState::default();
    Ok(())
}

/// 取消下载（设置 cancel_flag，正在进行的下载会尽快中止）
#[tauri::command]
pub async fn cancel_download(state: State<'_, AppState>) -> Result<(), String> {
    state
        .download_cancel_flag
        .store(true, std::sync::atomic::Ordering::Relaxed);
    // 同时清除暂停状态
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

/// 暂停下载（设置 pause_flag，新任务不再开始，已进行的任务完成当前文件后等待）
#[tauri::command]
pub async fn pause_download(state: State<'_, AppState>) -> Result<(), String> {
    state
        .download_pause_flag
        .store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

/// 恢复下载（清除 pause_flag）
#[tauri::command]
pub async fn resume_download(state: State<'_, AppState>) -> Result<(), String> {
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
