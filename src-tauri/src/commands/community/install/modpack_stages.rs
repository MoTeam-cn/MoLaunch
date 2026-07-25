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
            // Quilt 加载器检测：id 以 "quilt-" 开头直接报错
            for l in &manifest.minecraft.mod_loaders {
                if l.id.starts_with("quilt-") || l.id.starts_with("quilt_") {
                    return Err(
                        "CurseForge 整合包要求 Quilt 加载器，MoLaunch 暂不支持 Quilt".to_string(),
                    );
                }
            }
            // Forge recommended 字段检测：旧版整合包格式，直接报错提示版本过老
            for l in &manifest.minecraft.mod_loaders {
                if l.id.starts_with("forge-") && l.id.contains("recommended") {
                    return Err(
                        "该整合包版本过老（使用旧版 Forge recommended 格式），请尝试更新版本的整合包"
                            .to_string(),
                    );
                }
            }
            let (loader, ver) = manifest
                .minecraft
                .mod_loaders
                .iter()
                .find(|l| l.primary)
                .or_else(|| manifest.minecraft.mod_loaders.first())
                .map(|l| parse_cf_loader_id(&l.id))
                .unwrap_or((String::new(), String::new()));
            let count = manifest.files.len();
            let cf_overrides_name = manifest.overrides.clone();
            Ok(ModpackInfo {
                format,
                game_version: gv,
                loader,
                loader_version: ver,
                mod_files_count: count,
                archive_base_folder,
                cf_overrides_name,
                cf_manifest: Some(manifest),
                mr_index: None,
                hmcl_manifest: None,
                mmc_pack: None,
                mmc_cfg_content: None,
                mcbbs_manifest: None,
                launcher_inner_path: None,
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
            // Quilt 加载器检测：dependencies 含 quilt-loader 直接报错
            if index.dependencies.contains_key("quilt-loader") {
                return Err(
                    "Modrinth 整合包要求 Quilt 加载器，MoLaunch 暂不支持 Quilt".to_string(),
                );
            }
            let (loader, ver) = ["fabric-loader", "forge", "neoforge"]
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
                cf_overrides_name: None,
                cf_manifest: None,
                mr_index: Some(index),
                hmcl_manifest: None,
                mmc_pack: None,
                mmc_cfg_content: None,
                mcbbs_manifest: None,
                launcher_inner_path: None,
            })
        }
        ModpackFormat::Hmcl => {
            let manifest: HmclManifest = serde_json::from_str(
                detected.hmcl_content.as_deref().unwrap_or(""),
            )
            .map_err(|e| format!("解析 modpack.json 失败: {}", e))?;
            let gv = manifest.game_version.clone();
            // HMCL 整合包不指定加载器版本，仅含游戏版本；加载器信息（如有）打包在 overrides 中
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
                cf_overrides_name: None,
                cf_manifest: None,
                mr_index: None,
                hmcl_manifest: Some(manifest),
                mmc_pack: None,
                mmc_cfg_content: None,
                mcbbs_manifest: None,
                launcher_inner_path: None,
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
                        // 跳过 org.lwjgl.* 等
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
                cf_overrides_name: None,
                cf_manifest: None,
                mr_index: None,
                hmcl_manifest: None,
                mmc_pack: Some(pack),
                mmc_cfg_content: detected.mmc_cfg_content.clone(),
                mcbbs_manifest: None,
                launcher_inner_path: None,
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
                        // OptiFine 作为独立加载器
                        loader = "optifine".to_string();
                        loader_ver = addon.version.clone();
                    }
                    "quilt" => {
                        // MoLaunch 暂不支持 Quilt
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
                cf_overrides_name: None,
                cf_manifest: None,
                mr_index: None,
                hmcl_manifest: None,
                mmc_pack: None,
                mmc_cfg_content: None,
                mcbbs_manifest: Some(manifest),
                launcher_inner_path: None,
            })
        }
        ModpackFormat::LauncherPack => {
            // 带启动器整合包：本阶段只记录内层整合包路径，实际递归安装由 install 流程处理
            let inner_path = detected.launcher_inner_path.clone().ok_or_else(|| {
                "LauncherPack 检测异常：未记录内层整合包路径".to_string()
            })?;
            log_info!(
                "[Community] LauncherPack 整合包: 内层整合包路径={}",
                inner_path
            );
            Ok(ModpackInfo {
                format,
                game_version: String::new(),
                loader: String::new(),
                loader_version: String::new(),
                mod_files_count: 0,
                archive_base_folder: String::new(),
                cf_overrides_name: None,
                cf_manifest: None,
                mr_index: None,
                hmcl_manifest: None,
                mmc_pack: None,
                mmc_cfg_content: None,
                mcbbs_manifest: None,
                launcher_inner_path: Some(inner_path),
            })
        }
        ModpackFormat::Compress => {
            // 普通压缩包兜底：archive_base_folder 已是 `.minecraft/` 前缀
            // 无游戏版本/加载器信息，前端 install_merged 阶段需用户手动选择
            log_info!(
                "[Community] Compress 整合包: archive_base_folder={}",
                detected.archive_base_folder
            );
            Ok(ModpackInfo {
                format,
                game_version: String::new(),
                loader: String::new(),
                loader_version: String::new(),
                mod_files_count: 0,
                archive_base_folder: detected.archive_base_folder.clone(),
                cf_overrides_name: None,
                cf_manifest: None,
                mr_index: None,
                hmcl_manifest: None,
                mmc_pack: None,
                mmc_cfg_content: None,
                mcbbs_manifest: None,
                launcher_inner_path: None,
            })
        }
    }
}

