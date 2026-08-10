//! Packs 列表查询命令（is_packs_available / list_packs）

use std::path::Path;

use crate::log_info;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::resolve_packs_dir;
use super::types::{PackInfo, PackKind};

/// 判断版本是否可安装资源包/光影
///
/// - 资源包：原版亦可安装，版本目录存在即 true
/// - 光影：需 OptiFine（版本 JSON / ID 检测）或 Iris（mods 目录含 iris*.jar）
pub async fn is_packs_available(
    state: &AppState,
    version_id: String,
    kind: PackKind,
) -> Result<bool, String> {
    sanitize_version_id(&version_id)?;
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let version_dir = game_dir.join("versions").join(&version_id);
    if !version_dir.exists() {
        return Ok(false);
    }
    match kind {
        PackKind::Resourcepack => Ok(true),
        PackKind::Shader => Ok(is_shader_supported(state, &version_dir, &version_id).await),
    }
}

/// 版本是否支持光影：OptiFine 或 Iris
async fn is_shader_supported(state: &AppState, version_dir: &Path, version_id: &str) -> bool {
    // 1. 版本 JSON 检测 OptiFine
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if VersionType::detect_from_json(version_id, &json) == VersionType::OptiFine {
                    return true;
                }
            }
        }
    }

    // 2. 版本 ID 推断
    let id_lower = version_id.to_lowercase();
    if id_lower.contains("optifine") || id_lower.contains("iris") {
        return true;
    }

    // 3. mods 目录含 Iris mod（Iris 以 mod 形式随 Fabric/Forge 安装）
    if let Ok(mods_dir) = pack_common::resolve_version_subdir(state, version_id, "mods").await {
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            return entries.flatten().any(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.starts_with("iris") && name.ends_with(".jar")
            });
        }
    }

    false
}

/// 列出版本的资源包/光影（zip + 文件夹 + .disabled 变体）
pub async fn list_packs(
    state: &AppState,
    version_id: String,
    kind: PackKind,
) -> Result<Vec<PackInfo>, String> {
    sanitize_version_id(&version_id)?;
    log_info!("Listing packs for version: {}", version_id);

    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    let entries = pack_common::list_entries(&dir, kind.suffixes(), true)?;

    let packs: Vec<PackInfo> = entries
        .into_iter()
        .map(|e| PackInfo {
            file_name: e.file_name,
            enabled_name: e.enabled_name,
            is_enabled: e.is_enabled,
            is_folder: e.is_dir,
            size: e.size,
        })
        .collect();

    log_info!("Found {} packs for version {}", packs.len(), version_id);
    Ok(packs)
}
