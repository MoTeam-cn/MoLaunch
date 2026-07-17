//! 社区资源下载安装 - install_modpack 阶段辅助函数
//!
//! 从 install_modpack 中抽取的两个独立阶段，降低 install_modpack 自身行数：
//! - `download_modpack_archive`：Stage 0，下载原始整合包到 instance 目录
//! - `parse_modpack_info`：Stage 1，解析 manifest.json / modrinth.index.json 得到整合包信息

use crate::log_info;
use crate::state::{AppState, StageStatus};
use crate::minecraft::download::types::DownloadStatus;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::DownloadTask;
use crate::minecraft::sources::DownloadSourceMode;
use std::sync::Arc;

use super::curseforge::CfManifest;
use super::helpers::{format_bytes, parse_cf_loader_id, parse_mr_loader};
use super::modrinth::MrIndex;
use super::types::{ModpackFormat, ModpackInfo};

/// Stage 0：下载原始整合包到 instance 目录
///
/// 通过 DownloadManager 下载（自动分片 + 多线程 + 重试 + URL fallback），
/// 进度通过 `sync_stage_from_progress` 统一同步到 download_state 的 Stage 0。
///
/// 返回 archive_size（字节数），供日志输出。
pub(super) async fn download_modpack_archive(
    state: &AppState,
    archive_path: &std::path::Path,
    download_url: &str,
    file_name: &str,
) -> Result<u64, String> {
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(0, StageStatus::Loading, 0.0);
    }

    log_info!("[Community] 下载整合包到: {}", archive_path.display());

    let archive_task = DownloadTask {
        id: "modpack_archive".to_string(),
        urls: vec![download_url.to_string()],
        local_path: archive_path.to_string_lossy().to_string(),
        expected_size: 0, // 由 DownloadManager 自动探测
        expected_hash: None,
    };

    // stage 0 的进度回调：统一用 sync_stage_from_progress 同步 GlobalProgress 到 download_state
    let stage0_state = state.download_state.clone();
    let stage0_callback: Arc<dyn Fn(crate::minecraft::download::types::GlobalProgress) + Send + Sync> =
        Arc::new(move |p| {
            let mut ds = stage0_state.lock().unwrap();
            ds.sync_stage_from_progress(
                0,
                p.downloaded_bytes,
                p.total_bytes,
                p.completed_files,
                p.total_files,
                p.current_speed,
            );
        });

    let config = state.config.lock().await;
    let chunk_count = config.chunk_count.max(1) as usize;
    drop(config);
    let archive_manager = DownloadManager::new(4, chunk_count, 0, DownloadSourceMode::Smart);
    let archive_results = archive_manager
        .download_batch(vec![archive_task], Some(stage0_callback))
        .await;

    let archive_err = archive_results
        .first()
        .and_then(|r| {
            if r.status != DownloadStatus::Completed && r.status != DownloadStatus::Skipped {
                r.error.clone()
            } else {
                None
            }
        });

    if let Some(err) = archive_err {
        let msg = format!("下载整合包失败: {}", err);
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(0, StageStatus::Failed, 0.0);
            ds.mark_failed(1);
        }
        log_info!("[Community] 整合包安装失败: {}", msg);
        return Err(msg);
    }

    let archive_size = std::fs::metadata(archive_path)
        .map(|m| m.len())
        .unwrap_or(0);

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(0, StageStatus::Finished, 1.0);
    }
    log_info!(
        "[Community] 整合包下载完成: {} ({})",
        file_name,
        format_bytes(archive_size)
    );

    Ok(archive_size)
}

/// Stage 1：解析 manifest.json / modrinth.index.json 得到整合包信息
///
/// 根据 format 分支解析 CF manifest 或 MR index，提取：
/// - game_version（CF: minecraft.version / MR: dependencies["minecraft"]）
/// - loader + loader_version（CF: modLoaders primary / MR: dependencies["fabric-loader"|"quilt-loader"|...]）
/// - mod_files_count（files 数组长度）
/// - cf_manifest / mr_index（保留供 Stage 2 使用）
pub(super) fn parse_modpack_info(
    format: ModpackFormat,
    manifest_content: Option<&str>,
    index_content: Option<&str>,
) -> Result<ModpackInfo, String> {
    match format {
        ModpackFormat::Curseforge => {
            let manifest: CfManifest = serde_json::from_str(manifest_content.unwrap())
                .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;
            let gv = manifest.minecraft.version.clone();
            let (loader, ver) = manifest
                .minecraft
                .mod_loaders
                .iter()
                .find(|l| l.primary)
                .or_else(|| manifest.minecraft.mod_loaders.first())
                .map(|l| parse_cf_loader_id(&l.id))
                .unwrap_or((String::new(), String::new()));
            let count = manifest.files.len();
            Ok(ModpackInfo {
                format,
                game_version: gv,
                loader,
                loader_version: ver,
                mod_files_count: count,
                cf_manifest: Some(manifest),
                mr_index: None,
            })
        }
        ModpackFormat::Modrinth => {
            let index: MrIndex = serde_json::from_str(index_content.unwrap())
                .map_err(|e| format!("解析 modrinth.index.json 失败: {}", e))?;
            let gv = index.dependencies.get("minecraft").cloned().unwrap_or_default();
            let (loader, ver) = ["fabric-loader", "quilt-loader", "forge", "neoforge"]
                .iter()
                .find_map(|key| {
                    index.dependencies.get(*key).map(|v| {
                        let (ln, vv) = parse_mr_loader(key, v);
                        (ln.to_string(), vv)
                    })
                })
                .unwrap_or((String::new(), String::new()));
            let count = index.files.len();
            Ok(ModpackInfo {
                format,
                game_version: gv,
                loader,
                loader_version: ver,
                mod_files_count: count,
                cf_manifest: None,
                mr_index: Some(index),
            })
        }
    }
}
