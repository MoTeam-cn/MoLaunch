//! Pack 管理命令（toggle_pack / delete_pack）
//! 启停/删除同步写 options.txt（资源包 resourcePacks 数组；光影 shaderPack 键）。

use crate::state::AppState;
use crate::utils::path::sanitize_file_name;
use crate::{log_error, log_info};

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
    // 文件已改名但 options.txt 没写上时，回滚文件名：
    // 否则界面显示「已启用」、游戏里根本没加载，用户完全无从判断。
    if let Err(e) = sync_options(state, &version_id, &enabled_name, is_folder, enable, kind).await {
        if let Err(rollback_err) = pack_common::toggle_entry(&dir, &new_name, !enable) {
            log_error!("[Packs] 回滚文件名失败: {}", rollback_err);
        }
        return Err(e);
    }
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
    // 文件已经删了，回滚不现实；但必须让用户知道开关状态没同步
    sync_options(state, &version_id, &enabled_name, is_folder, false, kind).await?;
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
) -> Result<(), String> {
    let game_dir = pack_common::resolve_effective_game_dir(state, version_id)
        .await
        .map_err(|e| format!("定位游戏目录失败，开关状态未同步：{}", e))?;
    match kind {
        PackKind::Resourcepack => {
            // 版本号决定资源包引用要不要带 file/ 前缀，取不到就不能瞎猜：
            // 猜错会写进一条游戏不认的引用，表现为「已启用但没加载」。
            let mc_version = crate::commands::version::list::get_version_game_version(
                state,
                version_id.to_string(),
            )
            .await
            .map_err(|e| format!("读取游戏版本失败，开关状态未同步：{}", e))?
            .ok_or_else(|| "无法确定游戏版本，开关状态未同步".to_string())?;
            resourcepack_options::set_resource_pack_enabled(
                &game_dir,
                enabled_name,
                is_folder,
                enabled,
                &mc_version,
            )?;
        }
        PackKind::Shader => {
            let value = if enabled { Some(enabled_name) } else { None };
            resourcepack_options::set_shader_pack(&game_dir, value)?;
        }
    }
    Ok(())
}
