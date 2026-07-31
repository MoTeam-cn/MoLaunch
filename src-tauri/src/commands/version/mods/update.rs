//! Mod 更新命令（update_mod）
//! 阶段 4 新增：封装"下载新版本 → 删旧版本"为原子操作。前端 `useModUpdate.ts::installSelected`
//! 从 3 个 IPC（getVersionModsDir + downloadResourceToPath + deleteMod）降为 1 个 IPC（update_mod）。
//! 原子性保证：下载失败时不删旧文件，下载成功才删旧文件。进度通过 `DownloadSession` 统一推送。

use crate::log_info;
use crate::minecraft::download::DownloadSession;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::state::AppState;

use super::super::sanitize_version_id;
use super::helpers::get_mods_dir;
use crate::utils::path::sanitize_file_name;

/// 更新 Mod：下载新版本 + 删除旧版本（原子操作）
///
/// 流程：取 mods 目录 → DownloadSession（"Mod 更新"，2 stages）→ 用 cdn_urls 多 URL fallback
/// 下载新版本 → 失败 mark_failed 保留旧文件；成功则删旧文件（仅文件名不同）并 mark_complete。
/// 原子性：下载失败不删旧文件。进度经 DownloadSession 推送，前端下载管理页可见。
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

    let mods_dir = get_mods_dir(&state, &version_id).await?;
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("创建 mods 目录失败: {}", e))?;
    }

    let target_path = mods_dir.join(&new_file_name);

    // 启动 DownloadSession：统一 reset_stages + flag 重置 + manager 构造
    // 2 个 stages：下载新版本（权重 80%）+ 替换旧版本（权重 20%，本地操作瞬时完成）
    let session = DownloadSession::start_grouped(
        state,
        "Mod 更新",
        vec![("下载新版本", 80.0), ("替换旧版本", 20.0)],
    )
    .await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = new_file_name.clone();
    }

    // 构造下载任务（cdn_urls 根据 source 策略生成多 URL fallback）
    let task = DownloadTask {
        id: format!("mod_update_{}", new_file_name),
        urls: crate::minecraft::sources::cdn_urls(&download_url),
        local_path: target_path.to_string_lossy().to_string(),
        expected_size,
        expected_hash: None,
    };

    // Stage 0：下载新版本
    let progress_callback = session.make_progress_callback(state, 0);
    let results = session
        .manager()
        .download_batch(vec![task], Some(progress_callback))
        .await;

    let result = results.first().ok_or("下载结果为空")?;

    if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
        let err = result
            .error
            .clone()
            .unwrap_or_else(|| "未知错误".to_string());
        // 下载失败：mark_failed，旧文件保留（不删旧）
        session.mark_failed(state, 1);
        log_info!("[Mods] mod 更新下载失败，旧文件保留: {}", err);
        return Err(err);
    }

    // Stage 1：删除旧文件（仅当文件名不同）
    // 文件名相同时跳过删除（同名覆盖已在下载阶段完成）
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_bytes(1, 1, 1); // 标记 stage 1 完成（本地操作瞬时）
    }
    if old_file_name != new_file_name {
        let old_path = mods_dir.join(&old_file_name);
        if old_path.exists() {
            if let Err(e) = std::fs::remove_file(&old_path) {
                // 删除旧文件失败不阻断流程（新版本已下载成功）
                log_info!("[Mods] 删除旧 mod 文件失败（不阻断）: {}", e);
            } else {
                log_info!("[Mods] 旧 mod 文件已删除: {}", old_file_name);
            }
        }
    }

    session.mark_complete(state);

    log_info!(
        "[Mods] mod 更新完成: {} ({} bytes)",
        new_file_name,
        std::fs::metadata(&target_path).map(|m| m.len()).unwrap_or(0)
    );

    Ok(())
}