//! MCBBS 格式（.zip）：`mcbbs.packmeta` + `overrides/`

use std::path::Path;

use tauri::AppHandle;

use crate::commands::version::export::types::{ExportFileInfo, ExportModpackParams};
use crate::log_info;
use crate::minecraft::version::state::VersionType;

use super::helpers::{
    create_zip_writer, emit_zip_progress, parse_dependencies, parse_loader_info, write_file_entry,
    write_string_entry,
};

/// 构建 MCBBS 格式整合包(.zip）
///
/// - mcbbs.packmeta: addons[]（game + 加载器）+ launchInfo
/// - overrides/: 所有文件（含 mods）
pub(super) fn build_mcbbs_zip(
    instance_dir: &Path,
    files: &[ExportFileInfo],
    params: &ExportModpackParams,
    pack_path: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    let (mc_version, _) = parse_dependencies(instance_dir, &params.version_id);
    let mc_version = mc_version.unwrap_or_default();
    if mc_version.is_empty() {
        return Err("无法解析 Minecraft 版本，MCBBS 格式必须包含 game addon".to_string());
    }

    // 构建 addons[]：game + 加载器 addon（id 与加载器类型对应）
    let (loader_type, loader_ver) = parse_loader_info(instance_dir, &params.version_id);
    let mut addons: Vec<serde_json::Value> = vec![serde_json::json!({
        "id": "game",
        "version": mc_version
    })];

    if let Some(v) = loader_ver {
        let id = match loader_type {
            VersionType::Forge => "forge",
            VersionType::NeoForge => "neoforge",
            VersionType::Fabric => "fabric",
            VersionType::Quilt => "quilt", // MCBBS 支持 quilt，但导入时会拒绝；导出时仍写入（兼容其他启动器）
            _ => "",
        };
        if !id.is_empty() {
            addons.push(serde_json::json!({ "id": id, "version": v }));
        }
    }

    log_info!(
        "[Export] MCBBS 打包：总 {} 文件，{} 个 addons，全部打包到 overrides/",
        files.len(),
        addons.len()
    );

    let (mut zip, options) = create_zip_writer(pack_path)?;

    // mcbbs.packmeta
    let packmeta = serde_json::json!({
        "name": params.pack_name,
        "version": params.pack_version,
        "author": "MoLaunch",
        "description": "",
        "addons": addons,
        "launchInfo": {
            "javaArgument": [],
            "launchArgument": []
        }
    });
    let packmeta_str = serde_json::to_string_pretty(&packmeta)
        .map_err(|e| format!("序列化 mcbbs.packmeta 失败: {}", e))?;
    write_string_entry(&mut zip, options, "mcbbs.packmeta", &packmeta_str)?;
    log_info!(
        "[Export] mcbbs.packmeta 已写入 ({} 字节)",
        packmeta_str.len()
    );

    // overrides/ 前缀
    let total = files.len();
    for (i, f) in files.iter().enumerate() {
        let zip_path = format!("overrides/{}", f.relative_path);
        write_file_entry(&mut zip, options, &zip_path, &f.abs_path)?;
        emit_zip_progress(app, &params.version_id, i + 1, total);
    }
    log_info!("[Export] overrides 文件写入完成");

    zip.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}
