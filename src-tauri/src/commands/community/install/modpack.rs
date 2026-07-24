//! 整合包安装命令（install_modpack / install_local_modpack）

use crate::log_info;
use crate::minecraft::community::secure_storage;
use crate::minecraft::community::types::Platform;
use crate::state::{AppState, DownloadStage, StageStatus};
use tauri::State;

use super::concurrent;
use super::modpack_stages::{download_modpack_archive, parse_modpack_info};
use super::types::{InstallLocalModpackRequest, InstallModpackRequest, InstallModpackResult, ModpackFormat};

/// 安装整合包
///
/// 完整流程：
/// 1. CF 平台前置检查 API Key（未启用或为空立即报错）
/// 2. 下载原始整合包到 versions/{instance}/（委托 modpack_stages::download_modpack_archive）
/// 3. 检测格式 + 解析 manifest/modrinth.index.json（委托 modpack_stages::parse_modpack_info）
/// 4. 下载依赖文件（CF: install_cf_mods / MR: install_mr_files）
/// 5. 解压 overrides 到 instance 目录（concurrent::extract_overrides）
///
/// 进度通过 `state.download_state` 推送（与版本下载共用 DownloadPanel 展示）。
/// 完成后前端调用 `install_merged` 安装游戏本体。
///
/// 错误处理：任何阶段失败都会调用 `mark_failed(0)` 重置 `is_active=false`，
/// 避免前端下载管理页卡在 0% 进度（前端轮询 `is_downloading` 会返回 false，
/// Downloads.vue 的 watch 会自动 `router.back()`）。
#[tauri::command]
pub async fn install_modpack(
    state: State<'_, AppState>,
    req: InstallModpackRequest,
) -> Result<InstallModpackResult, String> {
    log_info!(
        "[Community] 开始安装整合包: platform={} instance={} url={}",
        req.platform.as_str(),
        req.instance_name,
        req.download_url
    );

    // 1. CF 平台前置检查 API Key（在 reset_stages 之前，失败时不需要 mark_failed）
    //    source=0 强制镜像时跳过：镜像站（mod.mcimirror.top）自带 API Key 请求 CF，
    //    用户无需配置自己的 Key 即可使用需要 Key 的接口（如 /mods/files）。
    if req.platform == Platform::CurseForge {
        let source = crate::minecraft::community::get_source_pref();
        if source != 0 {
            let (enabled, api_key) = secure_storage::get_config_async().await;
            if !enabled {
                return Err(
                    "CurseForge 整合包安装需要 API Key。请在「设置 → 社区资源」中启用 CurseForge 官方源并填写 API Key，或将下载源切换为「尽量镜像」使用镜像站。"
                        .to_string(),
                );
            }
            let key_empty = api_key.as_deref().map_or(true, |k| k.is_empty());
            if key_empty {
                return Err(
                    "CurseForge 整合包安装需要 API Key。已在设置中启用但未填写 API Key，请补全后重试，或将下载源切换为「尽量镜像」使用镜像站。"
                        .to_string(),
                );
            }
            log_info!("[Community] CF API Key 检查通过 (source={})", source);
        } else {
            log_info!("[Community] CF source=0 强制镜像，跳过 API Key 检查（镜像站自带 Key）");
        }
    }

    // 核心逻辑包在 async block 中，便于统一错误处理（失败时 mark_failed 重置 is_active）
    let result: Result<InstallModpackResult, String> = async {
        // 解析游戏目录
        let game_dir = crate::state::resolve_game_dir_from_state(&state).await;
        let max_threads = state.config.lock().await.download.max_threads.max(1) as usize;

        let instance_dir = game_dir.join("versions").join(&req.instance_name);
        std::fs::create_dir_all(&instance_dir)
            .map_err(|e| format!("创建整合包目录失败: {}", e))?;

        // 2. 重置 download_state，设置整合包专用 stages（统一方法）
        // 同时重置暂停/取消标志，防止上次残留导致新下载卡住
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.reset_stages(vec![
                DownloadStage::new_grouped("下载整合包", 10.0, "整合包安装"),
                DownloadStage::new_grouped("解析整合包", 1.0, "整合包安装"),
                DownloadStage::new_grouped("下载 MOD", 40.0, "整合包安装"),
                DownloadStage::new_grouped("复制配置文件", 5.0, "整合包安装"),
            ]);
            ds.version_name = req.instance_name.clone();
        }
        state
            .download_cancel_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);
        state
            .download_pause_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // 3. Stage 0：下载原始整合包（委托 modpack_stages）
        let archive_path = instance_dir.join(&req.file_name);
        download_modpack_archive(&state, &archive_path, &req.download_url, &req.file_name).await?;

        // 4. Stage 1：打开 zip + 检测格式 + 解析 manifest/index（委托 modpack_stages::parse_modpack_info）
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(1, StageStatus::Loading, 0.0);
        }
        let file =
            std::fs::File::open(&archive_path).map_err(|e| format!("打开整合包失败: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;

        let detected = concurrent::detect_modpack_format(&mut archive)?;
        let info = parse_modpack_info(&detected)?;

        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(1, StageStatus::Finished, 1.0);
        }
        log_info!(
            "[Community] 整合包格式={:?} game={} loader={}{} mods={}",
            info.format,
            info.game_version,
            info.loader,
            if info.loader_version.is_empty() {
                String::new()
            } else {
                format!("@{}", info.loader_version)
            },
            info.mod_files_count
        );

        // 5. Stage 2：下载依赖文件（仅 CF/MR 有依赖 mods 列表，HMCL/MMC/MCBBS 已打包在 overrides 中）
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(2, StageStatus::Loading, 0.0);
        }
        let mods_dir = instance_dir.join("mods");
        std::fs::create_dir_all(&mods_dir).map_err(|e| format!("创建 mods 目录失败: {}", e))?;

        match info.format {
            ModpackFormat::Curseforge => {
                let manifest = info.cf_manifest.expect("CF manifest 应已解析");
                super::curseforge::install_cf_mods(
                    &state,
                    &manifest.files,
                    &mods_dir,
                    max_threads,
                    &instance_dir,
                    2,
                )
                .await?;
            }
            ModpackFormat::Modrinth => {
                let index = info.mr_index.expect("MR index 应已解析");
                super::modrinth::install_mr_files(
                    &state,
                    &index.files,
                    &instance_dir,
                    max_threads,
                    2,
                )
                .await?;
            }
            // HMCL/MMC/MCBBS 整合包无依赖 mods 列表，mods 已打包在 overrides 中
            ModpackFormat::Hmcl | ModpackFormat::Mmc | ModpackFormat::Mcbbs => {
                log_info!(
                    "[Community] {:?} 整合包无依赖 mods 列表，跳过 Stage 2",
                    info.format
                );
            }
        }
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(2, StageStatus::Finished, 1.0);
        }

        // 6. Stage 3：复制 overrides
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(3, StageStatus::Loading, 0.0);
        }
        let prefixes =
            concurrent::build_overrides_prefixes(info.format, &info.archive_base_folder);
        concurrent::extract_overrides(&mut archive, &instance_dir, &state, &prefixes, 3)?;
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(3, StageStatus::Finished, 1.0);
            // 不调用 mark_complete()：前端会紧接着调用 install_merged 安装 MC 本体，
            // 轮询必须继续。mark_complete 由 install_merged 在全部完成后调用。
        }

        log_info!("[Community] 整合包安装完成: {}", req.instance_name);

        Ok(InstallModpackResult {
            format: info.format,
            game_version: info.game_version,
            loader: info.loader,
            loader_version: info.loader_version,
            archive_path: archive_path.to_string_lossy().to_string(),
            instance_dir: instance_dir.to_string_lossy().to_string(),
        })
    }
    .await;

    // 错误时重置 download_state，避免 is_active 仍为 true 导致前端下载管理页卡住
    if let Err(e) = result {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(0);
        return Err(e);
    }
    result
}

