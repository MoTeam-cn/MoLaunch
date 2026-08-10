//! Pack 更新命令（update_pack）
//! 复用 `pack_common::download_and_replace` 原子更新（下载失败保留旧文件）。

use crate::log_info;
use crate::state::AppState;
use crate::utils::path::sanitize_file_name;

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::resolve_packs_dir;
use super::types::PackKind;

/// 更新资源包/光影：下载新版本 + 删除旧版本（原子操作）
pub async fn update_pack(
    state: &AppState,
    version_id: String,
    old_file_name: String,
    download_url: String,
    new_file_name: String,
    expected_size: i64,
    kind: PackKind,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&new_file_name)?;
    sanitize_file_name(&old_file_name)?;
    log_info!(
        "[Packs] 更新 pack: version={} old={} new={}",
        version_id,
        old_file_name,
        new_file_name
    );

    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    let label = match kind {
        PackKind::Resourcepack => "资源包更新",
        PackKind::Shader => "光影更新",
    };
    pack_common::download_and_replace(
        state,
        &dir,
        &old_file_name,
        &download_url,
        &new_file_name,
        expected_size,
        label,
    )
    .await
}
