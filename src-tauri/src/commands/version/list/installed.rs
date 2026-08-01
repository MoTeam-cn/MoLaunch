//! 本地版本扫描与卸载（list_installed_versions / with_type / uninstall_version）

use crate::minecraft::version::scan as version_scan;
use crate::minecraft::version::setup::VersionSetup;
use crate::state::AppState;
use crate::{log_debug, log_error, log_info};
use serde::{Deserialize, Serialize};

use super::super::sanitize_version_id;
use super::{detect_version_type_from_dir, version_type_to_string};

/// Installed version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersionInfo {
    pub id: String,
    pub version_type: String,
    /// 自定义图标文件名（空=自动判断，来自 setup.ini 的 Logo 字段）
    pub logo: String,
}

/// Get installed versions
pub async fn list_installed_versions(state: &AppState) -> Result<Vec<String>, String> {
    log_info!("Fetching installed versions");

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;

    let versions = version_scan::scan_installed_versions(&game_dir);
    let version_ids: Vec<String> = versions.iter().map(|v| v.id.clone()).collect();

    log_info!(
        "Found {} version directories: {:?}",
        version_ids.len(),
        version_ids
    );
    Ok(version_ids)
}

/// Get installed versions with type info
pub async fn list_installed_versions_with_type(
    state: &AppState,
) -> Result<Vec<InstalledVersionInfo>, String> {
    log_debug!("Fetching installed versions with type info");

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;

    let versions = version_scan::scan_installed_versions(&game_dir);
    let mut result = Vec::new();

    for version in versions {
        let version_type = detect_version_type_from_dir(&game_dir, &version.id);
        log_debug!(
            "[VersionList] detect type: id={} type={:?}",
            version.id,
            version_type
        );
        // 读取版本独立 setup.ini 的 Logo 字段（空=自动判断）
        let version_dir = game_dir.join("versions").join(&version.id);
        let logo = VersionSetup::load_or_create(&version_dir, &version.id)
            .display
            .logo
            .unwrap_or_default();
        result.push(InstalledVersionInfo {
            id: version.id,
            version_type: version_type_to_string(&version_type),
            logo,
        });
    }

    log_debug!("Found {} versions with type info", result.len());
    Ok(result)
}

/// Uninstall version
pub async fn uninstall_version(state: &AppState, version_id: String) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Uninstalling version: '{}'", version_id);

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;

    version_scan::uninstall_version(&game_dir, &version_id).map_err(|e| {
        log_error!("Failed to uninstall version: {}", e);
        e.to_string()
    })?;

    log_info!("Version {} uninstalled successfully", version_id);
    Ok(())
}