/// 安装本地整合包（拖拽安装）
///
/// 与 `install_modpack` 的差异：跳过 Stage 0 下载，直接使用本地文件路径。
/// 共享 Stage 1-3 流程：解析 manifest → 下载依赖 mods → 复制 overrides。
///
/// 进度通过 `state.download_state` 推送（与版本下载共用 DownloadPanel 展示）。
/// 完成后前端调用 `install_merged` 安装游戏本体。
///
/// 错误处理：任何阶段失败都会调用 `mark_failed(0)` 重置 `is_active=false`，
/// 避免前端下载管理页卡在 0% 进度（前端轮询 `is_downloading` 会返回 false，
/// Downloads.vue 的 watch 会自动 `router.back()`）。
#[tauri::command]
pub async fn install_local_modpack(
    state: State<'_, AppState>,
    req: InstallLocalModpackRequest,
) -> Result<InstallModpackResult, String> {
    log_info!(
        "[Community] 开始安装本地整合包: path={} instance={}",
        req.file_path,
        req.instance_name
    );

    // 1. 校验文件存在（在 reset_stages 之前，失败时不需要 mark_failed）
    let archive_path = std::path::PathBuf::from(&req.file_path);
    if !archive_path.exists() {
        return Err(format!("整合包文件不存在: {}", req.file_path));
    }

    // 核心逻辑包在 async block 中，便于统一错误处理（失败时 mark_failed 重置 is_active）
    let result: Result<InstallModpackResult, String> = async {
        // 2. 解析游戏目录
        let game_dir = crate::state::resolve_game_dir_from_state(&state).await;
        let max_threads = state.config.lock().await.download.max_threads.max(1) as usize;

        let instance_dir = game_dir.join("versions").join(&req.instance_name);
        std::fs::create_dir_all(&instance_dir)
            .map_err(|e| format!("创建整合包目录失败: {}", e))?;

        // 3. 重置 download_state（本地拖拽跳过 Stage 0 下载，保留 3 个 stages）
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.reset_stages(vec![
                DownloadStage::new_grouped("解析整合包", 1.0, "整合包安装"),
                DownloadStage::new_grouped("下载 MOD", 90.0, "整合包安装"),
                DownloadStage::new_grouped("复制配置文件", 9.0, "整合包安装"),
            ]);
            ds.version_name = req.instance_name.clone();
        }
        state
            .download_cancel_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);
        state
            .download_pause_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // 4. Stage 0：打开 zip + 检测格式 + 解析 manifest/index
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(0, StageStatus::Loading, 0.0);
        }
        let file =
            std::fs::File::open(&archive_path).map_err(|e| format!("打开整合包失败: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;

        let detected = concurrent::detect_modpack_format(&mut archive)?;

        // CF 格式需要 API Key（install_cf_mods 会用到）
        // source=0 强制镜像时跳过：镜像站自带 Key 请求 CF，用户无需配置
        if detected.format == ModpackFormat::Curseforge {
            let source = crate::minecraft::community::get_source_pref();
            if source != 0 {
                let (enabled, api_key) = secure_storage::get_config_async().await;
                if !enabled {
                    return Err(
                        "CurseForge 整合包安装需要 API Key。请在「设置 → 社区资源」中启用 CurseForge 官方源并填写 API Key，或将下载源切换为「尽量镜像」使用镜像站。"
                            .to_string(),
                    );
                }
                let key_empty = api_key.as_deref().map_or(true, |k| k.is_empty());
                if key_empty {
                    return Err(
                        "CurseForge 整合包安装需要 API Key。已在设置中启用但未填写 API Key，请补全后重试，或将下载源切换为「尽量镜像」使用镜像站。"
                            .to_string(),
                    );
                }
                log_info!("[Community] CF API Key 检查通过 (source={})", source);
            } else {
                log_info!("[Community] CF source=0 强制镜像，跳过 API Key 检查（镜像站自带 Key）");
            }
        }

        let info = parse_modpack_info(&detected)?;
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(0, StageStatus::Finished, 1.0);
        }
        log_info!(
            "[Community] 本地整合包格式={:?} game={} loader={}{} mods={}",
            info.format,
            info.game_version,
            info.loader,
            if info.loader_version.is_empty() {
                String::new()
            } else {
                format!("@{}", info.loader_version)
            },
            info.mod_files_count
        );

        // 5. Stage 1：下载依赖文件（仅 CF/MR 有依赖 mods 列表，HMCL/MMC/MCBBS 已打包在 overrides 中）
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(1, StageStatus::Loading, 0.0);
        }
        let mods_dir = instance_dir.join("mods");
        std::fs::create_dir_all(&mods_dir).map_err(|e| format!("创建 mods 目录失败: {}", e))?;

        match info.format {
            ModpackFormat::Curseforge => {
                let manifest = info.cf_manifest.expect("CF manifest 应已解析");
                super::curseforge::install_cf_mods(
                    &state,
                    &manifest.files,
                    &mods_dir,
                    max_threads,
                    &instance_dir,
                    1,
                )
                .await?;
            }
            ModpackFormat::Modrinth => {
                let index = info.mr_index.expect("MR index 应已解析");
                super::modrinth::install_mr_files(
                    &state,
                    &index.files,
                    &instance_dir,
                    max_threads,
                    1,
                )
                .await?;
            }
            // HMCL/MMC/MCBBS 整合包无依赖 mods 列表，mods 已打包在 overrides 中
            ModpackFormat::Hmcl | ModpackFormat::Mmc | ModpackFormat::Mcbbs => {
                log_info!(
                    "[Community] {:?} 本地整合包无依赖 mods 列表，跳过 Stage 1",
                    info.format
                );
            }
        }
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(1, StageStatus::Finished, 1.0);
        }

        // 6. Stage 2：复制 overrides
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(2, StageStatus::Loading, 0.0);
        }
        let prefixes =
            concurrent::build_overrides_prefixes(info.format, &info.archive_base_folder);
        concurrent::extract_overrides(&mut archive, &instance_dir, &state, &prefixes, 2)?;
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(2, StageStatus::Finished, 1.0);
            // 不调用 mark_complete()：前端会紧接着调用 install_merged 安装 MC 本体，
            // 轮询必须继续。mark_complete 由 install_merged 在全部完成后调用。
        }

        log_info!("[Community] 本地整合包安装完成: {}", req.instance_name);

        Ok(InstallModpackResult {
            format: info.format,
            game_version: info.game_version,
            loader: info.loader,
            loader_version: info.loader_version,
            archive_path: archive_path.to_string_lossy().to_string(),
            instance_dir: instance_dir.to_string_lossy().to_string(),
        })
    }
    .await;

    // 错误时重置 download_state，避免 is_active 仍为 true 导致前端下载管理页卡住
    if let Err(e) = result {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(0);
        return Err(e);
    }
    result
}
