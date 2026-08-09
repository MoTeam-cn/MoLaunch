//! MC 本体下载（install_merged 的步骤 1）
//!
//! 负责构建进度/阶段回调、调用 `download_version_full` 下载原版 MC，
//! 并在失败/取消时清理残留并重置下载状态。

use crate::commands::version::download::build_snapshot;
use crate::minecraft::download::{self as mc_download, types as download_types};
use crate::minecraft::sources::DownloadSourceMode;
use crate::state::{AppState, StageStatus};
use crate::{log_error, log_info, log_warn};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use super::cleanup::cleanup_failed_install;

/// 下载原版 MC 本体（install_merged 步骤 1）
///
/// - 构建进度/阶段回调，广播到 Tauri 事件 + WS 推送
/// - 失败时清理已下载的部分文件并重置 download_state
/// - 检查取消信号，标记 MC 下载阶段完成
#[allow(clippy::too_many_arguments)]
pub(crate) async fn download_base_mc(
    state: &AppState,
    app: &AppHandle,
    mc_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    download_source_mode: DownloadSourceMode,
    stage_offset: usize,
    fabric_version: Option<&str>,
) -> Result<mc_download::VersionDownloadResult, String> {
    // progress callback：统一用 sync_stage_from_progress 同步 GlobalProgress 到 download_state
    let state_clone = state.download_state.clone();
    let pause_flag_for_cb = state.download_pause_flag.clone();
    let app_for_cb = app.clone();
    let version_name_for_cb = state.download_state.lock().unwrap().version_name.clone();
    let progress_callback = Arc::new(move |progress: download_types::GlobalProgress| {
        let mut ds = state_clone.lock().unwrap();
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
        // 广播进度（Tauri plugin event 推送）
        let is_paused = pause_flag_for_cb.load(std::sync::atomic::Ordering::Relaxed);
        let snapshot = build_snapshot(&ds, &version_name_for_cb, is_paused);
        drop(ds);
        let _ = app_for_cb.emit("download-progress", &snapshot);
    });

    // Stage callback：统一用 set_current_stage 切换阶段
    let state_for_stage = state.download_state.clone();
    let pause_flag_for_stage = state.download_pause_flag.clone();
    let app_for_stage = app.clone();
    let version_name_for_stage = state.download_state.lock().unwrap().version_name.clone();
    let stage_callback = Arc::new(move |stage_index: usize, _stage_name: &str| {
        let actual_index = stage_offset + stage_index;
        let mut ds = state_for_stage.lock().unwrap();
        ds.set_current_stage(actual_index);
        // 广播阶段切换
        let is_paused = pause_flag_for_stage.load(std::sync::atomic::Ordering::Relaxed);
        let snapshot = build_snapshot(&ds, &version_name_for_stage, is_paused);
        drop(ds);
        let _ = app_for_stage.emit("download-progress", &snapshot);
    });

    log_info!("[Merged] Downloading base MC version: {}", mc_version);
    let result = mc_download::download_version_full(
        state,
        mc_version,
        game_dir,
        mirror_url,
        download_source_mode,
        Some(progress_callback),
        Some(stage_callback),
    )
    .await
    .map_err(|e| {
        log_error!("Failed to download MC version: {}", e);
        // MC 本体下载失败：清理已下载的部分文件
        cleanup_failed_install(game_dir, mc_version, fabric_version);
        // 重置 download_state，避免 is_active 仍为 true 导致前端下载管理页卡住
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(0);
        // 广播失败状态（emit error_code，前端据此停止监听 flow）
        let snapshot = build_snapshot(&ds, &ds.version_name, false);
        drop(ds);
        let _ = app.emit("download-progress", &snapshot);
        e.to_string()
    })?;

    log_info!(
        "[Merged] MC download completed: libs {}/{}, assets {}/{}",
        result.libs_downloaded,
        result.libs_total,
        result.assets_downloaded,
        result.assets_total
    );

    // 检查取消信号
    if state
        .download_cancel_flag
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        log_warn!("[Merged] 下载已被用户取消");
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(1);
        return Err("下载已取消".to_string());
    }

    // 标记 MC 下载阶段完成
    {
        let mut ds = state.download_state.lock().unwrap();
        for stage in ds.stages.iter_mut() {
            if stage.status == StageStatus::Loading {
                stage.status = StageStatus::Finished;
                stage.progress = 1.0;
            }
        }
    }

    Ok(result)
}
