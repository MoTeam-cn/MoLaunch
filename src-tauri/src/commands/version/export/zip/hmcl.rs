//! HMCL 格式（.zip）：`modpack.json` + `minecraft/`

use std::path::Path;

use tauri::AppHandle;

use crate::commands::version::export::types::{ExportFileInfo, ExportModpackParams};
use crate::log_info;

use super::helpers::{
    create_zip_writer, emit_zip_progress, parse_dependencies, write_file_entry, write_string_entry,
};

/// 构建 HMCL 格式整合包（.zip）
///
/// - modpack.json: gameVersion + name
/// - minecraft/: 所有文件（含 mods）
///
/// HMCL 格式无 mods 下载列表，所有 mod 直接打包。
pub(super) fn build_hmcl_zip(
    instance_dir: &Path,
    files: &[ExportFileInfo],
    params: &ExportModpackParams,
    pack_path: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    // HMCL 不需要加载器版本（加载器信息打包在 minecraft/ 中），只需 mc 版本
    let (mc_version, _) = parse_dependencies(instance_dir, &params.version_id);
    let mc_version = mc_version.unwrap_or_default();

    log_info!(
        "[Export] HMCL 打包：总 {} 文件，全部打包到 minecraft/",
        files.len()
    );

    let (mut zip, options) = create_zip_writer(pack_path)?;

    // modpack.json
    let manifest = serde_json::json!({
        "gameVersion": mc_version,
        "name": params.pack_name,
        "version": params.pack_version,
        "author": "MoLaunch",
        "description": "",
        "fileApi": "",
        "launcher": "MoLaunch"
    });
    let manifest_str = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("序列化 modpack.json 失败: {}", e))?;
    write_string_entry(&mut zip, options, "modpack.json", &manifest_str)?;
    log_info!("[Export] modpack.json 已写入 ({} 字节)", manifest_str.len());

    // minecraft/ 前缀
    let total = files.len();
    for (i, f) in files.iter().enumerate() {
        let zip_path = format!("minecraft/{}", f.relative_path);
        write_file_entry(&mut zip, options, &zip_path, &f.abs_path)?;
        emit_zip_progress(app, &params.version_id, i + 1, total);
    }
    log_info!("[Export] minecraft/ 文件写入完成");

    zip.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}