/// 从 ModpackInfo 提取可选 Mod 列表（CF required=false / MR env.client=optional）
///
/// 用于前端 preview 后弹窗显示，让用户选择是否下载可选 Mod。
/// - CF: manifest.files 中 required=false 的项，display_name = "CF File #{file_id}"
///   （manifest 不含 displayName，需调用 /mods/files API 才能拿到，preview 阶段不调用 API）
/// - MR: index.files 中 env.client="optional" 的项，display_name = path 末段
/// - HMCL/MMC/MCBBS: 返回空列表（mods 已打包在 overrides 中，无可选概念）
pub(super) fn extract_optional_mods(info: &ModpackInfo) -> Vec<super::types::OptionalModInfo> {
    use super::types::{ModpackFormat, OptionalModInfo};

    match info.format {
        ModpackFormat::Curseforge => {
            let manifest = match &info.cf_manifest {
                Some(m) => m,
                None => return Vec::new(),
            };
            manifest
                .files
                .iter()
                .filter(|f| !f.required)
                .map(|f| OptionalModInfo {
                    display_name: format!(
                        "CF File #{}",
                        f.file_id.map_or("?".to_string(), |id| id.to_string())
                    ),
                    file_size: 0, // CF manifest 不含文件大小
                    file_id: f.file_id,
                    project_id: f.project_id,
                    path: None,
                })
                .collect()
        }
        ModpackFormat::Modrinth => {
            let index = match &info.mr_index {
                Some(i) => i,
                None => return Vec::new(),
            };
            index
                .files
                .iter()
                .filter(|f| f.env.client.as_deref() == Some("optional"))
                .map(|f| {
                    // path 末段作为 display_name
                    let display_name = f
                        .path
                        .rsplit(['/', '\\'])
                        .next()
                        .filter(|s| !s.is_empty())
                        .unwrap_or(&f.path)
                        .to_string();
                    OptionalModInfo {
                        display_name,
                        file_size: f.file_size,
                        file_id: None,
                        project_id: None,
                        path: Some(f.path.clone()),
                    }
                })
                .collect()
        }
        // HMCL/MMC/MCBBS/LauncherPack/Compress 整合包 mods 已打包在 overrides 中（或需递归处理），无可选概念
        ModpackFormat::Hmcl
        | ModpackFormat::Mmc
        | ModpackFormat::Mcbbs
        | ModpackFormat::LauncherPack
        | ModpackFormat::Compress => Vec::new(),
    }
}

