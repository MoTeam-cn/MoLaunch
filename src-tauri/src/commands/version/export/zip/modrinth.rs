//! Modrinth 格式（.mrpack）：`modrinth.index.json` + `overrides/`

use std::collections::HashMap;
use std::path::Path;

use tauri::AppHandle;

use crate::commands::version::export::types::{
    ExportFileInfo, ExportModpackParams, ModDownloadInfo, MrIndexFile, MrIndexHashes, MrIndexJson,
};
use crate::log_info;

use super::helpers::{
    create_zip_writer, emit_zip_progress, parse_dependencies, write_file_entry, write_string_entry,
};

/// 构建 Modrinth 格式整合包（.mrpack）
pub(super) fn build_modrinth_zip(
    instance_dir: &Path,
    files: &[ExportFileInfo],
    mod_infos: &[ModDownloadInfo],
    params: &ExportModpackParams,
    summary: &str,
    pack_path: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    let (_, dependencies) = parse_dependencies(instance_dir, &params.version_id);
    log_info!(
        "[Export] Modrinth 依赖: minecraft={}, deps={:?}",
        dependencies.get("minecraft").cloned().unwrap_or_default(),
        dependencies
    );

    // 联网获取到下载地址的 mod 不打包
    let mod_paths: std::collections::HashSet<String> =
        mod_infos.iter().map(|m| m.relative_path.clone()).collect();
    let override_files: Vec<&ExportFileInfo> = files
        .iter()
        .filter(|f| !mod_paths.contains(&f.relative_path))
        .collect();

    let total = override_files.len();
    log_info!(
        "[Export] Modrinth 打包：总 {} 文件，{} 个 mod 走联网下载，{} 个文件打包到 overrides",
        files.len(),
        mod_infos.len(),
        total
    );

    let (mut zip, options) = create_zip_writer(pack_path)?;

    // modrinth.index.json
    let index_json = build_index_json(mod_infos, params, summary, &dependencies);
    let index_str = serde_json::to_string_pretty(&index_json)
        .map_err(|e| format!("序列化 modrinth.index.json 失败: {}", e))?;
    write_string_entry(&mut zip, options, "modrinth.index.json", &index_str)?;
    log_info!(
        "[Export] modrinth.index.json 已写入 ({} 字节)",
        index_str.len()
    );

    // overrides/
    for (i, f) in override_files.iter().enumerate() {
        let zip_path = format!("overrides/{}", f.relative_path);
        write_file_entry(&mut zip, options, &zip_path, &f.abs_path)?;
        emit_zip_progress(app, &params.version_id, i + 1, total);
    }
    log_info!("[Export] overrides 文件写入完成");

    zip.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}

/// 构造 modrinth.index.json 结构
fn build_index_json(
    mod_infos: &[ModDownloadInfo],
    params: &ExportModpackParams,
    summary: &str,
    dependencies: &HashMap<String, String>,
) -> MrIndexJson {
    let files: Vec<MrIndexFile> = mod_infos
        .iter()
        .map(|m| MrIndexFile {
            path: m.relative_path.clone(),
            hashes: MrIndexHashes {
                sha1: m.sha1.clone(),
                sha512: m.sha512.clone(),
            },
            downloads: m.downloads.clone(),
            file_size: m.file_size,
        })
        .collect();

    MrIndexJson {
        game: "minecraft".to_string(),
        format_version: 1,
        version_id: params.pack_version.clone(),
        name: params.pack_name.clone(),
        summary: summary.to_string(),
        files,
        dependencies: dependencies.clone(),
    }
}
