//! install_modpack 阶段辅助：下载原始包、解析 manifest、提取可选 Mod、复制 Logo
//!
//! 子模块：parsers（各格式解析）/ migrate（配置迁移）

mod migrate;
mod parsers;

use crate::log_info;
use crate::minecraft::download::DownloadSession;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::state::{AppState, StageStatus};
use crate::utils::format;

use super::concurrent::DetectedModpack;
use super::types::{ModpackFormat, ModpackInfo, OptionalModInfo};

pub(super) use migrate::migrate_modpack_config;

/// Stage 0：下载原始整合包到 instance 目录
///
/// 通过 DownloadSession::attach 复用 install_modpack 父会话的 stages / flag 状态，
/// 仅构造 manager + callback。进度通过 `sync_stage_from_progress` 同步到 Stage 0。
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
        expected_size: 0,
        expected_hash: None,
    };

    let session = DownloadSession::attach(state).await;
    let stage0_callback = session.make_progress_callback(state, 0);
    let archive_results = session
        .manager()
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

    let archive_size = std::fs::metadata(archive_path).map(|m| m.len()).unwrap_or(0);

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
/// 根据 format 分发到 parsers 子模块对应解析函数，提取 game_version / loader /
/// mod_files_count / archive_base_folder 等字段。
pub(super) fn parse_modpack_info(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    match detected.format {
        ModpackFormat::Curseforge => parsers::parse_cf(detected),
        ModpackFormat::Modrinth => parsers::parse_mr(detected),
        ModpackFormat::Hmcl => parsers::parse_hmcl(detected),
        ModpackFormat::Mmc => parsers::parse_mmc(detected),
        ModpackFormat::Mcbbs => parsers::parse_mcbbs(detected),
        ModpackFormat::LauncherPack => parsers::parse_launcher_pack(detected),
        ModpackFormat::Compress => parsers::parse_compress(detected),
    }
}

/// 从 ModpackInfo 提取可选 Mod 列表（CF required=false / MR env.client=optional）
///
/// 用于前端 preview 后弹窗显示。HMCL/MMC/MCBBS 返回空列表（mods 已打包在 overrides 中）。
pub(super) fn extract_optional_mods(info: &ModpackInfo) -> Vec<OptionalModInfo> {
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
                    file_size: 0,
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
        // HMCL/MMC/MCBBS/LauncherPack/Compress 整合包 mods 已打包在 overrides 中
        ModpackFormat::Hmcl
        | ModpackFormat::Mmc
        | ModpackFormat::Mcbbs
        | ModpackFormat::LauncherPack
        | ModpackFormat::Compress => Vec::new(),
    }
}

/// 复制外部 Logo 文件到版本目录 `MoLaunch/Logo.png` 并写入 setup.ini
///
/// 用于 CurseForge / Modrinth 在线下载安装时复制平台缓存缩略图为版本图标。
/// `logo_path` 为 None 或文件不存在时直接返回 Ok。失败时返回 Err，由调用方决定是否中断。
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