/// 将 MMC instance.cfg / MCBBS launchInfo 中的配置迁移到版本 setup.ini
///
/// 必须在 `extract_overrides` 之后调用：MMC iconKey 复制需要 overrides 已解压到 instance 目录。
///
/// 字段映射：
/// - MMC `PreLaunchCommand`（OverrideCommands=true）→ `advance_run_cmd`（做变量替换）
/// - MMC `JoinServerOnLaunchAddress`（JoinServerOnLaunch=true）→ `server_enter`
/// - MMC `IgnoreJavaCompatibility=true` → `advance_ignore_java_warning=true`
/// - MMC `iconKey`（且 {iconKey}.png 存在）→ `logo`（复制到 MoLaunch/Logo.png）
/// - MMC `JvmArgs` → `advance_jvm_args`（OverrideJavaArgs=true 覆盖；false 也覆盖，简化不读全局）
/// - MCBBS `launchInfo.javaArgument` → `advance_jvm_args`（空格连接）
/// - MCBBS `launchInfo.launchArgument` → `advance_game_args`（空格连接）
///
/// 额外：所有格式都强制写入 `indie_type=1`，强制开启版本隔离，保证整合包独立性。
pub(super) fn migrate_modpack_config(
    info: &ModpackInfo,
    instance_dir: &std::path::Path,
    instance_name: &str,
) -> Result<(), String> {
    use crate::minecraft::version::setup::{PersonalizationUpdate, VersionSetup};

    let mut update = PersonalizationUpdate::default();

    // 强制开启版本隔离（所有格式都写）
    update.indie_type = Some(1);

    match info.format {
        ModpackFormat::Mmc => {
            let Some(cfg_content) = &info.mmc_cfg_content else {
                // 仍然写入 indie_type=1
                VersionSetup::update_personalization(instance_dir, &update)
                    .map_err(|e| format!("写入版本 setup.ini 失败: {}", e))?;
                return Ok(());
            };
            let cfg = super::mmc::parse_instance_cfg(cfg_content);

            // PreLaunchCommand（仅 OverrideCommands=true 时迁移）
            if cfg.override_commands {
                if let Some(cmd) = &cfg.pre_launch_command {
                    let replaced =
                        super::mmc::substitute_pre_launch_vars(cmd, instance_dir, instance_name);
                    log_info!("[Community] MMC 迁移 PreLaunchCommand: {}", replaced);
                    update.advance_run_cmd = Some(replaced);
                }
            }

            // JoinServerOnLaunchAddress（仅 JoinServerOnLaunch=true 时迁移）
            if cfg.join_server_on_launch {
                if let Some(addr) = &cfg.join_server_address {
                    log_info!("[Community] MMC 迁移 JoinServer: {}", addr);
                    update.server_enter = Some(addr.clone());
                }
            }

            // IgnoreJavaCompatibility
            if cfg.ignore_java_compatibility {
                log_info!("[Community] MMC 迁移 IgnoreJavaCompatibility=true");
                update.advance_ignore_java_warning = Some(true);
            }

            // iconKey：复制 {iconKey}.png 到 MoLaunch/Logo.png
            if let Some(icon_key) = &cfg.icon_key {
                let src_png = instance_dir.join(format!("{}.png", icon_key));
                if src_png.exists() {
                    let logo_dir = instance_dir.join("MoLaunch");
                    std::fs::create_dir_all(&logo_dir)
                        .map_err(|e| format!("创建 MoLaunch 目录失败: {}", e))?;
                    let logo_path = logo_dir.join("Logo.png");
                    if std::fs::copy(&src_png, &logo_path).is_ok() {
                        log_info!(
                            "[Community] MMC 复制图标: {} → {}",
                            src_png.display(),
                            logo_path.display()
                        );
                        update.logo = Some("MoLaunch\\Logo.png".to_string());
                    }
                } else {
                    log_info!(
                        "[Community] MMC iconKey 指定的图标不存在: {}",
                        src_png.display()
                    );
                }
            }

            // JvmArgs（简化：无论 OverrideJavaArgs 都直接覆盖版本独立 JVM 参数，
            // 不读取全局 LaunchAdvanceJvm 做追加，避免对全局配置的耦合）
            if let Some(jvm_args) = &cfg.jvm_args {
                log_info!(
                    "[Community] MMC 迁移 JvmArgs (override={}): {}",
                    cfg.override_java_args,
                    jvm_args
                );
                update.advance_jvm_args = Some(jvm_args.clone());
            }
        }
        ModpackFormat::Mcbbs => {
            if let Some(manifest) = &info.mcbbs_manifest {
                if let Some(launch_info) = &manifest.launch_info {
                    if let Some(java_args) = &launch_info.java_argument {
                        if !java_args.is_empty() {
                            let joined = java_args.join(" ");
                            log_info!("[Community] MCBBS 迁移 javaArgument: {}", joined);
                            update.advance_jvm_args = Some(joined);
                        }
                    }
                    if let Some(launch_args) = &launch_info.launch_argument {
                        if !launch_args.is_empty() {
                            let joined = launch_args.join(" ");
                            log_info!("[Community] MCBBS 迁移 launchArgument: {}", joined);
                            update.advance_game_args = Some(joined);
                        }
                    }
                }
            }
        }
        _ => {}
    }

    VersionSetup::update_personalization(instance_dir, &update)
        .map_err(|e| format!("写入版本 setup.ini 失败: {}", e))?;
    log_info!(
        "[Community] 配置迁移完成: instance={} format={:?} (indie_type=1)",
        instance_name,
        info.format
    );

    Ok(())
}

