//! install_merged：整合安装入口（MC 本体 + 加载器 + Fabric API + 后处理）
//!
//! 编排流程：
//!   1. 下载 MC 本体（版本清单/版本信息/客户端/库文件/资源文件）
//!   2. 批量安装加载器（stages.rs）
//!   3. 合并 JSON + 重命名版本目录（post_install.rs）
//!   4. 保存 setup.ini + 创建隔离目录（setup_persist.rs）
//!   5. 自动安装 Fabric API（fabric_api.rs）
//!
//! 各阶段的详细实现拆分到对应子模块，本文件只做编排。

pub mod cleanup;
mod fabric_api;
mod loader_helpers;
mod post_install;
mod setup_persist;
mod stages;
pub mod version_naming;

use crate::minecraft::download::{self, types as download_types};
use crate::minecraft::sources::DownloadSourceMode;
use crate::state::{AppState, StageStatus};
use crate::{log_error, log_info, log_warn};
use std::sync::Arc;
use tauri::{Emitter, State};

use super::{sanitize_mc_version, sanitize_version_id};
use cleanup::cleanup_failed_install;
use fabric_api::auto_install_fabric_api;
use post_install::merge_and_rename_version;
use setup_persist::save_setup_and_create_isolation;
use stages::install_all_loaders;
use version_naming::resolve_unique_instance_name;

use super::list::detect_version_type_from_dir;

