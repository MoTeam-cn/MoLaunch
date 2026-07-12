//! 下载管理模块 - 完整版本下载流程

pub mod assets;
pub mod chunk;
pub mod downloader;
pub mod manager;
pub mod rate_limiter;
pub mod types;

use crate::http;
use crate::{log_debug, log_info};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use super::sources::{self, DownloadSourceMode};
use super::utils::file_checker::FileChecker;
use super::version::libraries;
use manager::DownloadManager;
use types::{DownloadStatus, DownloadTask, GlobalProgress};

/// 版本列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionListResult {
    pub source_name: String,
    pub is_official: bool,
    pub value: serde_json::Value,
}

/// 版本条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub id: String,
    pub version_type: String,
    pub time: String,
    pub release_time: String,
    pub url: String,
}

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

/// 获取版本列表
pub async fn fetch_version_list(
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<VersionListResult> {
    let urls = sources::build_urls(
        mirror_url,
        &format!(
            "{}/mc/game/version_manifest.json",
            sources::MOJANG_LAUNCHERMETA
        ),
        sources::BMCLAPI_VERSION_MANIFEST,
        source_mode,
    );

    let content = sources::fetch_with_fallback(&urls).await?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let source_name = if urls
        .first()
        .map_or(false, |u: &String| u.contains("bmclapi"))
    {
        "BMCLAPI"
    } else if mirror_url.is_some()
        && urls
            .first()
            .map_or(false, |u: &String| u.starts_with(mirror_url.unwrap_or("")))
    {
        "Mirror"
    } else {
        "Mojang"
    };

    Ok(VersionListResult {
        source_name: source_name.to_string(),
        is_official: source_name == "Mojang",
        value: json,
    })
}

/// 获取版本 JSON URL
pub fn get_version_json_url(version_list: &serde_json::Value, version_id: &str) -> Option<String> {
    if let Some(versions) = version_list["versions"].as_array() {
        for version in versions {
            if let Some(id) = version["id"].as_str() {
                if id == version_id {
                    return version["url"].as_str().map(|s| s.to_string());
                }
            }
        }
    }
    None
}

/// 解析版本列表
pub fn parse_version_list(version_list: &serde_json::Value) -> Vec<VersionEntry> {
    let mut entries = Vec::new();
    if let Some(versions) = version_list["versions"].as_array() {
        for version in versions {
            if let (Some(id), Some(version_type), Some(time), Some(release_time), Some(url)) = (
                version["id"].as_str(),
                version["type"].as_str(),
                version["time"].as_str(),
                version["releaseTime"].as_str(),
                version["url"].as_str(),
            ) {
                // 检测愚人节版本，修正 type
                let actual_type =
                    if super::fools::detect_fool(id, version_type, release_time).is_some() {
                        "fool"
                    } else {
                        version_type
                    };

                entries.push(VersionEntry {
                    id: id.to_string(),
                    version_type: actual_type.to_string(),
                    time: time.to_string(),
                    release_time: release_time.to_string(),
                    url: url.to_string(),
                });
            }
        }
    }
    entries
}

/// 获取最新版本
pub fn get_latest_versions(version_list: &serde_json::Value) -> (Option<String>, Option<String>) {
    let latest_release = version_list["latest"]["release"]
        .as_str()
        .map(|s| s.to_string());
    let latest_snapshot = version_list["latest"]["snapshot"]
        .as_str()
        .map(|s| s.to_string());
    (latest_release, latest_snapshot)
}

/// 完整版本下载
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub async fn download_version_full(
    version_id: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    max_threads: usize,
    chunk_count: usize,
    speed_limit: u64,
    source_mode: DownloadSourceMode,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
    stage_callback: Option<Arc<dyn Fn(usize, &str) + Send + Sync>>,
) -> anyhow::Result<VersionDownloadResult> {
    let version_dir = game_dir.join("versions").join(version_id);
    std::fs::create_dir_all(&version_dir)?;

    // Step 1: 版本清单
    if let Some(ref cb) = stage_callback {
        cb(0, "版本清单");
    }
    log_info!("[Download] Step 1/5: Fetching version JSON URL");
    let version_list = fetch_version_list(mirror_url, source_mode).await?;
    let json_url = get_version_json_url(&version_list.value, version_id)
        .ok_or_else(|| anyhow::anyhow!("Version {} not found", version_id))?;

    log_info!("[Download] Step 1/5: Downloading version JSON");
    let json_path = version_dir.join(format!("{}.json", version_id));
    let json_content = fetch_with_retry(&json_url, &json_path, mirror_url, source_mode).await?;
    let version_json: serde_json::Value = serde_json::from_str(&json_content)?;

    // Step 2: 版本信息（合并 JSON 继承链）
    if let Some(ref cb) = stage_callback {
        cb(1, "版本信息");
    }
    log_info!("[Download] Step 2/5: Merging JSON inheritance");
    let merged_json = super::version::json_merge::merge_version_json(&version_json, game_dir)?;
    let merged_json_str = serde_json::to_string_pretty(&merged_json)?;
    std::fs::write(&json_path, &merged_json_str)?;

    // Step 3: 客户端
    if let Some(ref cb) = stage_callback {
        cb(2, "客户端");
    }
    log_info!("[Download] Step 3/5: Downloading client JAR");
    download_client_jar(
        &merged_json,
        game_dir,
        version_id,
        mirror_url,
        chunk_count,
        speed_limit,
        source_mode,
        progress_callback.clone(),
    )
    .await?;

    // Step 4: 库文件
    if let Some(ref cb) = stage_callback {
        cb(3, "库文件");
    }
    log_info!("[Download] Step 4/5: Downloading Libraries");
    let (libs_total, libs_downloaded, libs_skipped) = download_libraries(
        &merged_json,
        game_dir,
        mirror_url,
        max_threads,
        chunk_count,
        speed_limit,
        source_mode,
        progress_callback.clone(),
    )
    .await?;

    // Step 5: 资源文件
    if let Some(ref cb) = stage_callback {
        cb(4, "资源文件");
    }
    log_info!("[Download] Step 5/5: Downloading Assets");
    let (assets_total, assets_downloaded, assets_skipped) = download_assets(
        &merged_json,
        game_dir,
        mirror_url,
        max_threads,
        chunk_count,
        speed_limit,
        source_mode,
        progress_callback,
    )
    .await?;

    log_info!(
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

/// 下载客户端 JAR
#[allow(clippy::too_many_arguments)]
async fn download_client_jar(
    json: &serde_json::Value,
    game_dir: &Path,
    version_id: &str,
    mirror_url: Option<&str>,
    chunk_count: usize,
    speed_limit: u64,
    source_mode: DownloadSourceMode,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
) -> anyhow::Result<()> {
    let jar_path = game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}.jar", version_id));

    let checker = FileChecker::new()
        .with_min_size(1024)
        .with_actual_size(json["downloads"]["client"]["size"].as_i64().unwrap_or(-1))
        .with_hash(
            json["downloads"]["client"]["sha1"]
                .as_str()
                .map(|s| s.to_string()),
        );

    if checker.is_valid(&jar_path.to_string_lossy()) {
        log_info!("[Download] Client JAR already exists, skipping");
        return Ok(());
    }

    let url = json["downloads"]["client"]["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Client JAR URL not found"))?;

    let urls = build_launcher_meta_urls(url, mirror_url, source_mode);
    let task = DownloadTask {
        id: "client_jar".to_string(),
        urls,
        local_path: jar_path.to_string_lossy().to_string(),
        expected_size: json["downloads"]["client"]["size"].as_i64().unwrap_or(0),
        expected_hash: json["downloads"]["client"]["sha1"]
            .as_str()
            .map(|s| s.to_string()),
    };

    let manager = DownloadManager::new(1, chunk_count, speed_limit, source_mode);
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
#[allow(clippy::too_many_arguments)]
async fn download_libraries(
    json: &serde_json::Value,
    game_dir: &Path,
    mirror_url: Option<&str>,
    max_threads: usize,
    chunk_count: usize,
    speed_limit: u64,
    source_mode: DownloadSourceMode,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
) -> anyhow::Result<(usize, usize, usize)> {
    let all_libs = libraries::parse_libraries(json, game_dir);
    let missing_libs = libraries::find_missing_libs(&all_libs, game_dir);

    log_info!(
        "[Libraries] Total: {}, Missing: {}",
        all_libs.len(),
        missing_libs.len()
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

    let manager = DownloadManager::new(max_threads, chunk_count, speed_limit, source_mode);
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
#[allow(clippy::too_many_arguments)]
async fn download_assets(
    json: &serde_json::Value,
    game_dir: &Path,
    mirror_url: Option<&str>,
    max_threads: usize,
    chunk_count: usize,
    speed_limit: u64,
    source_mode: DownloadSourceMode,
    progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
) -> anyhow::Result<(usize, usize, usize)> {
    let index_meta = assets::get_asset_index_meta(json)
        .ok_or_else(|| anyhow::anyhow!("Asset index metadata not found"))?;

    let index_path = assets::get_asset_index_path(game_dir, &index_meta.id);

    if !index_path.exists() {
        log_info!("[Assets] Downloading asset index: {}", index_meta.id);
        let index_urls = assets::get_asset_index_urls(&index_meta, source_mode);
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

        let manager = DownloadManager::new(1, chunk_count, speed_limit, source_mode);
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
    let missing_assets = assets::find_missing_assets(&all_assets);

    log_info!(
        "[Assets] Total: {}, Missing: {}",
        all_assets.len(),
        missing_assets.len()
    );

    if missing_assets.is_empty() {
        return Ok((all_assets.len(), 0, all_assets.len()));
    }

    let tasks: Vec<DownloadTask> = missing_assets
        .iter()
        .enumerate()
        .map(|(i, asset)| {
            let urls = assets::build_asset_download_urls(asset, mirror_url, source_mode);
            DownloadTask {
                id: format!("asset_{}", i),
                urls,
                local_path: asset.local_path.clone(),
                expected_size: asset.size,
                expected_hash: Some(asset.hash.clone()),
            }
        })
        .collect();

    let manager = DownloadManager::new(max_threads, chunk_count, speed_limit, source_mode);
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

/// 构建 launcher/meta URL 列表
fn build_launcher_meta_urls(
    original: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> Vec<String> {
    sources::build_replace_urls(
        original,
        mirror_url,
        sources::MOJANG_REPLACEMENTS,
        source_mode,
    )
}

/// 带重试的下载
async fn fetch_with_retry(
    primary_url: &str,
    local_path: &Path,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<String> {
    let urls = build_launcher_meta_urls(primary_url, mirror_url, source_mode);

    for url in &urls {
        match fetch_url_to_file(url, local_path).await {
            Ok(content) => return Ok(content),
            Err(e) => {
                log_debug!("Failed to fetch from {}: {}", url, e);
                continue;
            }
        }
    }

    Err(anyhow::anyhow!("All download sources failed"))
}

/// 下载 URL 内容到文件
async fn fetch_url_to_file(url: &str, local_path: &Path) -> anyhow::Result<String> {
    http::fetch_url_to_file(url, local_path).await
}

/// 获取 URL 内容
pub async fn fetch_url(url: &str) -> anyhow::Result<String> {
    http::fetch_url(url).await
}

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
    let merged_json = super::version::json_merge::merge_version_json(&json, game_dir)?;

    let _ = download_libraries(
        &merged_json,
        game_dir,
        mirror_url,
        max_threads,
        chunk_count,
        speed_limit,
        source_mode,
        None,
    )
    .await?;
    let _ = download_assets(
        &merged_json,
        game_dir,
        mirror_url,
        max_threads,
        chunk_count,
        speed_limit,
        source_mode,
        None,
    )
    .await?;

    Ok(())
}
