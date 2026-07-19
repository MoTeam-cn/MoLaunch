//! 补全版本文件（libraries + assets）

use std::path::Path;

use super::super::sources::DownloadSourceMode;
use super::super::version::json_merge;
use super::manager::DownloadManager;
use super::stages::{download_assets, download_libraries};

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

    // merge_version_json 会处理父版本不存在的情况（参考PCL2的容错机制）
    let merged_json = json_merge::merge_version_json(&json, game_dir)?;

    // 复用单个 DownloadManager 实例（与 download_version_full 一致）
    let manager = DownloadManager::new(max_threads, chunk_count, speed_limit, source_mode);

    let _ = download_libraries(
        &merged_json,
        game_dir,
        mirror_url,
        &manager,
        None,
    )
    .await?;
    let _ = download_assets(
        &merged_json,
        game_dir,
        mirror_url,
        &manager,
        None,
    )
    .await?;

    Ok(())
}