/// Merged install (MC + loader)
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn install_merged(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    mc_version: String,
    forge_version: Option<String>,
    neoforge_version: Option<String>,
    fabric_version: Option<String>,
    optifine_version: Option<String>,
    liteloader_version: Option<String>,
    instance_name: Option<String>,
) -> Result<(), String> {
    sanitize_mc_version(&mc_version)?;
    if let Some(ref name) = instance_name {
        sanitize_version_id(name)?;
    }
    log_info!(
        "Merged install: mc={}, forge={:?}, neoforge={:?}, fabric={:?}, optifine={:?}",
        mc_version,
        forge_version,
        neoforge_version,
        fabric_version,
        optifine_version
    );

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

    // 重置取消/暂停信号（确保每次安装都是干净状态）
    state
        .download_cancel_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);

    // 清空上一次下载的 stages（避免累积导致前端显示历史阶段）
    // 修复：之前 append_stages 会保留旧 stages，多次安装后 stages 数组越来越长
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.stages.clear();
        ds.current_stage_index = 0;
        ds.is_active = false;
        ds.is_complete = false;
        ds.global_speed = 0;
        ds.global_bytes_downloaded = 0;
        ds.global_bytes_total = 0;
        ds.error_code = 0;
        ds.version_name = instance_name.clone().unwrap_or_else(|| mc_version.clone());
    }

    let base_name = instance_name.unwrap_or_else(|| mc_version.clone());
    let instance = resolve_unique_instance_name(&game_dir, &base_name);
    if instance != base_name {
        log_info!(
            "[Merged] Version name '{}' already exists, using '{}'",
            base_name,
            instance
        );
    }

    let has_any_loader = forge_version.is_some()
        || neoforge_version.is_some()
        || fabric_version.is_some()
        || optifine_version.is_some()
        || liteloader_version.is_some();

    // 设置下载阶段（append_stages 追加，保留整合包 stages）
    let stage_offset;
    let mut new_stages = vec![
        crate::state::DownloadStage::new_grouped("版本清单", 2.0, "MC本体安装"),
        crate::state::DownloadStage::new_grouped("版本信息", 3.0, "MC本体安装"),
        crate::state::DownloadStage::new_grouped("客户端", 5.0, "MC本体安装"),
        crate::state::DownloadStage::new_grouped("库文件", 15.0, "MC本体安装"),
        crate::state::DownloadStage::new_grouped("资源文件", 20.0, "MC本体安装"),
    ];
    if has_any_loader {
        new_stages.push(crate::state::DownloadStage::new_grouped(
            "加载器安装",
            30.0,
            "MC本体安装",
        ));
    }
    {
        let mut ds = state.download_state.lock().unwrap();
        stage_offset = ds.append_stages(new_stages);
    }

    // 读取下载相关配置
    let (mirror_url, loader_source_mode) = crate::state::resolve_mirror_and_source(&state).await;
    let (max_threads, chunk_count, speed_limit, download_source_mode) = {
        let config = state.config.lock().await;
        (
            config.download.max_threads as usize,
            config.download.chunk_count as usize,
            config.download.max_speed,
            DownloadSourceMode::from_str(&config.download.source),
        )
    };

    // progress callback：统一用 sync_stage_from_progress 同步 GlobalProgress 到 download_state
    let state_clone = state.download_state.clone();
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
    });

    // Stage callback：统一用 set_current_stage 切换阶段
    let state_for_stage = state.download_state.clone();
    let stage_callback = Arc::new(move |stage_index: usize, _stage_name: &str| {
        let actual_index = stage_offset + stage_index;
        let mut ds = state_for_stage.lock().unwrap();
        ds.set_current_stage(actual_index);
    });

    // Step 1: 下载原版 MC
    log_info!("[Merged] Downloading base MC version: {}", mc_version);
    let result = download::download_version_full(
        &mc_version,
        game_dir.as_path(),
        mirror_url.as_deref(),
        max_threads,
        chunk_count,
        speed_limit,
        download_source_mode,
        Some(progress_callback),
        Some(stage_callback),
        Some(state.download_cancel_flag.clone()),
        Some(state.download_pause_flag.clone()),
    )
    .await
    .map_err(|e| {
        log_error!("Failed to download MC version: {}", e);
        // MC 本体下载失败：清理已下载的部分文件
        cleanup_failed_install(&game_dir, &mc_version, fabric_version.as_deref());
        // 重置 download_state，避免 is_active 仍为 true 导致前端下载管理页卡住
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(0);
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

    // Step 2: 安装加载器
    let cancelled = state
        .download_cancel_flag
        .load(std::sync::atomic::Ordering::Relaxed);

    let loader_errors = if has_any_loader && !cancelled {
        install_all_loaders(
            &state,
            &mc_version,
            &game_dir,
            &forge_version,
            &neoforge_version,
            &fabric_version,
            &optifine_version,
            &liteloader_version,
            mirror_url.as_deref(),
            max_threads,
            loader_source_mode,
        )
        .await
    } else {
        Vec::new()
    };

    if cancelled {
        log_warn!("[Merged] 用户取消安装，跳过加载器安装");
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(1);
        return Err("下载已取消".to_string());
    }

    if !loader_errors.is_empty() {
        let msg = format!("部分加载器安装失败: {}", loader_errors.join("; "));
        log_error!("[Merged] {}", msg);
        // 加载器安装失败：清理已下载的 MC 本体和加载器目录
        cleanup_failed_install(&game_dir, &mc_version, fabric_version.as_deref());
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(1);
        let _ = app.emit(
            "install-merged-progress",
            serde_json::json!({ "stage": "failed", "message": msg }),
        );
        return Err(msg);
    }

    // Step 3: 安装后处理（JSON 合并 + 重命名）
    let actual_version_id =
        merge_and_rename_version(&game_dir, &mc_version, &instance, has_any_loader);

    let version_type = detect_version_type_from_dir(&game_dir, &actual_version_id);

    // Step 4: 保存 setup.ini + 创建隔离目录
    save_setup_and_create_isolation(
        &state,
        &game_dir,
        &actual_version_id,
        &mc_version,
        version_type,
    )
    .await;

    // Step 5: 自动安装 Fabric API（仅 Fabric 用户）
    if fabric_version.is_some() {
        auto_install_fabric_api(
            &state,
            &game_dir,
            &mc_version,
            &actual_version_id,
            version_type,
        )
        .await;
    }

    // 完成
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_complete();
    }
    let _ = app.emit(
        "install-merged-progress",
        serde_json::json!({ "stage": "completed", "version_id": actual_version_id }),
    );
    log_info!("[Merged] install_merged 完成: {}", actual_version_id);
    Ok(())
}
