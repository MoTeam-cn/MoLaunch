//! 版本列表、类型检测、隔离解析

use crate::minecraft::download;
use crate::minecraft::fools;
use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::sources::DownloadSourceMode;
use crate::minecraft::version::scan as version_scan;
use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;
use crate::{log_error, log_info};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::sanitize_version_id;
use super::types::{VersionInfo, VersionListResult};

/// Get version list
#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionListResult, String> {
    log_info!("Fetching version list");

    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    let source_mode = DownloadSourceMode::from_str(&config.meta_source);
    drop(config);

    let result = download::fetch_version_list(mirror_url.as_deref(), source_mode)
        .await
        .map_err(|e| {
            log_error!("Failed to list versions: {}", e);
            e.to_string()
        })?;

    let (latest_release, latest_snapshot) = download::get_latest_versions(&result.value);
    let entries = download::parse_version_list(&result.value);

    let versions: Vec<VersionInfo> = entries
        .iter()
        .map(|e| {
            let release_time = parse_timestamp(&e.release_time);
            let description = if e.version_type == "fool" {
                fools::get_fool_description(&e.id)
            } else {
                None
            };
            VersionInfo {
                id: e.id.clone(),
                version_type: e.version_type.clone(),
                release_time,
                url: e.url.clone(),
                description,
            }
        })
        .collect();

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

/// Installed version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersionInfo {
    pub id: String,
    pub version_type: String,
    /// 自定义图标文件名（空=自动判断，来自 setup.ini 的 Logo 字段）
    pub logo: String,
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
        log_info!(
            "[VersionList] detect type: id={} type={:?}",
            version.id,
            version_type
        );
        // 读取版本独立 setup.ini 的 Logo 字段（空=自动判断）
        let version_dir = game_dir.join("versions").join(&version.id);
        let logo = VersionSetup::load_or_create(&version_dir, &version.id)
            .logo
            .unwrap_or_default();
        result.push(InstalledVersionInfo {
            id: version.id,
            version_type: version_type_to_string(&version_type),
            logo,
        });
    }

    log_info!("Found {} versions with type info", result.len());
    Ok(result)
}

/// 根据版本独立隔离设置和全局设置，解析最终使用的 isolation_mode
///
/// - indie_type=0 或 None：跟随全局（返回 global_mode）
/// - indie_type=1：强制开启隔离（返回 4=IsolationAll）
/// - indie_type=2：强制关闭隔离（返回 0=IsolationNone）
pub fn resolve_isolation_mode(
    game_dir: &std::path::Path,
    version_id: &str,
    global_mode: u32,
) -> u32 {
    let version_dir = game_dir.join("versions").join(version_id);
    let setup = VersionSetup::load_or_create(&version_dir, version_id);
    match setup.indie_type.unwrap_or(0) {
        1 => 4, // 强制隔离 → IsolationAll
        2 => 0, // 强制不隔离 → IsolationNone
        _ => global_mode, // 跟随全局
    }
}

/// Detect version type from directory
pub fn detect_version_type_from_dir(game_dir: &std::path::Path, version_id: &str) -> VersionType {
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
pub(super) fn version_type_to_string(version_type: &VersionType) -> String {
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

/// 获取版本的有效游戏目录（考虑版本隔离）
///
/// 隔离时返回 `{game_dir}/versions/{version_id}/`
/// 非隔离时返回 `{game_dir}/`
#[tauri::command]
pub async fn get_version_effective_dir(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<String, String> {
    sanitize_version_id(&version_id)?;

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let global_isolation_mode = config.isolation_mode;
    drop(config);

    // 版本独立隔离设置覆盖全局
    let isolation_mode = resolve_isolation_mode(&game_dir, &version_id, global_isolation_mode);
    let version_type = detect_version_type_from_dir(&game_dir, &version_id);
    let mode = IsolationMode::from_u32(isolation_mode);
    let effective_dir = isolation::get_effective_game_dir(
        &game_dir,
        &version_id,
        mode,
        version_type,
    );

    Ok(effective_dir.to_string_lossy().to_string())
}

/// 获取版本对应的 Minecraft 游戏版本号（如 "1.20.1"）
///
/// 用于从 ModTab 打开资源详情弹窗时，自动选中整合包对应的版本筛选 tag。
/// 解析顺序参考 `version::scan::extract_original_version`：
/// 1. JSON 的 `inheritsFrom` 字段
/// 2. arguments.game 中的 `--fml.mcVersion`
/// 3. downloads.client.url 正则匹配
/// 4. JSON 的 `jar` 字段
/// 5. JSON 的 `id` 字段正则匹配
#[tauri::command]
pub async fn get_version_game_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<Option<String>, String> {
    sanitize_version_id(&version_id)?;

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let version_dir = game_dir.join("versions").join(&version_id);
    let json_path = version_dir.join(format!("{}.json", version_id));
    if !json_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read version JSON: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse version JSON: {}", e))?;

    Ok(version_scan::extract_original_version(&json, &content))
}
