use crate::state::{AppState, DownloadState, StageStatus};
use tauri::State;

use super::types::{DownloadProgressSnapshot, DownloadStageSnapshot};

/// Get download progress
#[tauri::command]
pub async fn get_download_progress(
    state: State<'_, AppState>,
) -> Result<DownloadProgressSnapshot, String> {
    let ds = state.download_state.lock().unwrap();
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
            })
            .collect(),
        current_stage_index: ds.current_stage_index,
        global_speed: ds.global_speed,
        global_bytes_downloaded: ds.global_bytes_downloaded,
        global_bytes_total: ds.global_bytes_total,
        is_active: ds.is_active,
        is_complete: ds.is_complete,
        error_code: ds.error_code,
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
