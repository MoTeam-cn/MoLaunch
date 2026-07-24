//! 社区资源下载安装 - install_modpack 阶段辅助函数
//!
//! 从 install_modpack 中抽取的两个独立阶段，降低 install_modpack 自身行数：
//! - `download_modpack_archive`：Stage 0，下载原始整合包到 instance 目录
//! - `parse_modpack_info`：Stage 1，解析 manifest/index 得到整合包信息

use crate::log_info;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::DownloadStatus;
use crate::minecraft::download::types::DownloadTask;
use crate::minecraft::sources::DownloadSourceMode;
use crate::state::{AppState, StageStatus};
use std::sync::Arc;

use super::concurrent::DetectedModpack;
use super::curseforge::CfManifest;
use super::helpers::{parse_cf_loader_id, parse_mr_loader};
use super::hmcl::HmclManifest;
use super::mcbbs::McbbsManifest;
use super::mmc::MmcPack;
use super::modrinth::MrIndex;
use super::types::{ModpackFormat, ModpackInfo};
use crate::utils::format;

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
        urls: crate::minecraft::sources::cdn_urls(download_url),
        local_path: archive_path.to_string_lossy().to_string(),
        expected_size: 0, // 由 DownloadManager 自动探测
        expected_hash: None,
    };

    // stage 0 的进度回调：统一用 sync_stage_from_progress 同步 GlobalProgress 到 download_state
    let stage0_state = state.download_state.clone();
    let stage0_callback: Arc<
        dyn Fn(crate::minecraft::download::types::GlobalProgress) + Send + Sync,
    > = Arc::new(move |p| {
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
    let chunk_count = config.download.chunk_count.max(1) as usize;
    drop(config);
    let archive_manager = DownloadManager::new(4, chunk_count, 0, DownloadSourceMode::Smart)
        .with_cancel_flag(state.download_cancel_flag.clone())
        .with_pause_flag(state.download_pause_flag.clone());
    let archive_results = archive_manager
        .download_batch(vec![archive_task], Some(stage0_callback))
        .await;

    let archive_err = archive_results.first().and_then(|r| {
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
        format::bytes(archive_size)
    );

    Ok(archive_size)
}

/// Stage 1：解析整合包 manifest/index 得到整合包信息
///
/// 根据 format 分支解析对应 manifest，提取：
/// - game_version（CF: minecraft.version / MR: dependencies["minecraft"] / HMCL: gameVersion /
///   MMC: components[net.minecraft].version / MCBBS: addons[game]）
/// - loader + loader_version（CF: modLoaders primary / MR: dependencies["fabric-loader"|...] /
///   MMC: components[net.minecraftforge|net.fabricmc.fabric-loader|net.neoforged] /
///   MCBBS: addons[forge|fabric|neoforge|optifine]）
/// - mod_files_count（CF/MR: files 数组长度 / HMCL/MMC/MCBBS: 0，mods 已打包在 overrides 中）
/// - archive_base_folder（关键文件所在层级前缀，供 extract_overrides 构造完整前缀）
pub(super) fn parse_modpack_info(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let format = detected.format;
    let archive_base_folder = detected.archive_base_folder.clone();

    match format {
        ModpackFormat::Curseforge => {
            let manifest: CfManifest = serde_json::from_str(
                detected.manifest_content.as_deref().unwrap_or(""),
            )
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
                archive_base_folder,
                cf_manifest: Some(manifest),
                mr_index: None,
                hmcl_manifest: None,
                mmc_pack: None,
                mcbbs_manifest: None,
            })
        }
        ModpackFormat::Modrinth => {
            let index: MrIndex = serde_json::from_str(
                detected.index_content.as_deref().unwrap_or(""),
            )
            .map_err(|e| format!("解析 modrinth.index.json 失败: {}", e))?;
            let gv = index
                .dependencies
                .get("minecraft")
                .cloned()
                .unwrap_or_default();
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
                archive_base_folder,
                cf_manifest: None,
                mr_index: Some(index),
                hmcl_manifest: None,
                mmc_pack: None,
                mcbbs_manifest: None,
            })
        }
        ModpackFormat::Hmcl => {
            let manifest: HmclManifest = serde_json::from_str(
                detected.hmcl_content.as_deref().unwrap_or(""),
            )
            .map_err(|e| format!("解析 modpack.json 失败: {}", e))?;
            let gv = manifest.game_version.clone();
            // HMCL 整合包不指定加载器版本，仅含游戏版本；加载器信息（如有）打包在 overrides 中
            // PCL2 的 InstallPackHMCL 也只读 gameVersion，不读 loader
            let count = 0;
            log_info!(
                "[Community] HMCL 整合包: game={} name={}",
                gv,
                manifest.name
            );
            Ok(ModpackInfo {
                format,
                game_version: gv,
                loader: String::new(),
                loader_version: String::new(),
                mod_files_count: count,
                archive_base_folder,
                cf_manifest: None,
                mr_index: None,
                hmcl_manifest: Some(manifest),
                mmc_pack: None,
                mcbbs_manifest: None,
            })
        }
        ModpackFormat::Mmc => {
            let pack: MmcPack = serde_json::from_str(
                detected.mmc_content.as_deref().unwrap_or(""),
            )
            .map_err(|e| format!("解析 mmc-pack.json 失败: {}", e))?;
            // 从 components 提取 game_version 和 loader
            let mut gv = String::new();
            let mut loader = String::new();
            let mut loader_ver = String::new();
            for comp in &pack.components {
                match comp.uid.as_str() {
                    "net.minecraft" => gv = comp.version.clone(),
                    "net.minecraftforge" => {
                        loader = "forge".to_string();
                        loader_ver = comp.version.clone();
                    }
                    "net.neoforged" => {
                        loader = "neoforge".to_string();
                        loader_ver = comp.version.clone();
                    }
                    "net.fabricmc.fabric-loader" => {
                        loader = "fabric".to_string();
                        loader_ver = comp.version.clone();
                    }
                    _ => {
                        // 跳过 org.lwjgl.* 等，与 PCL2 一致
                        if !comp.uid.starts_with("org.lwjgl") {
                            log_info!(
                                "[Community] MMC 整合包跳过不支持的组件: uid={} version={}",
                                comp.uid,
                                comp.version
                            );
                        }
                    }
                }
            }
            if gv.is_empty() {
                return Err(
                    "MMC 整合包未提供 game 版本（缺少 net.minecraft 组件）".to_string(),
                );
            }
            log_info!(
                "[Community] MMC 整合包: game={} loader={}{}",
                gv,
                loader,
                if loader_ver.is_empty() {
                    String::new()
                } else {
                    format!("@{}", loader_ver)
                }
            );
            Ok(ModpackInfo {
                format,
                game_version: gv,
                loader,
                loader_version: loader_ver,
                mod_files_count: 0,
                archive_base_folder,
                cf_manifest: None,
                mr_index: None,
                hmcl_manifest: None,
                mmc_pack: Some(pack),
                mcbbs_manifest: None,
            })
        }
        ModpackFormat::Mcbbs => {
            let manifest: McbbsManifest = serde_json::from_str(
                detected.manifest_content.as_deref().unwrap_or(""),
            )
            .map_err(|e| format!("解析 mcbbs.packmeta/manifest.json 失败: {}", e))?;
            // 从 addons 提取 game_version 和 loader
            let mut gv = String::new();
            let mut loader = String::new();
            let mut loader_ver = String::new();
            for addon in &manifest.addons {
                match addon.id.as_str() {
                    "game" => gv = addon.version.clone(),
                    "forge" => {
                        loader = "forge".to_string();
                        loader_ver = addon.version.clone();
                    }
                    "neoforge" => {
                        loader = "neoforge".to_string();
                        loader_ver = addon.version.clone();
                    }
                    "fabric" => {
                        loader = "fabric".to_string();
                        loader_ver = addon.version.clone();
                    }
                    "optifine" => {
                        // OptiFine 作为独立加载器，与 PCL2 一致（OptiFineVersion 字段）
                        loader = "optifine".to_string();
                        loader_ver = addon.version.clone();
                    }
                    "quilt" => {
                        // PCL2 直接拒绝 quilt，MoLaunch 暂也不支持
                        return Err(
                            "MCBBS 整合包要求 Quilt 加载器，MoLaunch 暂不支持 Quilt".to_string(),
                        );
                    }
                    _ => {
                        log_info!(
                            "[Community] MCBBS 整合包跳过未知 addon: id={} version={}",
                            addon.id,
                            addon.version
                        );
                    }
                }
            }
            if gv.is_empty() {
                return Err(
                    "MCBBS 整合包未提供 game 版本（addons 中缺少 id=game 项）".to_string(),
                );
            }
            log_info!(
                "[Community] MCBBS 整合包: game={} loader={}{} name={}",
                gv,
                loader,
                if loader_ver.is_empty() {
                    String::new()
                } else {
                    format!("@{}", loader_ver)
                },
                manifest.name
            );
            Ok(ModpackInfo {
                format,
                game_version: gv,
                loader,
                loader_version: loader_ver,
                mod_files_count: 0,
                archive_base_folder,
                cf_manifest: None,
                mr_index: None,
                hmcl_manifest: None,
                mmc_pack: None,
                mcbbs_manifest: Some(manifest),
            })
        }
    }
}
