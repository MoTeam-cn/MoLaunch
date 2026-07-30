use crate::state::{AppState, DownloadState, StageStatus};
use tauri::{AppHandle, State};

use super::types::{DownloadProgressSnapshot, DownloadStageSnapshot};
use crate::utils::dispatcher::ActionRequest;

/// 统一下载进度 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::version_progress_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn version_progress_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::version_progress_manager::dispatch(state, app, req).await
}


/// Get download progress
pub async fn get_download_progress(state: &AppState) -> Result<DownloadProgressSnapshot, String> {
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
pub async fn is_downloading(state: &AppState) -> Result<bool, String> {
    let ds = state.download_state.lock().unwrap();
    Ok(ds.is_active)
}

/// Reset download progress
pub async fn reset_download_progress(state: &AppState) -> Result<(), String> {
    let mut ds = state.download_state.lock().unwrap();
    *ds = DownloadState::default();
    Ok(())
}

/// 取消下载（设置 cancel_flag，正在进行的下载会尽快中止）
pub async fn cancel_download(state: &AppState) -> Result<(), String> {
    state
        .download_cancel_flag
        .store(true, std::sync::atomic::Ordering::Relaxed);
    // 同时清除暂停状态
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    // 广播当前状态（前端 WS 据此感知取消）
    broadcast_current(state);
    Ok(())
}

/// 暂停下载（设置 pause_flag，新任务不再开始，已进行的任务完成当前文件后等待）
pub async fn pause_download(state: &AppState) -> Result<(), String> {
    state
        .download_pause_flag
        .store(true, std::sync::atomic::Ordering::Relaxed);
    // 广播暂停状态（stages 中 is_paused=true，前端 UI 切换为暂停图标）
    broadcast_current(state);
    Ok(())
}

/// 恢复下载（清除 pause_flag）
pub async fn resume_download(state: &AppState) -> Result<(), String> {
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    // 广播恢复状态（stages 中 is_paused=null，前端 UI 切换为下载图标）
    broadcast_current(state);
    Ok(())
}

/// 构造当前下载状态 snapshot 并广播到 WS（供 cancel/pause/resume 调用）
///
/// 注：cancel/pause/resume 只改 atomic flag，不写 download_state。
/// 此函数读取当前 download_state + pause_flag 构造 snapshot，
/// 让前端通过 WS 即时感知控制信号变化，无需轮询。
///
/// 复用 `super::download::broadcast_current`，避免重复实现。
fn broadcast_current(state: &AppState) {
    super::download::broadcast_current(state);
}
