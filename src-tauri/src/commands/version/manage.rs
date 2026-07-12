use crate::minecraft::version::scan as version_scan;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;
use crate::{log_error, log_info};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::sanitize_version_id;

/// Installed version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersionInfo {
    pub id: String,
    pub version_type: String,
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

    log_info!(
        "Found {} version directories: {:?}",
        version_ids.len(),
        version_ids
    );
    Ok(version_ids)
}

/// Get installed versions with type info
#[tauri::command]
pub async fn list_installed_versions_with_type(
    state: State<'_, AppState>,
) -> Result<Vec<InstalledVersionInfo>, String> {
    log_info!("Fetching installed versions with type info");

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let versions = version_scan::scan_installed_versions(&game_dir);
    let mut result = Vec::new();

    for version in versions {
        let version_type = detect_version_type_from_dir(&game_dir, &version.id);
        result.push(InstalledVersionInfo {
            id: version.id,
            version_type: version_type_to_string(&version_type),
        });
    }

    log_info!("Found {} versions with type info", result.len());
    Ok(result)
}

/// Detect version type from directory
fn detect_version_type_from_dir(game_dir: &std::path::Path, version_id: &str) -> VersionType {
    let version_dir = game_dir.join("versions").join(version_id);

    // 1. 优先从 JSON 检测（检查libraries中的加载器）
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let detected = VersionType::detect_from_json(version_id, &json);
                // 如果检测到加载器类型，直接返回
                if detected != VersionType::Release {
                    return detected;
                }
            }
        }
    }

    // 2. 从 setup.ini 读取（仅当JSON检测为Release时）
    let setup_path = version_dir.join("setup.ini");
    if setup_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&setup_path) {
            for line in content.lines() {
                if let Some(value) = line.strip_prefix("Type=") {
                    let type_str = value.trim().to_lowercase();
                    // 忽略 "release"，继续检测
                    if type_str != "release" {
                        return match type_str.as_str() {
                            "forge" => VersionType::Forge,
                            "neoforge" => VersionType::NeoForge,
                            "fabric" => VersionType::Fabric,
                            "quilt" => VersionType::Quilt,
                            "optifine" => VersionType::OptiFine,
                            "liteloader" => VersionType::LiteLoader,
                            "snapshot" => VersionType::Snapshot,
                            "old" | "old_alpha" | "old_beta" => VersionType::Old,
                            _ => VersionType::Release,
                        };
                    }
                }
            }
        }
    }

    // 3. 从版本ID推断
    let id_lower = version_id.to_lowercase();
    if id_lower.contains("forge") {
        return VersionType::Forge;
    }
    if id_lower.contains("neoforge") {
        return VersionType::NeoForge;
    }
    if id_lower.contains("fabric") {
        return VersionType::Fabric;
    }
    if id_lower.contains("optifine") {
        return VersionType::OptiFine;
    }

    VersionType::Release
}

/// Convert VersionType to string
fn version_type_to_string(version_type: &VersionType) -> String {
    match version_type {
        VersionType::Release => "release".to_string(),
        VersionType::Snapshot => "snapshot".to_string(),
        VersionType::Old => "old".to_string(),
        VersionType::Fool => "fool".to_string(),
        VersionType::Forge => "forge".to_string(),
        VersionType::NeoForge => "neoforge".to_string(),
        VersionType::Fabric => "fabric".to_string(),
        VersionType::Quilt => "quilt".to_string(),
        VersionType::OptiFine => "optifine".to_string(),
        VersionType::LiteLoader => "liteloader".to_string(),
        VersionType::Unknown => "unknown".to_string(),
    }
}

/// Uninstall version
#[tauri::command]
pub async fn uninstall_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
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
