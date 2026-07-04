//! Version management commands

use crate::{log_info, log_error};
use crate::minecraft::download::{self, manager as download_manager};
use crate::minecraft::loaders;
use crate::minecraft::version::scan as version_scan;
use crate::state::{AppState, DownloadState, StageStatus};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tauri::{Emitter, State};

/// Version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    pub version_type: String,
    pub release_time: i64,  // Unix时间戳
    pub url: String,
}

/// Version list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionListResult {
    pub versions: Vec<VersionInfo>,
    pub latest_release: String,
    pub latest_snapshot: String,
    pub source_name: String,
}

/// Download progress snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStageSnapshot {
    pub name: String,
    pub progress: f64,
    pub weight: f64,
    pub status: String,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub files_downloaded: usize,
    pub files_total: usize,
}

/// Download progress snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressSnapshot {
    pub stages: Vec<DownloadStageSnapshot>,
    pub current_stage_index: usize,
    pub global_speed: u64,
    pub global_bytes_downloaded: u64,
    pub global_bytes_total: u64,
    pub is_active: bool,
    pub is_complete: bool,
    pub error_code: i32,
}

/// Get version list
#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionListResult, String> {
    log_info!("Fetching version list");

    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let result = download::fetch_version_list(mirror_url.as_deref()).await.map_err(|e| {
        log_error!("Failed to list versions: {}", e);
        e.to_string()
    })?;

    let (latest_release, latest_snapshot) = download::get_latest_versions(&result.value);
    let entries = download::parse_version_list(&result.value);

    let versions: Vec<VersionInfo> = entries.iter().map(|e| {
        // 将时间字符串转换为Unix时间戳
        let release_time = parse_timestamp(&e.release_time);
        VersionInfo {
            id: e.id.clone(),
            version_type: e.version_type.clone(),
            release_time,
            url: e.url.clone(),
        }
    }).collect();

    log_info!("Found {} versions", versions.len());
    Ok(VersionListResult {
        versions,
        latest_release: latest_release.unwrap_or_default(),
        latest_snapshot: latest_snapshot.unwrap_or_default(),
        source_name: result.source_name,
    })
}

/// 解析时间字符串为Unix时间戳
fn parse_timestamp(time_str: &str) -> i64 {
    // 尝试解析ISO 8601格式
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time_str) {
        #[allow(deprecated)]
        return dt.timestamp();
    }
    // 尝试解析其他格式
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S") {
        #[allow(deprecated)]
        return dt.timestamp();
    }
    0
}

