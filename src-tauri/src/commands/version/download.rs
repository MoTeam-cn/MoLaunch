use crate::minecraft::download::{self, types as download_types};
use crate::state::{AppState, DownloadStage, DownloadState, StageStatus};
use crate::{log_error, log_info};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use super::sanitize_version_id;
use super::types::DownloadStageSnapshot;

/// Download version
///
/// 注：已聚合为 `version_install_manager` IPC 入口，本函数由
/// `install_manager::dispatch` 反序列化参数后调用。
pub async fn download_version(
    app: &AppHandle,
    state: &AppState,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Downloading version: {}", version_id);

    // 重置 cancel/pause flag（确保每次下载都是干净状态）
    state
        .download_cancel_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);

    // 修复 stages bug：之前只 clear 不重建，progress_callback 中 ds.stages[idx] 越界
    // 统一用 reset_stages 注册 5 个 MC 本体 stages（与 download_version_full 的 stage_callback 索引对应）
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.reset_stages(vec![
            DownloadStage::new_grouped("版本清单", 5.0, "MC本体安装"),
            DownloadStage::new_grouped("版本信息", 5.0, "MC本体安装"),
            DownloadStage::new_grouped("客户端", 30.0, "MC本体安装"),
            DownloadStage::new_grouped("库文件", 30.0, "MC本体安装"),
            DownloadStage::new_grouped("资源文件", 30.0, "MC本体安装"),
        ]);
        ds.version_name = version_id.clone();
    }

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    // 注意：source_mode 用 config.download.source（文件下载源），保持原行为
    // fetch_version_list / fetch_with_retry 用这个 source_mode 决定元数据获取方式
    let (mirror_url, source_mode) = {
        let config = state.config.lock().await;
        let mirror_url = config.download.mirror_url.clone();
        let source_mode =
            crate::minecraft::sources::DownloadSourceMode::from_str(&config.download.source);
        (mirror_url, source_mode)
    };

    let game_path = game_dir.as_path();
    let app_clone = app.clone();
    let version_id_clone = version_id.clone();
    let state_clone = state.download_state.clone();

    // 统一进度回调：sync_stage_from_progress 替代手工累加
    // 删除 accumulated_bytes/accumulated_total/speed_window：DownloadManager 内部 timer 已计算 current_speed
    let state_for_progress = state_clone.clone();
    let pause_flag_for_cb = state.download_pause_flag.clone();
    let progress_callback = Arc::new(move |progress: download_types::GlobalProgress| {
        {
            let mut ds = state_for_progress.lock().unwrap();
            ds.is_active = progress.is_active;
            let idx = ds.current_stage_index;
            ds.sync_stage_from_progress(
                idx,
                progress.downloaded_bytes,
                progress.total_bytes,
                progress.completed_files,
                progress.total_files,
                progress.current_speed,
            );
        }
        let ds = state_for_progress.lock().unwrap();
        let is_paused = pause_flag_for_cb.load(std::sync::atomic::Ordering::Relaxed);
        let snapshot = build_snapshot(&ds, &version_id_clone, is_paused);
        drop(ds);
        // Tauri plugin event 推送进度（前端监听 download-progress）
        let _ = app_clone.emit("download-progress", &snapshot);
    });

    // Stage callback：统一用 set_current_stage 切换阶段（与 install_merged 行为一致）
    let state_for_stage = state_clone.clone();
    let app_for_stage = app.clone();
    let vid_for_stage = version_id.clone();
    let pause_flag_for_stage = state.download_pause_flag.clone();
    let stage_callback = Arc::new(move |stage_index: usize, _stage_name: &str| {
        let mut ds = state_for_stage.lock().unwrap();
        ds.set_current_stage(stage_index);
        let is_paused = pause_flag_for_stage.load(std::sync::atomic::Ordering::Relaxed);
        let snapshot = build_snapshot(&ds, &vid_for_stage, is_paused);
        drop(ds);
        let _ = app_for_stage.emit("download-progress", &snapshot);
    });

    // Full download flow
    let result = download::download_version_full(
        state,
        &version_id,
        game_path,
        mirror_url.as_deref(),
        source_mode,
        Some(progress_callback),
        Some(stage_callback),
    )
    .await
    .map_err(|e| {
        log_error!("Failed to download version: {}", e);
        // 重置 download_state，避免 is_active 仍为 true 导致前端下载管理页卡住
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(0);
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
        ds.mark_complete();
    }

    // 广播最终完成状态（emit is_complete=true，前端据此触发 finishDownload）
    {
        let ds = state.download_state.lock().unwrap();
        let snapshot = build_snapshot(&ds, &version_id, false);
        drop(ds);
        let _ = app.emit("download-progress", &snapshot);
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

/// 构造当前 download_state snapshot 并广播到前端
///
/// 供各下载路径的 progress_callback 调用，确保进度推送覆盖所有下载路径
/// （MC 本体 / 整合包安装 / 资源下载 / 暂停恢复取消）。调用方只需传入 `&AppState`。
pub fn broadcast_current(state: &AppState) {
    let ds = state.download_state.lock().unwrap();
    let is_paused = state
        .download_pause_flag
        .load(std::sync::atomic::Ordering::Relaxed);
    let version_name = ds.version_name.clone();
    let snapshot = build_snapshot(&ds, &version_name, is_paused);
    drop(ds);
    // AppHandle 在 Tauri setup 钩子注入，此处无需调用方持有
    if let Some(app) = state.app_handle.get() {
        let _ = app.emit("download-progress", &snapshot);
    }
}

pub fn build_snapshot(ds: &DownloadState, version_id: &str, is_paused: bool) -> serde_json::Value {
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
            is_paused: if is_paused { Some(true) } else { None },
        })
        .collect();

    serde_json::json!({
        "version_id": version_id,
        "version_name": ds.version_name,
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
