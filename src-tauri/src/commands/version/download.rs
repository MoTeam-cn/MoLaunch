use crate::minecraft::download::{self, types as download_types};
use crate::state::{AppState, DownloadState, StageStatus};
use crate::{log_error, log_info};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tauri::{Emitter, State};

use super::sanitize_version_id;
use super::types::DownloadStageSnapshot;

/// Download version
#[tauri::command]
pub async fn download_version(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Downloading version: {}", version_id);

    // 清空上一次下载的 stages（避免累积）
    // 修复：之前只重置已有 stages 的状态，不清空数组，多次下载后 stages 越来越长
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.stages.clear();
        ds.is_active = true;
        ds.is_complete = false;
        ds.current_stage_index = 0;
        ds.global_speed = 0;
        ds.global_bytes_downloaded = 0;
        ds.global_bytes_total = 0;
        ds.error_code = 0;
        ds.version_name = version_id.clone();
    }

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let mirror_url = config.mirror_url.clone();
    let max_threads = config.max_download_threads as usize;
    let chunk_count = config.chunk_count as usize;
    let speed_limit = config.max_download_speed;
    let source_mode =
        crate::minecraft::sources::DownloadSourceMode::from_str(&config.download_source);
    drop(config);

    let game_path = game_dir.as_path();
    let app_clone = app.clone();
    let version_id_clone = version_id.clone();
    let state_clone = state.download_state.clone();

    // 滑动窗口速度计算
    let speed_window = Arc::new(std::sync::Mutex::new(VecDeque::<(u64, Instant)>::new()));

    // 跨阶段累计字节跟踪
    let accumulated_bytes = Arc::new(std::sync::Mutex::new(0u64));
    let accumulated_total = Arc::new(std::sync::Mutex::new(0u64));

    // Create progress callback
    let state_for_progress = state_clone.clone();
    let sw = speed_window.clone();
    let acc_bytes_for_progress = accumulated_bytes.clone();
    let acc_total_for_progress = accumulated_total.clone();
    let progress_callback = Arc::new(move |progress: download_types::GlobalProgress| {
        {
            let base_bytes = *acc_bytes_for_progress.lock().unwrap();
            let base_total = *acc_total_for_progress.lock().unwrap();
            let mut ds = state_for_progress.lock().unwrap();
            ds.is_active = progress.is_active;
            ds.global_bytes_downloaded = base_bytes + progress.downloaded_bytes;
            ds.global_bytes_total = base_total + progress.total_bytes;

            // 更新当前阶段的进度
            let idx = ds.current_stage_index;
            if idx < ds.stages.len() {
                let stage = &mut ds.stages[idx];
                stage.bytes_downloaded = progress.downloaded_bytes;
                stage.bytes_total = progress.total_bytes;
                stage.files_downloaded = progress.completed_files;
                stage.files_total = progress.total_files;
                if progress.total_bytes > 0 {
                    stage.progress =
                        (progress.downloaded_bytes as f64 / progress.total_bytes as f64).min(1.0);
                }
                stage.status = StageStatus::Loading;
            }

            // 滑动窗口速度计算
            {
                let mut window = sw.lock().unwrap();
                window.push_back((ds.global_bytes_downloaded, Instant::now()));
                if window.len() > 10 {
                    window.pop_front();
                }
                if window.len() >= 2 {
                    let (first_bytes, first_time) = window.front().unwrap();
                    let (last_bytes, last_time) = window.back().unwrap();
                    let bytes_diff = last_bytes.saturating_sub(*first_bytes);
                    let time_diff = last_time.duration_since(*first_time).as_secs_f64();
                    if time_diff > 0.0 {
                        ds.global_speed = (bytes_diff as f64 / time_diff) as u64;
                    }
                }
            }
        }
        let ds = state_for_progress.lock().unwrap();
        let snapshot = build_snapshot(&ds, &version_id_clone);
        drop(ds);
        let _ = app_clone.emit("download-progress", snapshot);
    });

    // Stage callback
    let state_for_stage = state_clone.clone();
    let app_for_stage = app.clone();
    let vid_for_stage = version_id.clone();
    let acc_bytes_for_stage = accumulated_bytes.clone();
    let acc_total_for_stage = accumulated_total.clone();
    let stage_callback = Arc::new(move |stage_index: usize, _stage_name: &str| {
        let mut ds = state_for_stage.lock().unwrap();
        // 标记上一个阶段完成，并累加字节
        if ds.current_stage_index < ds.stages.len() && stage_index > 0 {
            let prev = ds.current_stage_index;
            if prev < ds.stages.len() {
                ds.stages[prev].status = StageStatus::Finished;
                ds.stages[prev].progress = 1.0;
                *acc_bytes_for_stage.lock().unwrap() += ds.stages[prev].bytes_downloaded;
                *acc_total_for_stage.lock().unwrap() += ds.stages[prev].bytes_total;
            }
        }
        ds.current_stage_index = stage_index;
        if stage_index < ds.stages.len() {
            ds.stages[stage_index].status = StageStatus::Loading;
            ds.stages[stage_index].progress = 0.0;
        }
        let snapshot = build_snapshot(&ds, &vid_for_stage);
        drop(ds);
        let _ = app_for_stage.emit("download-progress", snapshot);
    });

    // Full download flow
    let result = download::download_version_full(
        &version_id,
        game_path,
        mirror_url.as_deref(),
        max_threads,
        chunk_count,
        speed_limit,
        source_mode,
        Some(progress_callback),
        Some(stage_callback),
        Some(state.download_cancel_flag.clone()),
        Some(state.download_pause_flag.clone()),
    )
    .await
    .map_err(|e| {
        log_error!("Failed to download version: {}", e);
        e.to_string()
    })?;

    log_info!(
        "Version {} download completed: libs {}/{}, assets {}/{}",
        version_id,
        result.libs_downloaded,
        result.libs_total,
        result.assets_downloaded,
        result.assets_total
    );

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = false;
        ds.is_complete = true;
        // 标记所有阶段完成
        for stage in ds.stages.iter_mut() {
            stage.status = StageStatus::Finished;
            stage.progress = 1.0;
        }
    }

    let _ = app.emit(
        "download-complete",
        serde_json::json!({
            "version_id": version_id,
            "libs_total": result.libs_total,
            "libs_downloaded": result.libs_downloaded,
            "libs_skipped": result.libs_skipped,
            "assets_total": result.assets_total,
            "assets_downloaded": result.assets_downloaded,
            "assets_skipped": result.assets_skipped,
        }),
    );

    Ok(())
}

pub fn build_snapshot(ds: &DownloadState, version_id: &str) -> serde_json::Value {
    let stages: Vec<DownloadStageSnapshot> = ds
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
            is_paused: None,
        })
        .collect();

    serde_json::json!({
        "version_id": version_id,
        "stages": stages,
        "current_stage_index": ds.current_stage_index,
        "global_speed": ds.global_speed,
        "global_bytes_downloaded": ds.global_bytes_downloaded,
        "global_bytes_total": ds.global_bytes_total,
        "is_active": ds.is_active,
        "is_complete": ds.is_complete,
        "error_code": ds.error_code,
    })
}
