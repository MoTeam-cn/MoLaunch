//! Pack 管理命令（toggle_pack / delete_pack）
//! 启停/删除同步写 options.txt（资源包 resourcePacks 数组；光影 shaderPack 键）。

use crate::log_info;
use crate::state::AppState;
use crate::utils::path::sanitize_file_name;

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::resolve_packs_dir;
use super::types::PackKind;
use crate::minecraft::resourcepack_options;

/// 启用/禁用 Pack（重命名 .disabled 并同步 options.txt），返回新文件名
pub async fn toggle_pack(
    state: &AppState,
    version_id: String,
    file_name: String,
    enable: bool,
    kind: PackKind,
) -> Result<String, String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    log_info!(
        "Toggling pack {} for version {} (enable={})",
        file_name,
        version_id,
        enable
    );

    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    let new_name = pack_common::toggle_entry(&dir, &file_name, enable)?;
    let enabled_name = pack_common::enabled_name_of(&new_name);
    let is_folder = dir.join(&enabled_name).is_dir();
    sync_options(state, &version_id, &enabled_name, is_folder, enable, kind).await;
    Ok(new_name)
}

/// 删除 Pack 文件/目录并同步 options.txt
pub async fn delete_pack(
    state: &AppState,
    version_id: String,
    file_name: String,
    kind: PackKind,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    log_info!("Deleting pack {} for version {}", file_name, version_id);

    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    let is_folder = dir.join(&file_name).is_dir();
    let enabled_name = pack_common::enabled_name_of(&file_name);
    pack_common::delete_entry(&dir, &file_name)?;
    sync_options(state, &version_id, &enabled_name, is_folder, false, kind).await;
    Ok(())
}

/// 同步 options.txt：资源包写 resourcePacks 数组，光影写 shaderPack 键
async fn sync_options(
    state: &AppState,
    version_id: &str,
    enabled_name: &str,
    is_folder: bool,
    enabled: bool,
    kind: PackKind,
) {
    let game_dir = match pack_common::resolve_effective_game_dir(state, version_id).await {
        Ok(g) => g,
        Err(_) => return,
    };
    match kind {
        PackKind::Resourcepack => {
            let mc_version = crate::commands::version::list::get_version_game_version(
                state,
                version_id.to_string(),
            )
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
            let _ = resourcepack_options::set_resource_pack_enabled(
                &game_dir,
                enabled_name,
                is_folder,
                enabled,
                &mc_version,
            );
        }
        PackKind::Shader => {
            let value = if enabled { Some(enabled_name) } else { None };
            let _ = resourcepack_options::set_shader_pack(&game_dir, value);
        }
    }
}