/// Download version
#[tauri::command]
pub async fn download_version(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    log_info!("Downloading version: {}", version_id);

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = true;
        ds.is_complete = false;
        ds.current_stage_index = 0;
        ds.global_speed = 0;
        ds.global_bytes_downloaded = 0;
        ds.global_bytes_total = 0;
        ds.error_code = 0;
        for stage in ds.stages.iter_mut() {
            stage.progress = 0.0;
            stage.status = StageStatus::Waiting;
            stage.bytes_downloaded = 0;
            stage.bytes_total = 0;
        }
    }

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let mirror_url = config.mirror_url.clone();
    let max_threads = config.max_download_threads as usize;
    let speed_limit = config.max_download_speed;
    let source_mode = download_manager::DownloadSourceMode::from_str(&config.download_source);
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
    let progress_callback = Arc::new(move |progress: download_manager::GlobalProgress| {
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
                    stage.progress = (progress.downloaded_bytes as f64 / progress.total_bytes as f64).min(1.0);
                }
                stage.status = StageStatus::Loading;
            }

            // 滑动窗口速度计算
            {
                let mut window = sw.lock().unwrap();
                window.push_back((ds.global_bytes_downloaded, Instant::now()));
                if window.len() > 10 { window.pop_front(); }
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
        speed_limit,
        source_mode,
        Some(progress_callback),
        Some(stage_callback),
    ).await.map_err(|e| {
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

fn build_snapshot(ds: &DownloadState, version_id: &str) -> serde_json::Value {
    let stages: Vec<DownloadStageSnapshot> = ds.stages.iter().map(|s| {
        DownloadStageSnapshot {
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
        }
    }).collect();

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

/// Get installed versions
#[tauri::command]
pub async fn list_installed_versions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    log_info!("Fetching installed versions");

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let versions = version_scan::scan_installed_versions(&game_dir);
    let version_ids: Vec<String> = versions.iter().map(|v| v.id.clone()).collect();

    log_info!("Found {} version directories: {:?}", version_ids.len(), version_ids);
    Ok(version_ids)
}

/// Uninstall version
#[tauri::command]
pub async fn uninstall_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    log_info!("Uninstalling version: '{}'", version_id);

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    version_scan::uninstall_version(&game_dir, &version_id).map_err(|e| {
        log_error!("Failed to uninstall version: {}", e);
        e.to_string()
    })?;

    log_info!("Version {} uninstalled successfully", version_id);
    Ok(())
}

/// Get download progress
#[tauri::command]
pub async fn get_download_progress(state: State<'_, AppState>) -> Result<DownloadProgressSnapshot, String> {
    let ds = state.download_state.lock().unwrap();
    Ok(DownloadProgressSnapshot {
        stages: ds.stages.iter().map(|s| DownloadStageSnapshot {
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
        }).collect(),
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

/// List Forge versions
#[tauri::command]
pub async fn list_forge_versions(state: State<'_, AppState>, mc_version: String) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let versions = loaders::list_forge_versions(&mc_version, mirror_url.as_deref()).await.map_err(|e| {
        log_error!("Failed to list Forge versions: {}", e);
        e.to_string()
    })?;

    // 前端期望 { version: string, is_recommended: boolean, release_time: string }[]
    let result: Vec<serde_json::Value> = versions.iter().map(|v| {
        serde_json::json!({
            "version": v.version,
            "is_recommended": v.is_recommended,
            "release_time": v.release_time.as_deref().unwrap_or("")
        })
    }).collect();
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

/// List NeoForge versions
#[tauri::command]
pub async fn list_neoforge_versions(state: State<'_, AppState>, mc_version: String) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let versions = loaders::list_neoforge_versions(&mc_version, mirror_url.as_deref()).await.map_err(|e| {
        log_error!("Failed to list NeoForge versions: {}", e);
        e.to_string()
    })?;

    let result: Vec<serde_json::Value> = versions.iter().map(|v| {
        serde_json::json!({
            "version": v.version,
            "recommended": v.is_recommended
        })
    }).collect();
    
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

/// List Fabric versions
#[tauri::command]
pub async fn list_fabric_versions(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let versions = loaders::list_fabric_versions(mirror_url.as_deref()).await.map_err(|e| {
        log_error!("Failed to list Fabric versions: {}", e);
        e.to_string()
    })?;

    serde_json::to_string(&versions).map_err(|e| e.to_string())
}

/// List OptiFine versions
#[tauri::command]
pub async fn list_optifine_versions(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let versions = loaders::list_optifine_versions(mirror_url.as_deref()).await.map_err(|e| {
        log_error!("Failed to list OptiFine versions: {}", e);
        e.to_string()
    })?;

    // 前端期望 { display_name: string; is_preview: boolean }[]
    let result: Vec<serde_json::Value> = versions.iter().map(|v| {
        serde_json::json!({
            "display_name": v.version,
            "is_preview": !v.is_recommended
        })
    }).collect();
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

/// List LiteLoader versions
#[tauri::command]
pub async fn list_liteloader_versions(state: State<'_, AppState>, mc_version: String) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let versions = loaders::list_liteloader_versions(&mc_version, mirror_url.as_deref()).await.map_err(|e| {
        log_error!("Failed to list LiteLoader versions: {}", e);
        e.to_string()
    })?;

    // 前端期望 string[]，只返回版本号
    let version_strings: Vec<String> = versions.iter().map(|v| v.version.clone()).collect();
    serde_json::to_string(&version_strings).map_err(|e| e.to_string())
}

/// Validate loaders compatibility
#[tauri::command]
pub async fn validate_loaders(
    _mc_version: String,
    _forge_version: Option<String>,
    _neoforge_version: Option<String>,
    _fabric_version: Option<String>,
    _optifine_version: Option<String>,
) -> Result<bool, String> {
    Ok(true)
}

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
    log_info!("Merged install: mc={}, forge={:?}, neoforge={:?}, fabric={:?}, optifine={:?}",
        mc_version, forge_version, neoforge_version, fabric_version, optifine_version);

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = true;
        ds.is_complete = false;
        ds.current_stage_index = 0;
        for stage in ds.stages.iter_mut() {
            stage.progress = 0.0;
            stage.status = StageStatus::Waiting;
        }
    }

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let mirror_url = config.mirror_url.clone();
    let max_threads = config.max_download_threads as usize;
    drop(config);

    // Download base MC first
    log_info!("Downloading base MC version: {}", mc_version);
    download_version(app.clone(), state.clone(), mc_version.clone()).await?;

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = true;
        ds.is_complete = false;
        ds.current_stage_index = 0;
        for stage in ds.stages.iter_mut() {
            stage.progress = 0.0;
            stage.status = StageStatus::Waiting;
        }
    }

    // Install loaders
    if let Some(forge_ver) = forge_version {
        log_info!("Installing Forge {}", forge_ver);
        loaders::install_loader(
            loaders::LoaderType::Forge,
            &mc_version,
            &forge_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log_error!("Failed to install Forge: {}", e);
            e.to_string()
        })?;
    }

    if let Some(neoforge_ver) = neoforge_version {
        log_info!("Installing NeoForge {}", neoforge_ver);
        loaders::install_loader(
            loaders::LoaderType::NeoForge,
            &mc_version,
            &neoforge_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log_error!("Failed to install NeoForge: {}", e);
            e.to_string()
        })?;
    }

    if let Some(fabric_ver) = fabric_version {
        log_info!("Installing Fabric {}", fabric_ver);
        loaders::install_loader(
            loaders::LoaderType::Fabric,
            &mc_version,
            &fabric_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log_error!("Failed to install Fabric: {}", e);
            e.to_string()
        })?;
    }

    if let Some(optifine_ver) = optifine_version {
        log_info!("Installing OptiFine {}", optifine_ver);
        loaders::install_loader(
            loaders::LoaderType::OptiFine,
            &mc_version,
            &optifine_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log_error!("Failed to install OptiFine: {}", e);
            e.to_string()
        })?;
    }

    if let Some(liteloader_ver) = liteloader_version {
        log_info!("Installing LiteLoader {}", liteloader_ver);
        loaders::install_loader(
            loaders::LoaderType::LiteLoader,
            &mc_version,
            &liteloader_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log_error!("Failed to install LiteLoader: {}", e);
            e.to_string()
        })?;
    }

    let instance = instance_name.unwrap_or_else(|| mc_version.clone());
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = false;
        ds.is_complete = true;
        for stage in ds.stages.iter_mut() {
            stage.status = StageStatus::Finished;
            stage.progress = 1.0;
        }
    }
    let _ = app.emit("install-complete", serde_json::json!({ "instance_name": instance }));
    log_info!("Merged install completed");
    Ok(())
}
