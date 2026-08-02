//! 加载器版本查询与安装命令
//!
//! 注：原 8 个独立 Tauri 命令已聚合为 `version_install_manager` IPC 入口，
//! 通过请求体的 `action` 字段分发。本模块函数已去掉 `#[tauri::command]` 标注，
//! 由 `install_manager::dispatch` 反序列化参数后调用。

use crate::error_util::log_err;
use crate::log_error;
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::loaders;
use crate::state::AppState;

/// List Forge versions
pub async fn list_forge_versions(
    state: &AppState,
    mc_version: String,
) -> Result<Vec<serde_json::Value>, String> {
    let (mirror_url, source_mode) = crate::state::resolve_mirror_and_source(state).await;

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
    Ok(result)
}

/// List NeoForge versions
pub async fn list_neoforge_versions(
    state: &AppState,
    mc_version: String,
) -> Result<Vec<serde_json::Value>, String> {
    let (mirror_url, source_mode) = crate::state::resolve_mirror_and_source(state).await;

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

    Ok(result)
}

/// List Fabric versions
pub async fn list_fabric_versions(state: &AppState) -> Result<serde_json::Value, String> {
    let (mirror_url, source_mode) = crate::state::resolve_mirror_and_source(state).await;

    let versions = loaders::list_fabric_versions(mirror_url.as_deref(), source_mode)
        .await
        .map_err(|e| {
            log_error!("Failed to list Fabric versions: {}", e);
            e.to_string()
        })?;

    serde_json::to_value(&versions).map_err(log_err("Failed to serialize Fabric versions"))
}

/// List OptiFine versions
pub async fn list_optifine_versions(state: &AppState) -> Result<Vec<serde_json::Value>, String> {
    let (mirror_url, source_mode) = crate::state::resolve_mirror_and_source(state).await;

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
    Ok(result)
}

/// List LiteLoader versions
pub async fn list_liteloader_versions(
    state: &AppState,
    mc_version: String,
) -> Result<Vec<String>, String> {
    let (mirror_url, source_mode) = crate::state::resolve_mirror_and_source(state).await;

    let versions =
        loaders::list_liteloader_versions(&mc_version, mirror_url.as_deref(), source_mode)
            .await
            .map_err(|e| {
                log_error!("Failed to list LiteLoader versions: {}", e);
                e.to_string()
            })?;

    let version_strings: Vec<String> = versions.iter().map(|v| v.version.clone()).collect();
    Ok(version_strings)
}

/// Validate loaders compatibility
pub async fn validate_loaders(
    _mc_version: String,
    _forge_version: Option<String>,
    _neoforge_version: Option<String>,
    _fabric_version: Option<String>,
    _optifine_version: Option<String>,
) -> Result<bool, String> {
    Ok(true)
}

/// List Fabric API versions compatible with the given MC version
///
/// 从 Modrinth 查询 fabric-api 版本列表并按 MC 版本筛选
///
/// 直接返回 Vec<FabricApiVersion>，由 dispatcher 序列化为 JSON 数组。
pub async fn list_fabric_api_versions(
    mc_version: String,
) -> Result<Vec<loaders::fabric_api::FabricApiVersion>, String> {
    let versions = loaders::fabric_api::list_versions(&mc_version)
        .await
        .map_err(|e| {
            crate::log_error!("Failed to list Fabric API versions: {}", e);
            e
        })?;

    Ok(versions)
}

/// Install Fabric API for a specific version
///
/// 下载到版本对应的 mods 目录（考虑版本隔离）
pub async fn install_fabric_api_for_version(
    state: &AppState,
    version_id: String,
    download_url: String,
    file_name: String,
    hash: Option<String>,
) -> Result<(), String> {
    use crate::commands::version::mods::helpers::get_mods_dir;

    let config = DownloadManagerConfig::from_state_for_meta(state).await;

    let mods_dir: std::path::PathBuf = get_mods_dir(state, &version_id).await?;

    crate::log_info!(
        "[FabricAPI] 为版本 {} 安装 Fabric API: {}",
        version_id,
        file_name
    );

    loaders::fabric_api::install(
        &download_url,
        &file_name,
        &mods_dir,
        hash.as_deref(),
        &config,
        None,
    )
    .await
    .map_err(|e| {
        crate::log_error!("Failed to install Fabric API: {}", e);
        e.to_string()
    })?;

    Ok(())
}
