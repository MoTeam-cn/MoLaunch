//! 整合包安装命令（install_modpack / install_local_modpack）

use crate::log_info;
use crate::minecraft::community::secure_storage;
use crate::minecraft::community::types::Platform;
use crate::minecraft::version::modpack_meta::ModpackMetaFile;
use crate::state::{AppState, DownloadStage, StageStatus};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;

use super::concurrent;
use super::modpack_stages::{
    copy_external_logo, download_modpack_archive, extract_optional_mods, parse_modpack_info,
};
use super::types::{
    InstallLocalModpackRequest, InstallModpackRequest, InstallModpackResult, ModpackFormat,
    ModpackPreview,
};

/// 当前正在安装的整合包实例名集合（用于重复任务检查）
///
/// 防止同名整合包同时安装。InstallGuard 在 install_modpack / install_local_modpack
/// 入口 acquire，函数返回时（成功或失败）通过 Drop 自动释放，无需手动清理。
static INSTALLING_INSTANCES: Lazy<Mutex<HashSet<String>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

/// 整合包安装占用 guard：构造时插入实例名，Drop 时自动移除
///
/// 用法：`let _guard = InstallGuard::acquire(&req.instance_name)?;`
/// 函数返回时（成功 Err 或 Ok）自动 Drop，无需手动 release。
struct InstallGuard {
    name: String,
}

impl InstallGuard {
    fn acquire(name: &str) -> Result<Self, String> {
        let mut set = INSTALLING_INSTANCES.lock().unwrap();
        if set.contains(name) {
            return Err(format!(
                "整合包 \"{}\" 正在安装中，请等待当前安装完成或取消后再试",
                name
            ));
        }
        set.insert(name.to_string());
        Ok(InstallGuard {
            name: name.to_string(),
        })
    }
}

