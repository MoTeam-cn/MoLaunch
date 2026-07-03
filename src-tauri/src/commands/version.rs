//! Version management commands

use crate::minecraft::download::{self, manager as download_manager};
use crate::minecraft::loaders;
use crate::minecraft::version::scan as version_scan;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, State};

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
pub struct DownloadProgressSnapshot {
    pub stage: u32,
    pub current: usize,
    pub total: usize,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub speed: u64,
    pub files_remaining: usize,
    pub is_active: bool,
    pub is_complete: bool,
    pub error_code: i32,
}

/// Get version list
#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionListResult, String> {
    log::info!("Fetching version list");

    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let result = download::fetch_version_list(mirror_url.as_deref()).await.map_err(|e| {
        log::error!("Failed to list versions: {}", e);
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

    log::info!("Found {} versions", versions.len());
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
    log::info!("Downloading version: {}", version_id);

    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    let mirror_url = config.mirror_url.clone();
    let max_threads = config.max_download_threads as usize;
    let speed_limit = config.max_download_speed;
    let source_mode = download_manager::DownloadSourceMode::from_str(&config.download_source);
    drop(config);

    let game_path = std::path::Path::new(&game_dir);
    let app_clone = app.clone();
    let version_id_clone = version_id.clone();

    // Create progress callback
    let progress_callback = Arc::new(move |progress: download_manager::GlobalProgress| {
        let _ = app_clone.emit_all("download-progress", serde_json::json!({
            "version_id": version_id_clone,
            "total_files": progress.total_files,
            "completed_files": progress.completed_files,
            "failed_files": progress.failed_files,
            "skipped_files": progress.skipped_files,
            "total_bytes": progress.total_bytes,
            "downloaded_bytes": progress.downloaded_bytes,
            "current_speed": progress.current_speed,
            "is_active": progress.is_active,
        }));
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
    ).await.map_err(|e| {
        log::error!("Failed to download version: {}", e);
        e.to_string()
    })?;

    log::info!(
        "Version {} download completed: libs {}/{}, assets {}/{}",
        version_id,
        result.libs_downloaded,
        result.libs_total,
        result.assets_downloaded,
        result.assets_total
    );

    let _ = app.emit_all(
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

/// Get installed versions
#[tauri::command]
pub async fn list_installed_versions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    log::info!("Fetching installed versions");

    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    drop(config);

    let game_path = std::path::Path::new(&game_dir);
    let versions = version_scan::scan_installed_versions(game_path);
    let version_ids: Vec<String> = versions.iter().map(|v| v.id.clone()).collect();

    log::info!("Found {} version directories: {:?}", version_ids.len(), version_ids);
    Ok(version_ids)
}

/// Uninstall version
#[tauri::command]
pub async fn uninstall_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    log::info!("Uninstalling version: '{}'", version_id);

    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    drop(config);

    let game_path = std::path::Path::new(&game_dir);
    version_scan::uninstall_version(game_path, &version_id).map_err(|e| {
        log::error!("Failed to uninstall version: {}", e);
        e.to_string()
    })?;

    log::info!("Version {} uninstalled successfully", version_id);
    Ok(())
}

/// Get download progress
#[tauri::command]
pub async fn get_download_progress() -> Result<DownloadProgressSnapshot, String> {
    Ok(DownloadProgressSnapshot {
        stage: 0,
        current: 0,
        total: 0,
        bytes_downloaded: 0,
        bytes_total: 0,
        speed: 0,
        files_remaining: 0,
        is_active: false,
        is_complete: true,
        error_code: 0,
    })
}

/// Check if downloading
#[tauri::command]
pub async fn is_downloading() -> Result<bool, String> {
    Ok(false)
}

/// Reset download progress
#[tauri::command]
pub async fn reset_download_progress() -> Result<(), String> {
    Ok(())
}

/// List Forge versions
#[tauri::command]
pub async fn list_forge_versions(state: State<'_, AppState>, mc_version: String) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let versions = loaders::list_forge_versions(&mc_version, mirror_url.as_deref()).await.map_err(|e| {
        log::error!("Failed to list Forge versions: {}", e);
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
        log::error!("Failed to list NeoForge versions: {}", e);
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
        log::error!("Failed to list Fabric versions: {}", e);
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
        log::error!("Failed to list OptiFine versions: {}", e);
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
        log::error!("Failed to list LiteLoader versions: {}", e);
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
    log::info!("Merged install: mc={}, forge={:?}, neoforge={:?}, fabric={:?}, optifine={:?}",
        mc_version, forge_version, neoforge_version, fabric_version, optifine_version);

    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    let mirror_url = config.mirror_url.clone();
    let max_threads = config.max_download_threads as usize;
    drop(config);

    let game_path = std::path::Path::new(&game_dir);

    // Download base MC first
    log::info!("Downloading base MC version: {}", mc_version);
    download_version(app.clone(), state.clone(), mc_version.clone()).await?;

    // Install loaders
    if let Some(forge_ver) = forge_version {
        log::info!("Installing Forge {}", forge_ver);
        loaders::install_loader(
            loaders::LoaderType::Forge,
            &mc_version,
            &forge_ver,
            game_path,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log::error!("Failed to install Forge: {}", e);
            e.to_string()
        })?;
    }

    if let Some(neoforge_ver) = neoforge_version {
        log::info!("Installing NeoForge {}", neoforge_ver);
        loaders::install_loader(
            loaders::LoaderType::NeoForge,
            &mc_version,
            &neoforge_ver,
            game_path,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log::error!("Failed to install NeoForge: {}", e);
            e.to_string()
        })?;
    }

    if let Some(fabric_ver) = fabric_version {
        log::info!("Installing Fabric {}", fabric_ver);
        loaders::install_loader(
            loaders::LoaderType::Fabric,
            &mc_version,
            &fabric_ver,
            game_path,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log::error!("Failed to install Fabric: {}", e);
            e.to_string()
        })?;
    }

    if let Some(optifine_ver) = optifine_version {
        log::info!("Installing OptiFine {}", optifine_ver);
        loaders::install_loader(
            loaders::LoaderType::OptiFine,
            &mc_version,
            &optifine_ver,
            game_path,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log::error!("Failed to install OptiFine: {}", e);
            e.to_string()
        })?;
    }

    if let Some(liteloader_ver) = liteloader_version {
        log::info!("Installing LiteLoader {}", liteloader_ver);
        loaders::install_loader(
            loaders::LoaderType::LiteLoader,
            &mc_version,
            &liteloader_ver,
            game_path,
            mirror_url.as_deref(),
            max_threads,
        ).await.map_err(|e| {
            log::error!("Failed to install LiteLoader: {}", e);
            e.to_string()
        })?;
    }

    let instance = instance_name.unwrap_or_else(|| mc_version.clone());
    let _ = app.emit_all("install-complete", serde_json::json!({ "instance_name": instance }));
    log::info!("Merged install completed");
    Ok(())
}
