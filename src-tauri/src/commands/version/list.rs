//! 版本列表、类型检测、隔离解析
//!
//! 注：原 6 个独立 Tauri 命令已聚合为 `version_list_manager` IPC 入口，
//! 通过请求体的 `action` 字段分发。本模块函数已去掉 `#[tauri::command]` 标注，
//! 由 `utils::version_list_manager::dispatch` 反序列化参数后调用。

use crate::minecraft::download;
use crate::minecraft::fools;
use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::version::scan as version_scan;
use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;
use crate::{log_debug, log_error, log_info};
use serde::{Deserialize, Serialize};

use super::sanitize_version_id;
use super::types::{VersionInfo, VersionListResult};

/// Get version list
pub async fn list_versions(state: &AppState) -> Result<VersionListResult, String> {
    log_info!("Fetching version list");

    let (mirror_url, source_mode) = crate::state::resolve_mirror_and_source(&state).await;

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
    // 使用统一的时间解析工具，支持 RFC3339 / naive datetime / 纯日期
    match crate::utils::datetime::parse_utc(time_str) {
        #[allow(deprecated)]
        Some(dt) => dt.timestamp(),
        None => 0,
    }
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
pub async fn list_installed_versions(state: &AppState) -> Result<Vec<String>, String> {
    log_info!("Fetching installed versions");

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

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

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

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
    match setup.display.indie_type.unwrap_or(0) {
        1 => 4,           // 强制隔离 → IsolationAll
        2 => 0,           // 强制不隔离 → IsolationNone
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
pub async fn uninstall_version(
    state: &AppState,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Uninstalling version: '{}'", version_id);

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

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
pub async fn get_version_effective_dir(
    state: &AppState,
    version_id: String,
) -> Result<String, String> {
    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;
    let global_isolation_mode = state.config.lock().await.isolation_mode;

    // 版本独立隔离设置覆盖全局
    let isolation_mode = resolve_isolation_mode(&game_dir, &version_id, global_isolation_mode);
    let version_type = detect_version_type_from_dir(&game_dir, &version_id);
    let mode = IsolationMode::from_u32(isolation_mode);
    let effective_dir =
        isolation::get_effective_game_dir(&game_dir, &version_id, mode, version_type);

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
pub async fn get_version_game_version(
    state: &AppState,
    version_id: String,
) -> Result<Option<String>, String> {
    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

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

/// 获取版本加载器信息（加载器类型 + 加载器版本号）
///
/// 读取 `versions/{id}/setup.ini` 的 `Type` 字段和对应的 `XxxVersion` 字段，
/// 用于创建联机房间时上报 `host_loader` / `host_loader_version`。
///
/// 返回 `(loader_type, loader_version)`：
/// - `loader_type`：`forge` / `fabric` / `neoforge` / `quilt` / `optifine` / `liteloader` / `release` / `snapshot` / `old` / `unknown`
/// - `loader_version`：对应加载器的版本号（如 `47.2.0`），无加载器时为空字符串
///
/// setup.ini 不存在时返回 `("release", "")`（兜底为原版）。
pub async fn get_version_loader_info(
    state: &AppState,
    version_id: String,
) -> Result<(String, String), String> {
    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;
    let version_dir = game_dir.join("versions").join(&version_id);

    // 优先读 setup.ini
    if let Some(setup) = VersionSetup::load(&version_dir)
        .map_err(|e| format!("Failed to read setup.ini: {}", e))?
    {
        let loader_type = version_type_to_string(&setup.loader.version_type);
        let loader_version = match setup.loader.version_type {
            VersionType::Forge => setup.loader.forge_version.clone().unwrap_or_default(),
            VersionType::NeoForge => setup.loader.neoforge_version.clone().unwrap_or_default(),
            VersionType::Fabric => setup.loader.fabric_version.clone().unwrap_or_default(),
            VersionType::Quilt => setup.loader.quilt_version.clone().unwrap_or_default(),
            VersionType::OptiFine => setup.loader.optifine_version.clone().unwrap_or_default(),
            VersionType::LiteLoader => setup.loader.liteloader_version.clone().unwrap_or_default(),
            _ => String::new(),
        };
        return Ok((loader_type, loader_version));
    }

    // setup.ini 不存在：从版本 JSON 检测类型，但无法获取加载器版本号
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let detected = VersionType::detect_from_json(&version_id, &json);
                return Ok((version_type_to_string(&detected), String::new()));
            }
        }
    }

    Ok(("release".to_string(), String::new()))
}

/// 读取本地整合包元数据（联机大厅阶段 3 新增）
///
/// 从 `versions/{id}/modpack.meta.json` 读取整合包来源元数据，
/// 用于创建联机房间时上报 `modpack` 字段。
///
/// 返回 `Option<ModpackMetaFile>`：
/// - `Some`：文件存在且解析成功，含 source/project_id/file_id/name/... 等字段
/// - `None`：文件不存在（非平台安装的版本，如手动导入或原版）
///
/// 文件存在但解析失败时返回错误（提示用户 modpack.meta.json 可能损坏）。
pub async fn read_local_modpack_meta(
    state: &AppState,
    version_id: String,
) -> Result<Option<crate::minecraft::version::modpack_meta::ModpackMetaFile>, String> {
    use crate::minecraft::version::modpack_meta::ModpackMetaFile;

    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;
    let version_dir = game_dir.join("versions").join(&version_id);

    ModpackMetaFile::load(&version_dir)
        .map_err(|e| format!("Failed to read modpack.meta.json: {}", e))
}

/// 校验本地是否已安装指定整合包的检测结果（联机大厅阶段 4 新增）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckLocalModpackResult {
    /// 是否已安装
    pub installed: bool,
    /// 匹配的 version_id（`installed=false` 时为 None）
    pub version_id: Option<String>,
}

/// 校验本地是否已安装指定整合包（联机大厅阶段 4 新增）
///
/// 扫描所有已安装版本的 `modpack.meta.json`，按以下优先级匹配：
/// 1. `manifest_hash` 优先匹配（双方都有且一致）
/// 2. 回退三元组匹配：`(source, project_id, file_id)`
///
/// 用于加入方加入房间后判断本地是否已装房主要求的整合包。
pub async fn check_local_modpack(
    state: &AppState,
    manifest_hash: Option<String>,
    source: String,
    project_id: String,
    file_id: String,
) -> Result<CheckLocalModpackResult, String> {
    use crate::minecraft::version::modpack_meta::ModpackMetaFile;

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;
    let versions = version_scan::scan_installed_versions(&game_dir);

    for version in &versions {
        let version_dir = game_dir.join("versions").join(&version.id);
        if let Ok(Some(meta)) = ModpackMetaFile::load(&version_dir) {
            // 优先 manifest_hash 匹配
            if let (Some(req_hash), Some(local_hash)) = (&manifest_hash, &meta.manifest_hash) {
                if req_hash == local_hash {
                    return Ok(CheckLocalModpackResult {
                        installed: true,
                        version_id: Some(version.id.clone()),
                    });
                }
            }
            // 回退三元组匹配
            if meta.source == source && meta.project_id == project_id && meta.file_id == file_id {
                return Ok(CheckLocalModpackResult {
                    installed: true,
                    version_id: Some(version.id.clone()),
                });
            }
        }
    }

    Ok(CheckLocalModpackResult {
        installed: false,
        version_id: None,
    })
}

