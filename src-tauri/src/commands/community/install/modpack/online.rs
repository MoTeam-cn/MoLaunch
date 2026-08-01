//! 整合包在线安装（install_modpack）

use crate::log_info;
use crate::minecraft::community::types::Platform;
use crate::minecraft::version::modpack_meta::ModpackMetaFile;
use crate::state::{AppState, DownloadStage, StageStatus};

use super::super::concurrent;
use super::super::modpack_stages::{download_modpack_archive, parse_modpack_info};
use super::super::types::{InstallModpackRequest, InstallModpackResult};
use super::shared;

/// 安装整合包（在线下载）
///
/// 流程：CF Key 检查 → 下载原始包 → 解析 manifest → 下载依赖 mods → 复制 overrides。
/// 进度通过 state.download_state 推送，完成后前端调用 install_merged 安装游戏本体。
/// 失败时 mark_failed(0) 重置 is_active=false 并清理版本目录。
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

    // 0. 实例名校验 + 重复任务检查（入口拦截）
    super::super::helpers::validate_instance_name(&req.instance_name)?;
    super::super::helpers::validate_modpack_extension(&req.file_name)?;
    let _guard = super::InstallGuard::acquire(&req.instance_name)?;

    // 1. CF 平台前置检查 API Key（在 reset_stages 之前，失败时不需要 mark_failed）
    if req.platform == Platform::CurseForge {
        shared::validate_cf_api_key().await?;
    }

    // 解析游戏目录、创建 instance_dir（提到 async block 外，便于错误时清理版本目录）
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let instance_dir = game_dir.join("versions").join(&req.instance_name);
    std::fs::create_dir_all(&instance_dir).map_err(|e| format!("创建整合包目录失败: {}", e))?;

    let instance_dir_ref = &instance_dir;
    let result: Result<InstallModpackResult, String> = async {
        // 2. 重置 download_state，设置整合包专用 stages
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

        // 3. Stage 0：下载原始整合包
        let archive_path = instance_dir_ref.join(&req.file_name);
        download_modpack_archive(state, &archive_path, &req.download_url, &req.file_name).await?;

        // 4. Stage 1：打开 zip + 检测格式 + 解析 manifest
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(1, StageStatus::Loading, 0.0);
        }
        // 伪进度 ticker（0→90% @5%/s），解析完成后 stop 并跳 100%
        let parse_ticker =
            crate::commands::version::install::loader_helpers::start_parse_ticker(state, 1);
        let file =
            std::fs::File::open(&archive_path).map_err(|e| format!("打开整合包失败: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;
        let detected = concurrent::detect_modpack_format(&mut archive)?;
        let info = parse_modpack_info(&detected)?;
        parse_ticker.store(true, std::sync::atomic::Ordering::Relaxed);
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

        // 5. Stage 2：下载依赖文件（仅 CF/MR 有依赖 mods 列表）
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(2, StageStatus::Loading, 0.0);
        }
        let mods_dir = instance_dir_ref.join("mods");
        std::fs::create_dir_all(&mods_dir).map_err(|e| format!("创建 mods 目录失败: {}", e))?;
        let include_optional = req.include_optional.unwrap_or(true);
        shared::download_mods_by_format(
            state,
            &info,
            &mods_dir,
            instance_dir_ref,
            2,
            include_optional,
            false,
        )
        .await?;
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(2, StageStatus::Finished, 1.0);
        }

        // 6. Stage 3：复制 overrides + 迁移配置 + 复制 Logo
        shared::finalize_overrides_and_config(
            &mut archive,
            &info,
            instance_dir_ref,
            state,
            3,
            &req.instance_name,
            req.logo_path.as_deref(),
        )?;

        // 联机大厅阶段 3：写入 modpack.meta.json（仅在线安装且有平台来源信息时）
        // 拖拽安装无 project_id/file_id，跳过写入。失败不中断安装流程。
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
                name: req
                    .name
                    .clone()
                    .unwrap_or_else(|| req.instance_name.clone()),
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
        super::super::helpers::cleanup_version_dir_on_failure(&instance_dir);
        return Err(e);
    }
    result
}
