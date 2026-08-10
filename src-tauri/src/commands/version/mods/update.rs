//! Mod 更新命令（update_mod）
//! 阶段 4 新增：封装"下载新版本 → 删旧版本"为原子操作。前端 `useModUpdate.ts::installSelected`
//! 从 3 个 IPC（getVersionModsDir + downloadResourceToPath + deleteMod）降为 1 个 IPC（update_mod）。
//! 原子性保证：下载失败时不删旧文件，下载成功才删旧文件。进度通过 `DownloadSession` 统一推送。
//! 实现委托 `pack_common::download_and_replace`（mods / resourcepacks / shaderpacks 共用）。

use crate::log_info;
use crate::state::AppState;

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::get_mods_dir;
use crate::utils::path::sanitize_file_name;

/// 更新 Mod：下载新版本 + 删除旧版本（原子操作）
///
/// 流程：取 mods 目录 → DownloadSession（"Mod 更新"，2 stages）→ 用 cdn_urls 多 URL fallback
/// 下载新版本 → 失败 mark_failed 保留旧文件；成功则删旧文件（仅文件名不同）并 mark_complete。
pub async fn update_mod(
    state: &AppState,
    version_id: String,
    old_file_name: String,
    download_url: String,
    new_file_name: String,
    expected_size: i64,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&new_file_name)?;
    // 旧文件名可能是 .disabled 后缀，也需校验
    sanitize_file_name(&old_file_name)?;

    log_info!(
        "[Mods] 更新 mod: version={} old={} new={} url={}",
        version_id,
        old_file_name,
        new_file_name,
        download_url
    );

    let mods_dir = get_mods_dir(state, &version_id).await?;
    pack_common::download_and_replace(
        state,
        &mods_dir,
        &old_file_name,
        &download_url,
        &new_file_name,
        expected_size,
        "Mod 更新",
    )
    .await
}
