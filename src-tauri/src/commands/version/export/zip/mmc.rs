//! MultiMC 格式（.zip）：`mmc-pack.json` + `instance.cfg` + `.minecraft/`

use std::path::Path;

use tauri::AppHandle;

use crate::commands::version::export::types::{ExportFileInfo, ExportModpackParams};
use crate::log_info;
use crate::minecraft::version::state::VersionType;

use super::helpers::{
    create_zip_writer, emit_zip_progress, parse_dependencies, parse_loader_info, write_file_entry,
    write_string_entry,
};

/// 构建 MultiMC 格式整合包（.zip）
///
/// - mmc-pack.json: components[]（net.minecraft + 加载器）
/// - instance.cfg: name
/// - .minecraft/: 所有文件（含 mods）
pub(super) fn build_mmc_zip(
    instance_dir: &Path,
    files: &[ExportFileInfo],
    params: &ExportModpackParams,
    pack_path: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    let (mc_version, _) = parse_dependencies(instance_dir, &params.version_id);
    let mc_version = mc_version.unwrap_or_default();
    if mc_version.is_empty() {
        return Err("无法解析 Minecraft 版本，MultiMC 格式必须包含 net.minecraft 组件".to_string());
    }

    // 构建 components[]：net.minecraft + 加载器组件（uid 与加载器类型对应）
    let (loader_type, loader_ver) = parse_loader_info(instance_dir, &params.version_id);
    let mut components: Vec<serde_json::Value> = vec![serde_json::json!({
        "uid": "net.minecraft",
        "version": mc_version
    })];

    if let Some(v) = loader_ver {
        let uid = match loader_type {
            VersionType::Forge => "net.minecraftforge",
            VersionType::NeoForge => "net.neoforged",
            VersionType::Fabric => "net.fabricmc.fabric-loader",
            VersionType::Quilt => "org.quiltmc.quilt-loader",
            VersionType::LiteLoader => "com.mumfrey.liteloader",
            _ => "",
        };
        if !uid.is_empty() {
            components.push(serde_json::json!({
                "uid": uid,
                "version": v
            }));
        }
    }

    log_info!(
        "[Export] MultiMC 打包：总 {} 文件，{} 个组件，全部打包到 .minecraft/",
        files.len(),
        components.len()
    );

    let (mut zip, options) = create_zip_writer(pack_path)?;

    // mmc-pack.json
    let mmc_pack = serde_json::json!({ "components": components });
    let mmc_pack_str = serde_json::to_string_pretty(&mmc_pack)
        .map_err(|e| format!("序列化 mmc-pack.json 失败: {}", e))?;
    write_string_entry(&mut zip, options, "mmc-pack.json", &mmc_pack_str)?;
    log_info!(
        "[Export] mmc-pack.json 已写入 ({} 字节)",
        mmc_pack_str.len()
    );

    // instance.cfg（INI 格式，name=pack_name）
    let instance_cfg = format!("[General]\nname={}\n", params.pack_name);
    write_string_entry(&mut zip, options, "instance.cfg", &instance_cfg)?;

    // .minecraft/ 前缀
    let total = files.len();
    for (i, f) in files.iter().enumerate() {
        let zip_path = format!(".minecraft/{}", f.relative_path);
        write_file_entry(&mut zip, options, &zip_path, &f.abs_path)?;
        emit_zip_progress(app, &params.version_id, i + 1, total);
    }
    log_info!("[Export] .minecraft/ 文件写入完成");

    zip.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}
