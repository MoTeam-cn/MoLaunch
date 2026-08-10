//! Packs 列表查询命令（is_packs_available / list_packs）

use crate::log_info;
use crate::state::AppState;

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::resolve_packs_dir;
use super::types::{PackInfo, PackKind};

/// 判断版本是否可安装资源包/光影（版本目录存在即 true，原版亦可装）
pub async fn is_packs_available(
    state: &AppState,
    version_id: String,
    _kind: PackKind,
) -> Result<bool, String> {
    sanitize_version_id(&version_id)?;
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    Ok(game_dir.join("versions").join(&version_id).exists())
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
