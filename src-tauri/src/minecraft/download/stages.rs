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
pub async fn download_client_jar(
    json: &serde_json::Value,
    game_dir: &Path,
    version_id: &str,
    mirror_url: Option<&str>,
    manager: &DownloadManager,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
) -> anyhow::Result<()> {
    // 用 find_original_version 确定主 jar 的正确位置
    // 有 inheritsFrom 时主 jar 在父版本目录下（如 Fabric 版本的主 jar 在原版目录）
    // 无 inheritsFrom 时主 jar 在当前版本目录下
    let jar_version = crate::minecraft::launch::classpath::find_original_version(game_dir, json);
    let jar_path = game_dir
        .join("versions")
        .join(&jar_version)
        .join(format!("{}.jar", jar_version));

    let checker = FileChecker::new()
        .with_min_size(1024)
        .with_actual_size(json["downloads"]["client"]["size"].as_i64().unwrap_or(-1))
        .with_hash(
            json["downloads"]["client"]["sha1"]
                .as_str()
                .map(|s| s.to_string()),
        );

    if checker.is_valid(&jar_path.to_string_lossy()) {
        log_info!("[Download] Client JAR already exists at {}, skipping", jar_path.display());
        return Ok(());
    }

    let url = json["downloads"]["client"]["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Client JAR URL not found in version {} (merged json may be missing downloads.client)", version_id))?;

    log_info!("[Download] Downloading client JAR to {}", jar_path.display());

    let urls = build_launcher_meta_urls(url, mirror_url, source_mode_of(manager));
    let task = DownloadTask {
        id: "client_jar".to_string(),
        urls,
        local_path: jar_path.to_string_lossy().to_string(),
        expected_size: json["downloads"]["client"]["size"].as_i64().unwrap_or(0),
        expected_hash: json["downloads"]["client"]["sha1"]
            .as_str()
            .map(|s| s.to_string()),
    };

    let results = manager.download_batch(vec![task], progress_callback).await;

    if let Some(result) = results.first() {
        if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
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
            let urls = assets::build_asset_download_urls(asset, mirror_url, source_mode_of(manager));
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
