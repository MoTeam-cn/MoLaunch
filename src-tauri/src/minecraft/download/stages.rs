//! 下载阶段函数：客户端 JAR / 库文件 / 资源文件

use std::path::Path;
use std::sync::Arc;

use crate::log_info;

use super::super::utils::file_checker::FileChecker;
use super::super::version::libraries;
use super::assets;
use super::manager::DownloadManager;
use super::types::{DownloadStatus, DownloadTask, GlobalProgress};
use super::util::{build_launcher_meta_urls, source_mode_of};

/// 下载客户端 JAR
///
/// `original_json`：原始版本 JSON（含 inheritsFrom，用于判断 jar 路径）
/// `merged_json`：合并后的 JSON（含 downloads.client.url，用于获取下载 URL）
/// 修复：之前只传 merged_json，但 merge_version_json 会移除 inheritsFrom，
/// 导致 find_original_version 找不到父版本，jar 路径错误，启动时重复下载
pub async fn download_client_jar(
    original_json: &serde_json::Value,
    merged_json: &serde_json::Value,
    game_dir: &Path,
    version_id: &str,
    mirror_url: Option<&str>,
    manager: &DownloadManager,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
) -> anyhow::Result<()> {
    // 用原始 json 的 inheritsFrom 确定主 jar 的正确位置
    // 有 inheritsFrom 时主 jar 在父版本目录下（如 Fabric 版本的主 jar 在原版目录）
    // 无 inheritsFrom 时主 jar 在当前版本目录下
    let jar_version =
        crate::minecraft::launch::classpath::find_original_version(game_dir, original_json);
    let jar_path = game_dir
        .join("versions")
        .join(&jar_version)
        .join(format!("{}.jar", jar_version));

    let checker = FileChecker::new()
        .with_min_size(1024)
        .with_actual_size(
            merged_json["downloads"]["client"]["size"]
                .as_i64()
                .unwrap_or(-1),
        )
        .with_hash(
            merged_json["downloads"]["client"]["sha1"]
                .as_str()
                .map(|s| s.to_string()),
        );

    if checker.is_valid(&jar_path.to_string_lossy()) {
        log_info!(
            "[Download] Client JAR already exists at {}, skipping",
            jar_path.display()
        );
        return Ok(());
    }

    let url = merged_json["downloads"]["client"]["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Client JAR URL not found in version {} (merged json may be missing downloads.client)", version_id))?;

    log_info!(
        "[Download] Downloading client JAR to {}",
        jar_path.display()
    );

    let urls = build_launcher_meta_urls(url, mirror_url, source_mode_of(manager));
    let task = DownloadTask {
        id: "client_jar".to_string(),
        urls,
        local_path: jar_path.to_string_lossy().to_string(),
        expected_size: merged_json["downloads"]["client"]["size"]
            .as_i64()
            .unwrap_or(0),
        expected_hash: merged_json["downloads"]["client"]["sha1"]
            .as_str()
            .map(|s| s.to_string()),
    };

    let results = manager.download_batch(vec![task], progress_callback).await;

    if let Some(result) = results.first() {
        if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
            // 取消导致的失败返回更友好的错误信息
            let err_msg = result.error.as_deref().unwrap_or("unknown error");
            if err_msg.contains("取消") || err_msg.contains("cancel") {
                return Err(anyhow::anyhow!("下载已取消"));
            }
            return Err(anyhow::anyhow!(
                "Failed to download client JAR: {:?}",
                result.error
            ));
        }
    }

    Ok(())
}

/// 下载 Libraries
pub async fn download_libraries(
    json: &serde_json::Value,
    game_dir: &Path,
    mirror_url: Option<&str>,
    manager: &DownloadManager,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
    quick_check: bool,
) -> anyhow::Result<(usize, usize, usize)> {
    let all_libs = libraries::parse_libraries(json, game_dir);
    let missing_libs = libraries::find_missing_libs(&all_libs, game_dir, quick_check);

    log_info!(
        "[Libraries] Total: {}, Missing: {} (mode: {})",
        all_libs.len(),
        missing_libs.len(),
        if quick_check { "quick" } else { "full" }
    );

    if missing_libs.is_empty() {
        return Ok((all_libs.len(), 0, all_libs.len()));
    }

    let tasks: Vec<DownloadTask> = missing_libs
        .iter()
        .enumerate()
        .map(|(i, lib)| {
            let urls = libraries::build_download_urls(lib, mirror_url);
            DownloadTask {
                id: format!("lib_{}", i),
                urls,
                local_path: lib.local_path.clone(),
                expected_size: lib.size,
                expected_hash: lib.sha1.clone(),
            }
        })
        .collect();

    let results = manager.download_batch(tasks, progress_callback).await;

    let downloaded = results
        .iter()
        .filter(|r| r.status == DownloadStatus::Completed)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == DownloadStatus::Skipped)
        .count();

    Ok((all_libs.len(), downloaded, skipped))
}

/// 下载 Assets
pub async fn download_assets(
    json: &serde_json::Value,
    game_dir: &Path,
    mirror_url: Option<&str>,
    manager: &DownloadManager,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
    quick_check: bool,
) -> anyhow::Result<(usize, usize, usize)> {
    let index_meta = assets::get_asset_index_meta(json)
        .ok_or_else(|| anyhow::anyhow!("Asset index metadata not found"))?;

    let index_path = assets::get_asset_index_path(game_dir, &index_meta.id);

    if !index_path.exists() {
        log_info!("[Assets] Downloading asset index: {}", index_meta.id);
        let index_urls = assets::get_asset_index_urls(&index_meta, source_mode_of(manager));
        let task = DownloadTask {
            id: "asset_index".to_string(),
            urls: index_urls,
            local_path: index_path.to_string_lossy().to_string(),
            expected_size: index_meta.size,
            expected_hash: if index_meta.sha1.is_empty() {
                None
            } else {
                Some(index_meta.sha1.clone())
            },
        };

        let results = manager.download_batch(vec![task], None).await;

        if let Some(result) = results.first() {
            if result.status != DownloadStatus::Completed
                && result.status != DownloadStatus::Skipped
            {
                return Err(anyhow::anyhow!("Failed to download asset index"));
            }
        }
    }

    let index_content = std::fs::read_to_string(&index_path)?;
    let index_json: serde_json::Value = serde_json::from_str(&index_content)?;
    let all_assets = assets::parse_asset_index(&index_json, game_dir);
    let missing_assets = assets::find_missing_assets(&all_assets, quick_check);

    log_info!(
        "[Assets] Total: {}, Missing: {} (mode: {})",
        all_assets.len(),
        missing_assets.len(),
        if quick_check { "quick" } else { "full" }
    );

    if missing_assets.is_empty() {
        return Ok((all_assets.len(), 0, all_assets.len()));
    }

    let tasks: Vec<DownloadTask> = missing_assets
        .iter()
        .enumerate()
        .map(|(i, asset)| {
            let urls =
                assets::build_asset_download_urls(asset, mirror_url, source_mode_of(manager));
            DownloadTask {
                id: format!("asset_{}", i),
                urls,
                local_path: asset.local_path.clone(),
                expected_size: asset.size,
                expected_hash: Some(asset.hash.clone()),
            }
        })
        .collect();

    let results = manager.download_batch(tasks, progress_callback).await;

    let downloaded = results
        .iter()
        .filter(|r| r.status == DownloadStatus::Completed)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == DownloadStatus::Skipped)
        .count();

    Ok((all_assets.len(), downloaded, skipped))
}
