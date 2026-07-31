//! 整合包本地拖拽安装（install_local_modpack）

use crate::log_info;
use crate::state::{AppState, DownloadStage, StageStatus};

use super::super::concurrent;
use super::super::modpack_stages::parse_modpack_info;
use super::super::types::{InstallLocalModpackRequest, InstallModpackResult, ModpackFormat};
use super::shared;

/// 安装本地整合包（拖拽安装）
///
/// 与 install_modpack 的差异：跳过 Stage 0 下载，直接使用本地文件路径。
/// 共享 Stage 1-3 流程：解析 manifest → 下载依赖 mods → 复制 overrides。
/// LauncherPack（带启动器整合包）会先提取内层整合包到临时目录再继续主流程。
pub async fn install_local_modpack(
    state: &AppState,
    req: InstallLocalModpackRequest,
) -> Result<InstallModpackResult, String> {
    log_info!(
        "[Community] 开始安装本地整合包: path={} instance={}",
        req.file_path,
        req.instance_name
    );

    // 0. 实例名校验 + 重复任务检查
    super::super::helpers::validate_instance_name(&req.instance_name)?;
    super::super::helpers::validate_modpack_extension(&req.file_path)?;
    let _guard = super::InstallGuard::acquire(&req.instance_name)?;

    // 1. 校验文件存在
    let archive_path = std::path::PathBuf::from(&req.file_path);
    if !archive_path.exists() {
        return Err(format!("整合包文件不存在: {}", req.file_path));
    }

    // 1.1 预检测：LauncherPack 先提取内层整合包到临时目录，避免递归调用本函数
    let archive_path_owned: std::path::PathBuf;
    let _temp_cleanup: Option<std::path::PathBuf>;
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
        } else {
            archive_path_owned = archive_path.clone();
            _temp_cleanup = None;
        }
    }
    let archive_path = &archive_path_owned;

    // 解析游戏目录、创建 instance_dir
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let instance_dir = game_dir.join("versions").join(&req.instance_name);
    std::fs::create_dir_all(&instance_dir)
        .map_err(|e| format!("创建整合包目录失败: {}", e))?;

    let instance_dir_ref = &instance_dir;
    let result: Result<InstallModpackResult, String> = async {
        // 2. 重置 download_state（本地拖拽跳过 Stage 0 下载，保留 3 个 stages）
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

        // 3. Stage 0：打开 zip + 检测格式 + 解析 manifest
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(0, StageStatus::Loading, 0.0);
        }
        let parse_ticker = crate::commands::version::install::loader_helpers::start_parse_ticker(
            state, 0,
        );
        let file =
            std::fs::File::open(&archive_path).map_err(|e| format!("打开整合包失败: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;
        let detected = concurrent::detect_modpack_format(&mut archive)?;

        // CF 格式需要 API Key（source=0 强制镜像时跳过）
        if detected.format == ModpackFormat::Curseforge {
            if let Err(e) = shared::validate_cf_api_key().await {
                parse_ticker.store(true, std::sync::atomic::Ordering::Relaxed);
                return Err(e);
            }
        }

        let info = parse_modpack_info(&detected)?;
        parse_ticker.store(true, std::sync::atomic::Ordering::Relaxed);
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

        // 4. Stage 1：下载依赖文件（仅 CF/MR 有依赖 mods 列表）
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(1, StageStatus::Loading, 0.0);
        }
        let mods_dir = instance_dir_ref.join("mods");
        std::fs::create_dir_all(&mods_dir).map_err(|e| format!("创建 mods 目录失败: {}", e))?;
        let include_optional = req.include_optional.unwrap_or(true);
        shared::download_mods_by_format(
            state, &info, &mods_dir, instance_dir_ref, 1, include_optional, true,
        )
        .await?;
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(1, StageStatus::Finished, 1.0);
        }

        // 5. Stage 2：复制 overrides + 迁移配置 + 复制 Logo
        shared::finalize_overrides_and_config(
            &mut archive,
            &info,
            instance_dir_ref,
            state,
            2,
            &req.instance_name,
            req.logo_path.as_deref(),
        )?;

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

    // 错误时重置 download_state + 清理版本目录 + LauncherPack 临时文件
    if let Err(e) = result {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(0);
        super::super::helpers::cleanup_version_dir_on_failure(&instance_dir);
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
