//! Zip 打包分发逻辑实现（build_modpack_zip，原聚合入口 mod.rs 中的实现）
//!
//! 按 ExportFormat 分发到 6 种格式 builder；共享辅助与各格式实现在
//! `helpers` 及 `modrinth` / `curseforge` / `hmcl` / `mmc` / `mcbbs` / `compress`。

use std::path::Path;

use tauri::AppHandle;

use crate::commands::version::export::types::{
    ExportFileInfo, ExportFormat, ExportModpackParams, ModDownloadInfo,
};

/// 构建整合包 zip 文件（按格式分发）
///
/// `app` 用于在打包过程中按文件数 emit 进度事件（50-95% 区间）。
pub fn build_modpack_zip(
    instance_dir: &Path,
    files: &[ExportFileInfo],
    mod_infos: &mut [ModDownloadInfo],
    params: &ExportModpackParams,
    summary: &str,
    pack_path: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    match params.format {
        ExportFormat::Modrinth => super::modrinth::build_modrinth_zip(
            instance_dir,
            files,
            mod_infos,
            params,
            summary,
            pack_path,
            app,
        ),
        ExportFormat::Curseforge => super::curseforge::build_curseforge_zip(
            instance_dir,
            files,
            mod_infos,
            params,
            pack_path,
            app,
        ),
        ExportFormat::Hmcl => {
            super::hmcl::build_hmcl_zip(instance_dir, files, params, pack_path, app)
        }
        ExportFormat::Mmc => super::mmc::build_mmc_zip(instance_dir, files, params, pack_path, app),
        ExportFormat::Mcbbs => {
            super::mcbbs::build_mcbbs_zip(instance_dir, files, params, pack_path, app)
        }
        ExportFormat::Compress => {
            super::compress::build_compress_zip(files, pack_path, app, &params.version_id)
        }
    }
}
