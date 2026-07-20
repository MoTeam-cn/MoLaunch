//! 补全版本文件（主 jar + libraries + assets）

use std::path::Path;

use crate::log_info;
use super::super::sources::DownloadSourceMode;
use super::super::version::json_merge;
use super::manager::DownloadManager;
use super::stages::{download_assets, download_client_jar, download_libraries};

/// 补全版本文件
pub async fn fix_version_files(
    version_id: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    max_threads: usize,
    chunk_count: usize,
    speed_limit: u64,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<()> {
    let json_path = game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}.json", version_id));

    if !json_path.exists() {
        return Err(anyhow::anyhow!("Version {} JSON not found", version_id));
    }

    let json_content = std::fs::read_to_string(&json_path)?;
    let json: serde_json::Value = serde_json::from_str(&json_content)?;

    // merge_version_json 会处理父版本不存在的情况（容错机制）
    let merged_json = json_merge::merge_version_json(&json, game_dir)?;

    // 复用单个 DownloadManager 实例（与 download_version_full 一致）
    let manager = DownloadManager::new(max_threads, chunk_count, speed_limit, source_mode);

    // 1. 下载主 jar（client.jar）
    // 修复：传原始 json 给 download_client_jar（含 inheritsFrom，用于判断 jar 路径）
    // 之前只传 merged_json，但 merge_version_json 会移除 inheritsFrom，
    // 导致 find_original_version 找不到父版本，jar 路径错误，启动时重复下载
    if let Err(e) = download_client_jar(
        &json,
        &merged_json,
        game_dir,
        version_id,
        mirror_url,
        &manager,
        None,
    )
    .await
    {
        log_info!("[Fix] download_client_jar failed (may be expected for some versions): {}", e);
    }

    // 2. 下载 Libraries（启动时用快速检查模式）
    // 启动时的文件补全：使用快速检查模式（只检查文件存在 + 大小，不计算 SHA1）
    // 启动时只构建 classpath，不做哈希校验，避免每次启动卡顿
    // 文件下载时已经做过完整校验，正常情况下不会损坏
    let _ = download_libraries(
        &merged_json,
        game_dir,
        mirror_url,
        &manager,
        None,
        true, // quick_check
    )
    .await?;

    // 3. 下载 Assets（启动时用快速检查模式）
    let _ = download_assets(
        &merged_json,
        game_dir,
        mirror_url,
        &manager,
        None,
        true, // quick_check
    )
    .await?;

    Ok(())
}
