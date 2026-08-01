//! 整合包安装共享逻辑（online / local 共用）

use crate::log_info;
use crate::minecraft::community::secure_storage;
use crate::state::{AppState, StageStatus};

use super::super::concurrent;
use super::super::modpack_stages::{copy_external_logo, migrate_modpack_config};
use super::super::types::{ModpackFormat, ModpackInfo};

/// 校验 CurseForge API Key（source=0 强制镜像时跳过）
///
/// 在线安装在 reset_stages 前按 platform 调用；本地安装在 detect 后按 format 调用。
pub(super) async fn validate_cf_api_key() -> Result<(), String> {
    let source = crate::minecraft::community::get_source_pref();
    if source == 0 {
        log_info!("[Community] CF source=0 强制镜像，跳过 API Key 检查（镜像站自带 Key）");
        return Ok(());
    }
    let (enabled, api_key) = secure_storage::get_config_async().await;
    if !enabled {
        return Err(
            "CurseForge 整合包安装需要 API Key。请在「设置 → 社区资源」中启用 CurseForge 官方源并填写 API Key，或将下载源切换为「尽量镜像」使用镜像站。"
                .to_string(),
        );
    }
    let key_empty = api_key.as_deref().is_none_or(|k| k.is_empty());
    if key_empty {
        return Err(
            "CurseForge 整合包安装需要 API Key。已在设置中启用但未填写 API Key，请补全后重试，或将下载源切换为「尽量镜像」使用镜像站。"
                .to_string(),
        );
    }
    log_info!("[Community] CF API Key 检查通过 (source={})", source);
    Ok(())
}

/// 按格式下载依赖 mods（CF/MR 有依赖列表，HMCL/MMC/MCBBS/Compress 跳过）
///
/// P0 安全修复：cf_manifest / mr_index 用 ok_or_else 返回 Result，避免 panic。
pub(super) async fn download_mods_by_format(
    state: &AppState,
    info: &ModpackInfo,
    mods_dir: &std::path::Path,
    instance_dir: &std::path::Path,
    stage_index: usize,
    include_optional: bool,
    is_local: bool,
) -> Result<(), String> {
    match info.format {
        ModpackFormat::Curseforge => {
            let manifest = info
                .cf_manifest
                .as_ref()
                .ok_or_else(|| "CF manifest 应已解析".to_string())?;
            super::super::curseforge::install_cf_mods(
                state,
                &manifest.files,
                mods_dir,
                instance_dir,
                stage_index,
                include_optional,
            )
            .await?;
        }
        ModpackFormat::Modrinth => {
            let index = info
                .mr_index
                .as_ref()
                .ok_or_else(|| "MR index 应已解析".to_string())?;
            super::super::modrinth::install_mr_files(
                state,
                &index.files,
                instance_dir,
                stage_index,
                include_optional,
            )
            .await?;
        }
        ModpackFormat::Hmcl | ModpackFormat::Mmc | ModpackFormat::Mcbbs => {
            log_info!(
                "[Community] {:?} {}整合包无依赖 mods 列表，跳过 Stage {}",
                info.format,
                if is_local { "本地" } else { "" },
                stage_index
            );
        }
        ModpackFormat::LauncherPack => {
            return Err(if is_local {
                "LauncherPack 不应进入主安装流程（应在入口预检测阶段递归处理）".to_string()
            } else {
                "在线下载的整合包不应为 LauncherPack 格式（带启动器整合包），请改用拖拽安装"
                    .to_string()
            });
        }
        ModpackFormat::Compress => {
            log_info!(
                "[Community] Compress {}整合包无依赖 mods 列表，跳过 Stage {}",
                if is_local { "本地" } else { "" },
                stage_index
            );
        }
    }
    Ok(())
}

/// 复制 overrides + 迁移配置 + 复制外部 Logo（online/local 共用收尾阶段）
pub(super) fn finalize_overrides_and_config(
    archive: &mut zip::ZipArchive<std::fs::File>,
    info: &ModpackInfo,
    instance_dir: &std::path::Path,
    state: &AppState,
    stage_index: usize,
    instance_name: &str,
    logo_path: Option<&str>,
) -> Result<(), String> {
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(stage_index, StageStatus::Loading, 0.0);
    }
    let prefixes = concurrent::build_overrides_prefixes(
        info.format,
        &info.archive_base_folder,
        info.cf_overrides_name.as_deref(),
    );
    concurrent::extract_overrides(archive, instance_dir, state, &prefixes, stage_index)?;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(stage_index, StageStatus::Finished, 1.0);
        // 不调用 mark_complete()：前端会紧接着调用 install_merged 安装 MC 本体，
        // 轮询必须继续。mark_complete 由 install_merged 在全部完成后调用。
    }
    // 迁移 MMC instance.cfg / MCBBS launchInfo 配置（必须在 extract_overrides 之后）
    migrate_modpack_config(info, instance_dir, instance_name)?;
    // 复制外部 Logo（失败不中断安装，仅装饰性）
    if let Err(e) = copy_external_logo(logo_path, instance_dir) {
        crate::log_warn!("[Community] 复制外部 Logo 失败（不中断安装）: {}", e);
    }
    Ok(())
}
