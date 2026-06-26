//! 版本管理命令

use crate::sdk::VersionList;
use crate::state::AppState;
use tauri::State;

/// 获取版本列表
#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionList, String> {
    log::info!("Fetching version list");

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    let versions = sdk.list_versions().map_err(|e| {
        log::error!("Failed to list versions: {}", e);
        e.to_string()
    })?;

    log::info!("Found {} versions", versions.versions.len());
    Ok(versions)
}

/// 下载版本
#[tauri::command]
pub async fn download_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    log::info!("Downloading version: {}", version_id);

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    sdk.download_version(&version_id).map_err(|e| {
        log::error!("Failed to download version: {}", e);
        e.to_string()
    })?;

    log::info!("Version {} downloaded successfully", version_id);
    Ok(())
}

/// 获取已安装版本列表
#[tauri::command]
pub async fn list_installed_versions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    log::info!("Fetching installed versions");

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    let versions = sdk.list_installed_versions().map_err(|e| {
        log::error!("Failed to list installed versions: {}", e);
        e.to_string()
    })?;

    log::info!("Found {} installed versions", versions.len());
    Ok(versions)
}
