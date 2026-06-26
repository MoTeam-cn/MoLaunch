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
