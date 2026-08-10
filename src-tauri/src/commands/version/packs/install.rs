//! Pack 安装与文件操作命令（install_pack / open_packs_dir / reveal_pack_file / get_pack_icon）

use crate::error_util::log_err;
use crate::log_info;
use crate::state::AppState;
use crate::utils::path::sanitize_file_name;

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::resolve_packs_dir;
use super::icon::extract_pack_icon_data_url;
use super::types::PackKind;

/// 从外部路径安装资源包/光影（复制到目标目录，自动去除启停后缀）
pub async fn install_pack(
    state: &AppState,
    version_id: String,
    source_path: String,
    kind: PackKind,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Installing pack to version {}", version_id);

    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    let clean_name = pack_common::install_entry(&dir, &source_path, kind.suffixes())?;
    log_info!("Pack installed: {}", clean_name);
    Ok(())
}

/// 打开版本的内容目录（自动创建）
pub async fn open_packs_dir(
    state: &AppState,
    version_id: String,
    kind: PackKind,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(log_err("Failed to create packs directory"))?;
    }
    crate::minecraft::system::shell::open_path(&dir.to_string_lossy())
}

/// 在资源管理器中打开并选中指定文件
pub async fn reveal_pack_file(
    state: &AppState,
    version_id: String,
    file_name: String,
    kind: PackKind,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    let path = dir.join(&file_name);
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_name));
    }
    crate::minecraft::system::shell::reveal_in_file_manager(&path.to_string_lossy())
}

/// 提取包内图标为 base64 data URL（无图标返回 None）
pub async fn get_pack_icon(
    state: &AppState,
    version_id: String,
    file_name: String,
    kind: PackKind,
) -> Result<Option<String>, String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    let path = dir.join(&file_name);
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_name));
    }
    Ok(extract_pack_icon_data_url(&path))
}
