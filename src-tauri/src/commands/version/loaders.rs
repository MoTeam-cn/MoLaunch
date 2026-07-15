use crate::log_error;
use crate::minecraft::loaders;
use crate::minecraft::sources::DownloadSourceMode;
use crate::state::AppState;
use tauri::State;

/// List Forge versions
#[tauri::command]
pub async fn list_forge_versions(
    state: State<'_, AppState>,
    mc_version: String,
) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    let source_mode = DownloadSourceMode::from_str(&config.meta_source);
    drop(config);

    let versions = loaders::list_forge_versions(&mc_version, mirror_url.as_deref(), source_mode)
        .await
        .map_err(|e| {
            log_error!("Failed to list Forge versions: {}", e);
            e.to_string()
        })?;

    let result: Vec<serde_json::Value> = versions
        .iter()
        .map(|v| {
            serde_json::json!({
                "version": v.version,
                "is_recommended": v.is_recommended,
                "release_time": v.release_time.as_deref().unwrap_or("")
            })
        })
        .collect();
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

/// List NeoForge versions
#[tauri::command]
pub async fn list_neoforge_versions(
    state: State<'_, AppState>,
    mc_version: String,
) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    let source_mode = DownloadSourceMode::from_str(&config.meta_source);
    drop(config);

    let versions = loaders::list_neoforge_versions(&mc_version, mirror_url.as_deref(), source_mode)
        .await
        .map_err(|e| {
            log_error!("Failed to list NeoForge versions: {}", e);
            e.to_string()
        })?;

    let result: Vec<serde_json::Value> = versions
        .iter()
        .map(|v| {
            serde_json::json!({
                "version": v.version,
                "recommended": v.is_recommended
            })
        })
        .collect();

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

/// List Fabric versions
#[tauri::command]
pub async fn list_fabric_versions(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    let source_mode = DownloadSourceMode::from_str(&config.meta_source);
    drop(config);

    let versions = loaders::list_fabric_versions(mirror_url.as_deref(), source_mode)
        .await
        .map_err(|e| {
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
    let source_mode = DownloadSourceMode::from_str(&config.meta_source);
    drop(config);

    let versions = loaders::list_optifine_versions(mirror_url.as_deref(), source_mode)
        .await
        .map_err(|e| {
            log_error!("Failed to list OptiFine versions: {}", e);
            e.to_string()
        })?;

    let result: Vec<serde_json::Value> = versions
        .iter()
        .map(|v| {
            serde_json::json!({
                "display_name": v.version,
                "is_preview": !v.is_recommended
            })
        })
        .collect();
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

/// List LiteLoader versions
#[tauri::command]
pub async fn list_liteloader_versions(
    state: State<'_, AppState>,
    mc_version: String,
) -> Result<String, String> {
    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    let source_mode = DownloadSourceMode::from_str(&config.meta_source);
    drop(config);

    let versions =
        loaders::list_liteloader_versions(&mc_version, mirror_url.as_deref(), source_mode)
            .await
            .map_err(|e| {
                log_error!("Failed to list LiteLoader versions: {}", e);
                e.to_string()
            })?;

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