impl Drop for InstallGuard {
    fn drop(&mut self) {
        INSTALLING_INSTANCES.lock().unwrap().remove(&self.name);
    }
}

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
pub async fn install_modpack(
    state: &AppState,
    req: InstallModpackRequest,
) -> Result<InstallModpackResult, String> {
    log_info!(
        "[Community] 开始安装整合包: platform={} instance={} url={}",
        req.platform.as_str(),
        req.instance_name,
        req.download_url
    );

    // 0. 实例名校验（入口拦截，避免后续创建目录失败或 Java 启动失败）
    super::helpers::validate_instance_name(&req.instance_name)?;
    super::helpers::validate_modpack_extension(&req.file_name)?;

    // 0.1 重复任务检查（同实例名正在安装时拒绝，防止并发安装冲突）
    let _guard = InstallGuard::acquire(&req.instance_name)?;

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

    // 解析游戏目录、创建 instance_dir（提到 async block 外，便于错误时清理版本目录）
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let max_threads = state.config.lock().await.download.max_threads.max(1) as usize;
    let instance_dir = game_dir.join("versions").join(&req.instance_name);
    std::fs::create_dir_all(&instance_dir)
        .map_err(|e| format!("创建整合包目录失败: {}", e))?;

    // 核心逻辑包在 async block 中，便于统一错误处理（失败时 mark_failed 重置 is_active）
    let instance_dir_ref = &instance_dir;
    let result: Result<InstallModpackResult, String> = async {
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
        let archive_path = instance_dir_ref.join(&req.file_name);
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
        let mods_dir = instance_dir_ref.join("mods");
        std::fs::create_dir_all(&mods_dir).map_err(|e| format!("创建 mods 目录失败: {}", e))?;

        // include_optional：在线资源页安装默认 true（不弹窗），
        // 拖拽安装由前端 preview 后弹窗询问用户传入。
        let include_optional = req.include_optional.unwrap_or(true);

        match info.format {
            ModpackFormat::Curseforge => {
                let manifest = info.cf_manifest.as_ref().expect("CF manifest 应已解析");
                super::curseforge::install_cf_mods(
                    &state,
                    &manifest.files,
                    &mods_dir,
                    max_threads,
                    instance_dir_ref,
                    2,
                    include_optional,
                )
                .await?;
            }
            ModpackFormat::Modrinth => {
                let index = info.mr_index.as_ref().expect("MR index 应已解析");
                super::modrinth::install_mr_files(
                    &state,
                    &index.files,
                    instance_dir_ref,
                    max_threads,
                    2,
                    include_optional,
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
            // LauncherPack 在线下载场景不应出现（外层 zip 包含启动器+整合包，非平台分发格式）
            ModpackFormat::LauncherPack => {
                return Err(
                    "在线下载的整合包不应为 LauncherPack 格式（带启动器整合包），请改用拖拽安装"
                        .to_string(),
                );
            }
            // Compress 普通压缩包：无依赖 mods 列表，全部内容在 .minecraft/ 目录下
            ModpackFormat::Compress => {
                log_info!("[Community] Compress 整合包无依赖 mods 列表，跳过 Stage 2");
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
        let prefixes = concurrent::build_overrides_prefixes(
            info.format,
            &info.archive_base_folder,
            info.cf_overrides_name.as_deref(),
        );
        concurrent::extract_overrides(&mut archive, instance_dir_ref, &state, &prefixes, 3)?;
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(3, StageStatus::Finished, 1.0);
            // 不调用 mark_complete()：前端会紧接着调用 install_merged 安装 MC 本体，
            // 轮询必须继续。mark_complete 由 install_merged 在全部完成后调用。
        }

        // 迁移 MMC instance.cfg / MCBBS launchInfo 配置到版本 setup.ini
        // 必须在 extract_overrides 之后：MMC iconKey 复制需要 overrides 已解压
        super::modpack_stages::migrate_modpack_config(&info, instance_dir_ref, &req.instance_name)?;

        // CF/MR 在线下载安装时复制外部 Logo（拖拽安装 logo_path 通常为 None，会自动跳过）
        if let Err(e) = copy_external_logo(req.logo_path.as_deref(), instance_dir_ref) {
            crate::log_warn!("[Community] 复制外部 Logo 失败（不中断安装）: {}", e);
        }

        // 联机大厅阶段 3：写入 modpack.meta.json（仅在线安装且有平台来源信息时）
        // 拖拽安装（install_local_modpack）无 project_id/file_id，跳过写入。
        // 写入失败不中断安装流程（整合包已安装成功，仅影响联机大厅上报）。
        if let (Some(project_id), Some(file_id)) = (&req.project_id, &req.file_id) {
            let source = match req.platform {
                Platform::CurseForge => "curseforge",
                Platform::Modrinth => "modrinth",
            };
            let meta = ModpackMetaFile {
                source: source.to_string(),
                project_id: project_id.clone(),
                file_id: file_id.clone(),
                mc_version: info.game_version.clone(),
                modpack_version: req.modpack_version.clone(),
                name: req.name.clone().unwrap_or_else(|| req.instance_name.clone()),
                loader: if info.loader.is_empty() {
                    None
                } else {
                    Some(info.loader.clone())
                },
                loader_version: if info.loader_version.is_empty() {
                    None
                } else {
                    Some(info.loader_version.clone())
                },
                file_size: req.file_size,
                file_count: Some(info.mod_files_count as u32),
                // manifest_hash 暂未实现：需在 parse_modpack_info 中计算 manifest 原始内容 SHA-256
                // 阶段 4 加入方校验本地已装时再补充
                manifest_hash: None,
                installed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            };
            if let Err(e) = meta.save(instance_dir_ref) {
                crate::log_warn!(
                    "[Community] 写入 modpack.meta.json 失败（不中断安装）: {}",
                    e
                );
            } else {
                log_info!(
                    "[Community] modpack.meta.json 已写入: {}:{} ({})",
                    source,
                    project_id,
                    file_id
                );
            }
        }

        log_info!("[Community] 整合包安装完成: {}", req.instance_name);

        Ok(InstallModpackResult {
            format: info.format,
            game_version: info.game_version,
            loader: info.loader,
            loader_version: info.loader_version,
            archive_path: archive_path.to_string_lossy().to_string(),
            instance_dir: instance_dir_ref.to_string_lossy().to_string(),
        })
    }
    .await;

    // 错误时重置 download_state + 清理版本目录（带 saves/versions 保护）
    if let Err(e) = result {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(0);
        super::helpers::cleanup_version_dir_on_failure(&instance_dir);
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
pub async fn install_local_modpack(
    state: &AppState,
    req: InstallLocalModpackRequest,
) -> Result<InstallModpackResult, String> {
    log_info!(
        "[Community] 开始安装本地整合包: path={} instance={}",
        req.file_path,
        req.instance_name
    );

    // 0. 实例名校验（入口拦截，避免后续创建目录失败或 Java 启动失败）
    super::helpers::validate_instance_name(&req.instance_name)?;
    super::helpers::validate_modpack_extension(&req.file_path)?;

    // 0.1 重复任务检查（同实例名正在安装时拒绝，防止并发安装冲突）
    let _guard = InstallGuard::acquire(&req.instance_name)?;

    // 1. 校验文件存在（在 reset_stages 之前，失败时不需要 mark_failed）
    let archive_path = std::path::PathBuf::from(&req.file_path);
    if !archive_path.exists() {
        return Err(format!("整合包文件不存在: {}", req.file_path));
    }

    // 1.1 预检测：如果是 LauncherPack（带启动器整合包），先提取内层整合包到临时目录，
    // 然后将 archive_path 替换为内层整合包路径继续走主流程。
    // 避免递归调用 install_local_modpack（async fn 递归需要 boxing，且会重置 download_state）。
    let archive_path_owned: std::path::PathBuf;
    let _temp_cleanup: Option<std::path::PathBuf>; // 携带临时文件路径用于函数结束时清理
    {
        let pre_file = std::fs::File::open(&archive_path)
            .map_err(|e| format!("打开整合包失败: {}", e))?;
        let mut pre_archive = zip::ZipArchive::new(pre_file)
            .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;
        let pre_detected = concurrent::detect_modpack_format(&mut pre_archive)?;
        if pre_detected.format == ModpackFormat::LauncherPack {
            let inner_path = pre_detected.launcher_inner_path.as_deref().ok_or_else(|| {
                "LauncherPack 检测异常：未记录内层整合包路径".to_string()
            })?;
            log_info!(
                "[Community] LauncherPack：提取内层整合包 {} 到临时目录后继续安装",
                inner_path
            );

            // 提取内层整合包到 instance_dir 同级的临时目录
            let game_dir_pre = crate::state::resolve_game_dir_from_state(state).await;
            let temp_dir = game_dir_pre.join(".tmp_launcher_extract");
            std::fs::create_dir_all(&temp_dir)
                .map_err(|e| format!("创建临时目录失败: {}", e))?;
            let inner_file_name = inner_path
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or("modpack.zip");
            let inner_local_path = temp_dir.join(inner_file_name);
            let mut inner_entry = pre_archive
                .by_name(inner_path)
                .map_err(|e| format!("读取内层整合包失败: {} ({})", inner_path, e))?;
            use std::io::Read;
            let mut buf = Vec::new();
            inner_entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("读取内层整合包内容失败: {}", e))?;
            std::fs::write(&inner_local_path, &buf)
                .map_err(|e| format!("写入内层整合包失败: {}", e))?;
            log_info!(
                "[Community] LauncherPack：内层整合包已提取到 {}",
                inner_local_path.display()
            );

            archive_path_owned = inner_local_path.clone();
            _temp_cleanup = Some(inner_local_path.clone());
            // 临时目录留待函数结束时清理（temp_dir 路径已固定，下面手动清理）
            let _ = temp_dir; // 不在此处删除，避免内层整合包还在使用
        } else {
            archive_path_owned = archive_path.clone();
            _temp_cleanup = None;
        }
    }
    let archive_path = &archive_path_owned;

    // 解析游戏目录、创建 instance_dir（提到 async block 外，便于错误时清理版本目录）
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let max_threads = state.config.lock().await.download.max_threads.max(1) as usize;
    let instance_dir = game_dir.join("versions").join(&req.instance_name);
    std::fs::create_dir_all(&instance_dir)
        .map_err(|e| format!("创建整合包目录失败: {}", e))?;

    // 核心逻辑包在 async block 中，便于统一错误处理（失败时 mark_failed 重置 is_active）
    let instance_dir_ref = &instance_dir;
    let result: Result<InstallModpackResult, String> = async {
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
        let mods_dir = instance_dir_ref.join("mods");
        std::fs::create_dir_all(&mods_dir).map_err(|e| format!("创建 mods 目录失败: {}", e))?;

        // include_optional：由前端 preview 后弹窗询问用户传入，None 默认 true（保持向后兼容）
        let include_optional = req.include_optional.unwrap_or(true);

        match info.format {
            ModpackFormat::Curseforge => {
                let manifest = info.cf_manifest.as_ref().expect("CF manifest 应已解析");
                super::curseforge::install_cf_mods(
                    &state,
                    &manifest.files,
                    &mods_dir,
                    max_threads,
                    instance_dir_ref,
                    1,
                    include_optional,
                )
                .await?;
            }
            ModpackFormat::Modrinth => {
                let index = info.mr_index.as_ref().expect("MR index 应已解析");
                super::modrinth::install_mr_files(
                    &state,
                    &index.files,
                    instance_dir_ref,
                    max_threads,
                    1,
                    include_optional,
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
            // LauncherPack 不会走到这里：入口预检测已递归处理
            ModpackFormat::LauncherPack => {
                return Err(
                    "LauncherPack 不应进入主安装流程（应在入口预检测阶段递归处理）".to_string(),
                );
            }
            // Compress 普通压缩包：无依赖 mods 列表，全部内容在 .minecraft/ 目录下
            ModpackFormat::Compress => {
                log_info!("[Community] Compress 本地整合包无依赖 mods 列表，跳过 Stage 1");
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
        let prefixes = concurrent::build_overrides_prefixes(
            info.format,
            &info.archive_base_folder,
            info.cf_overrides_name.as_deref(),
        );
        concurrent::extract_overrides(&mut archive, instance_dir_ref, &state, &prefixes, 2)?;
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(2, StageStatus::Finished, 1.0);
            // 不调用 mark_complete()：前端会紧接着调用 install_merged 安装 MC 本体，
            // 轮询必须继续。mark_complete 由 install_merged 在全部完成后调用。
        }

        // 迁移 MMC instance.cfg / MCBBS launchInfo 配置到版本 setup.ini
        // 必须在 extract_overrides 之后：MMC iconKey 复制需要 overrides 已解压
        super::modpack_stages::migrate_modpack_config(&info, instance_dir_ref, &req.instance_name)?;

        // CF/MR 在线下载安装时复制外部 Logo（拖拽安装 logo_path 通常为 None，会自动跳过）
        if let Err(e) = copy_external_logo(req.logo_path.as_deref(), instance_dir_ref) {
            crate::log_warn!("[Community] 复制外部 Logo 失败（不中断安装）: {}", e);
        }

        log_info!("[Community] 本地整合包安装完成: {}", req.instance_name);

        Ok(InstallModpackResult {
            format: info.format,
            game_version: info.game_version,
            loader: info.loader,
            loader_version: info.loader_version,
            archive_path: archive_path.to_string_lossy().to_string(),
            instance_dir: instance_dir_ref.to_string_lossy().to_string(),
        })
    }
    .await;

    // 错误时重置 download_state + 清理版本目录（带 saves/versions 保护）
    if let Err(e) = result {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(0);
        super::helpers::cleanup_version_dir_on_failure(&instance_dir);
        // LauncherPack 临时文件清理
        if let Some(tmp) = &_temp_cleanup {
            let _ = std::fs::remove_file(tmp);
            let _ = std::fs::remove_dir(tmp.parent().unwrap_or(std::path::Path::new(".")));
        }
        return Err(e);
    }
    // LauncherPack 临时文件清理（成功路径）
    if let Some(tmp) = &_temp_cleanup {
        let _ = std::fs::remove_file(tmp);
        let _ = std::fs::remove_dir(tmp.parent().unwrap_or(std::path::Path::new(".")));
    }
    result
}

/// 预览本地整合包（拖拽安装前置步骤）
///
/// 仅打开 zip + 检测格式 + 解析 manifest/index，不下载、不复制 overrides。
/// 返回整合包基本信息 + 可选 Mod 列表，前端据弹窗询问用户是否下载可选 Mod。
/// 用户选择后调用 `install_local_modpack` 传入 `include_optional` 参数完成安装。
///
/// RPC 模型需要拆分为 preview + install 两步，preview 阶段不阻塞、不调用 API。
pub async fn preview_local_modpack(file_path: String) -> Result<ModpackPreview, String> {
    log_info!("[Community] 预览本地整合包: {}", file_path);

    super::helpers::validate_modpack_extension(&file_path)?;

    let archive_path = std::path::PathBuf::from(&file_path);
    if !archive_path.exists() {
        return Err(format!("整合包文件不存在: {}", file_path));
    }

    let file =
        std::fs::File::open(&archive_path).map_err(|e| format!("打开整合包失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;

    let detected = concurrent::detect_modpack_format(&mut archive)?;
    let info = parse_modpack_info(&detected)?;

    let optional_mods = extract_optional_mods(&info);

    log_info!(
        "[Community] 预览完成: format={:?} game={} loader={}{} optional_mods={}",
        info.format,
        info.game_version,
        info.loader,
        if info.loader_version.is_empty() {
            String::new()
        } else {
            format!("@{}", info.loader_version)
        },
        optional_mods.len()
    );

    Ok(ModpackPreview {
        format: info.format,
        game_version: info.game_version,
        loader: info.loader,
        loader_version: info.loader_version,
        optional_mods,
    })
}
