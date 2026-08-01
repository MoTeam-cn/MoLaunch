//! CurseForge 格式（.zip）：`manifest.json` + `modlist.html` + `overrides/`

use std::path::Path;

use tauri::AppHandle;

use crate::commands::version::export::types::{
    ExportFileInfo, ExportModpackParams, ModDownloadInfo,
};
use crate::minecraft::version::state::VersionType;
use crate::{log_info, log_warn};

use super::helpers::{
    create_zip_writer, emit_zip_progress, parse_dependencies, parse_loader_info, write_file_entry,
    write_string_entry,
};

/// 构建 CurseForge 格式整合包（.zip）
///
/// - manifest.json: 含 minecraft 版本 + 加载器 + files[]（projectID + fileID）
/// - modlist.html: 简单的 mod 列表
/// - overrides/: 其他文件
///
/// 联网获取到 projectID/fileID 的 mod 写入 files[]，未获取到的直接打包到 overrides。
pub(super) fn build_curseforge_zip(
    instance_dir: &Path,
    files: &[ExportFileInfo],
    mod_infos: &[ModDownloadInfo],
    params: &ExportModpackParams,
    pack_path: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    let (mc_version, _) = parse_dependencies(instance_dir, &params.version_id);
    let mc_version = mc_version.unwrap_or_default();
    if mc_version.is_empty() {
        return Err("无法解析 Minecraft 版本，CurseForge 格式必须包含 gameVersion".to_string());
    }

    // 加载器 ID（CF 格式：forge-<v> / fabric-loader-<v> / neoforge-<v>）
    let (loader_type, loader_ver) = parse_loader_info(instance_dir, &params.version_id);
    let loader_id = build_cf_loader_id(loader_type, loader_ver.as_deref());
    if loader_id.is_empty() {
        log_warn!(
            "[Export] CurseForge 未检测到加载器 ({:?})，manifest.modLoaders 将为空",
            loader_type
        );
    }

    // 区分有 projectID/fileID 的 mod（写入 files[]）和无的（打包到 overrides）
    let mut cf_files: Vec<CfManifestFile> = Vec::new();

    for m in mod_infos {
        if let (Some(pid), Some(fid)) = (m.project_id, m.file_id) {
            cf_files.push(CfManifestFile {
                project_id: pid,
                file_id: fid,
                required: true,
            });
        }
    }

    // 其他文件全部打包到 overrides（含未获取到 CF 信息的 mod）
    let override_files: Vec<&ExportFileInfo> = files
        .iter()
        .filter(|f| {
            // 已写入 files[] 的 mod 不打包
            !mod_infos.iter().any(|m| {
                m.relative_path == f.relative_path && m.project_id.is_some() && m.file_id.is_some()
            })
        })
        .collect();

    log_info!(
        "[Export] CurseForge 打包：总 {} 文件，{} 个 mod 写入 manifest.files，{} 个文件打包到 overrides",
        files.len(),
        cf_files.len(),
        override_files.len()
    );

    let (mut zip, options) = create_zip_writer(pack_path)?;

    // manifest.json
    let manifest = CfManifest {
        manifest_type: "minecraftModpack".to_string(),
        manifest_version: 1,
        name: params.pack_name.clone(),
        version: params.pack_version.clone(),
        author: "MoLaunch".to_string(),
        minecraft: CfMinecraft {
            version: mc_version.clone(),
            mod_loaders: if loader_id.is_empty() {
                Vec::new()
            } else {
                vec![CfModLoader {
                    id: loader_id,
                    primary: true,
                }]
            },
        },
        files: cf_files,
        overrides: "overrides".to_string(),
    };
    let manifest_str = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("序列化 manifest.json 失败: {}", e))?;
    write_string_entry(&mut zip, options, "manifest.json", &manifest_str)?;
    log_info!(
        "[Export] manifest.json 已写入 ({} 字节)",
        manifest_str.len()
    );

    // modlist.html（CF 标准：含每个 mod 的链接）
    let modlist = build_modlist_html(mod_infos);
    write_string_entry(&mut zip, options, "modlist.html", &modlist)?;

    // overrides/
    let total = override_files.len();
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

/// 构造 CF 加载器 ID
///
/// - Fabric: `fabric-loader-<version>`
/// - Forge: `forge-<version>`
/// - NeoForge: `neoforge-<version>`
/// - Quilt: CF 不支持 quilt，返回空字符串
/// - 其他：返回空字符串
fn build_cf_loader_id(loader_type: VersionType, version: Option<&str>) -> String {
    let v = match version {
        Some(v) if !v.is_empty() => v,
        _ => return String::new(),
    };
    match loader_type {
        VersionType::Forge => format!("forge-{}", v),
        VersionType::NeoForge => format!("neoforge-{}", v),
        VersionType::Fabric => format!("fabric-loader-{}", v),
        _ => String::new(),
    }
}

/// 生成 modlist.html（CF 标准：ul + 每个文件一个 li 链接）
fn build_modlist_html(mod_infos: &[ModDownloadInfo]) -> String {
    let mut html = String::from("<ul>\n");
    for m in mod_infos {
        let url = m.downloads.first().cloned().unwrap_or_default();
        let name = m
            .relative_path
            .rsplit('/')
            .next()
            .unwrap_or(&m.relative_path);
        if !url.is_empty() {
            html.push_str(&format!("  <li><a href=\"{}\">{}</a></li>\n", url, name));
        } else {
            html.push_str(&format!("  <li>{}</li>\n", name));
        }
    }
    html.push_str("</ul>");
    html
}

#[derive(serde::Serialize)]
struct CfManifest {
    manifest_type: String,
    manifest_version: u32,
    name: String,
    version: String,
    author: String,
    minecraft: CfMinecraft,
    files: Vec<CfManifestFile>,
    overrides: String,
}

#[derive(serde::Serialize)]
struct CfMinecraft {
    version: String,
    mod_loaders: Vec<CfModLoader>,
}

#[derive(serde::Serialize)]
struct CfModLoader {
    id: String,
    primary: bool,
}

#[derive(serde::Serialize)]
struct CfManifestFile {
    project_id: i64,
    file_id: i64,
    required: bool,
}
