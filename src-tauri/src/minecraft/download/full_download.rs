//! 完整版本下载主流程

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::log_debug;

use super::super::sources::DownloadSourceMode;
use super::super::version::json_merge;
use super::manager::DownloadManager;
use super::stages::{download_assets, download_client_jar, download_libraries};
use super::types::GlobalProgress;
use super::util::fetch_with_retry;
use super::version_list::{fetch_version_list, get_version_json_url};
use crate::state::AppState;

/// 版本下载结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDownloadResult {
    pub version_id: String,
    pub libs_total: usize,
    pub libs_downloaded: usize,
    pub libs_skipped: usize,
    pub assets_total: usize,
    pub assets_downloaded: usize,
    pub assets_skipped: usize,
}

/// 完整版本下载
///
/// 改造：参数收敛为 `state`，内部用 `DownloadManager::from_state` 统一构造，
/// 取消/暂停 flag 自动接入 `state.download_cancel_flag` / `download_pause_flag`，
/// 调用方只需关心 `progress_callback` / `stage_callback`。
pub async fn download_version_full(
    state: &AppState,
    version_id: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
    stage_callback: Option<Arc<dyn Fn(usize, &str) + Send + Sync>>,
) -> anyhow::Result<VersionDownloadResult> {
    let version_dir = game_dir.join("versions").join(version_id);
    std::fs::create_dir_all(&version_dir)?;

    // 复用单个 DownloadManager 实例（避免每个阶段 new 一个独立 manager + 独立 timer）
    // client_jar / asset_index 只传 1 个 task（自然单线程），libraries / assets 传多 task
    // 参数统一从 state 读取（max_threads/chunk_count/speed_limit/source_mode），
    // flag 自动接入 state 的全局 cancel/pause flag
    let manager = DownloadManager::from_state(state)
        .await
        .with_cancel_flag(state.download_cancel_flag.clone())
        .with_pause_flag(state.download_pause_flag.clone());

    // Step 1: 版本清单
    if let Some(ref cb) = stage_callback {
        cb(0, "版本清单");
    }
    log_debug!("[Download] Step 1/5: Fetching version JSON URL");
    let version_list = fetch_version_list(mirror_url, source_mode).await?;
    let json_url = get_version_json_url(&version_list.value, version_id)
        .ok_or_else(|| anyhow::anyhow!("Version {} not found", version_id))?;

    log_debug!("[Download] Step 1/5: Downloading version JSON");
    let json_path = version_dir.join(format!("{}.json", version_id));
    let json_content = fetch_with_retry(&json_url, &json_path, mirror_url, source_mode).await?;
    let version_json: serde_json::Value = serde_json::from_str(&json_content)?;

    // Step 2: 版本信息（合并 JSON 继承链）
    if let Some(ref cb) = stage_callback {
        cb(1, "版本信息");
    }
    log_debug!("[Download] Step 2/5: Merging JSON inheritance");
    let merged_json = json_merge::merge_version_json(&version_json, game_dir)?;
    let merged_json_str = serde_json::to_string_pretty(&merged_json)?;
    std::fs::write(&json_path, &merged_json_str)?;

    // Step 3: 客户端
    if let Some(ref cb) = stage_callback {
        cb(2, "客户端");
    }
    log_debug!("[Download] Step 3/5: Downloading client JAR");
    download_client_jar(
        &version_json,
        &merged_json,
        game_dir,
        version_id,
        mirror_url,
        &manager,
        progress_callback.clone(),
    )
    .await?;

    // Step 4: 库文件
    if let Some(ref cb) = stage_callback {
        cb(3, "库文件");
    }
    log_debug!("[Download] Step 4/5: Downloading Libraries");
    let (libs_total, libs_downloaded, libs_skipped) = download_libraries(
        &merged_json,
        game_dir,
        mirror_url,
        &manager,
        progress_callback.clone(),
        false, // 完整校验模式（下载时严格校验哈希）
    )
    .await?;

    // Step 5: 资源文件
    if let Some(ref cb) = stage_callback {
        cb(4, "资源文件");
    }
    log_debug!("[Download] Step 5/5: Downloading Assets");
    let (assets_total, assets_downloaded, assets_skipped) = download_assets(
        &merged_json,
        game_dir,
        mirror_url,
        &manager,
        progress_callback,
        false, // 完整校验模式（下载时严格校验哈希）
    )
    .await?;

    log_debug!(
        "[Download] Done: Libs {}/{}, Assets {}/{}",
        libs_downloaded,
        libs_total,
        assets_downloaded,
        assets_total
    );

    Ok(VersionDownloadResult {
        version_id: version_id.to_string(),
        libs_total,
        libs_downloaded,
        libs_skipped,
        assets_total,
        assets_downloaded,
        assets_skipped,
    })
}