/// 复制外部 Logo 文件到版本目录 `MoLaunch/Logo.png` 并写入 setup.ini
///
/// 用于 CurseForge / Modrinth 在线下载安装时，将平台缓存的整合包缩略图
/// 复制为版本图标。拖拽安装通常无外部 Logo，跳过此步。
///
/// 行为：
/// - `logo_path` 为 None 或文件不存在时直接返回 Ok（视为无外部 Logo）
/// - 创建 `{instance_dir}/MoLaunch/` 目录（若不存在）
/// - 复制源文件到 `{instance_dir}/MoLaunch/Logo.png`（强制重命名，不保留原扩展名）
/// - 更新 setup.ini 的 `logo` 字段为 `MoLaunch\Logo.png`，`is_star` 不变
///
/// 失败时返回 Err，由调用方决定是否中断安装（一般应允许继续，Logo 仅装饰）。
pub(super) fn copy_external_logo(
    logo_path: Option<&str>,
    instance_dir: &std::path::Path,
) -> Result<(), String> {
    use crate::minecraft::version::setup::{PersonalizationUpdate, VersionSetup};

    let Some(path) = logo_path else {
        return Ok(());
    };
    if path.trim().is_empty() {
        return Ok(());
    }
    let src = std::path::Path::new(path);
    if !src.exists() {
        log_info!("[Community] 外部 Logo 文件不存在，跳过: {}", path);
        return Ok(());
    }

    let logo_dir = instance_dir.join("MoLaunch");
    std::fs::create_dir_all(&logo_dir)
        .map_err(|e| format!("创建 MoLaunch 目录失败: {}", e))?;
    let dest = logo_dir.join("Logo.png");

    std::fs::copy(src, &dest)
        .map_err(|_e| format!("复制 Logo 失败: {} → {}", src.display(), dest.display()))?;

    let update = PersonalizationUpdate {
        logo: Some("MoLaunch\\Logo.png".to_string()),
        ..Default::default()
    };
    VersionSetup::update_personalization(instance_dir, &update)
        .map_err(|e| format!("写入 Logo 路径到 setup.ini 失败: {}", e))?;

    log_info!(
        "[Community] 外部 Logo 已复制: {} → {}",
        src.display(),
        dest.display()
    );
    Ok(())
}
