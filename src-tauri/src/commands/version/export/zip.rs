//! Zip 打包 + manifest 生成
//!
//! 支持 6 种导出格式（与导入格式对齐）：
//! - Modrinth: `modrinth.index.json` + `overrides/`
//! - CurseForge: `manifest.json` + `modlist.html` + `overrides/`
//! - HMCL: `modpack.json` + `minecraft/`
//! - MultiMC: `mmc-pack.json` + `instance.cfg` + `.minecraft/`
//! - MCBBS: `mcbbs.packmeta` + `overrides/`
//! - Compress: `.minecraft/` 兜底
//!
//! 仅 Modrinth/CurseForge 走联网检查（mod_infos 非空），
//! 其他格式所有文件全部打包到对应 overrides 前缀。

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use tauri::{AppHandle, Emitter};

use crate::log_info;
use crate::log_warn;
use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;

use super::EXPORT_PROGRESS_EVENT;
use super::types::{
    ExportFileInfo, ExportFormat, ExportModpackParams, ExportProgress, ExportStage, ModDownloadInfo,
    MrIndexFile, MrIndexHashes, MrIndexJson,
};

/// 构建整合包 zip 文件（按格式分发）
///
/// `app` 用于在打包过程中按文件数 emit 进度事件（50-95% 区间）。
pub fn build_modpack_zip(
    instance_dir: &Path,
    files: &[ExportFileInfo],
    mod_infos: &mut Vec<ModDownloadInfo>,
    params: &ExportModpackParams,
    summary: &str,
    pack_path: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    match params.format {
        ExportFormat::Modrinth => build_modrinth_zip(instance_dir, files, mod_infos, params, summary, pack_path, app),
        ExportFormat::Curseforge => build_curseforge_zip(instance_dir, files, mod_infos, params, pack_path, app),
        ExportFormat::Hmcl => build_hmcl_zip(instance_dir, files, params, pack_path, app),
        ExportFormat::Mmc => build_mmc_zip(instance_dir, files, params, pack_path, app),
        ExportFormat::Mcbbs => build_mcbbs_zip(instance_dir, files, params, pack_path, app),
        ExportFormat::Compress => build_compress_zip(files, pack_path, app, &params.version_id),
    }
}

/// 按 zip 写入进度 emit 事件（50-95% 区间，按文件数线性插值）
///
/// `current` 为当前已写入的文件数（从 0 开始），`total` 为待写入文件总数。
/// 区间设计：扫描+联网已占 0-50%，zip 写入占 50-95%，剩余 5% 留给完成事件。
fn emit_zip_progress(app: &AppHandle, version_id: &str, current: usize, total: usize) {
    if total == 0 {
        let _ = app.emit(
            EXPORT_PROGRESS_EVENT,
            ExportProgress::new(ExportStage::Zip, 95, "正在写入 zip...", version_id),
        );
        return;
    }
    // 50% 起步，每写入一个文件推进到 95%
    let ratio = current as f32 / total as f32;
    let percent = (50.0 + ratio * 45.0).min(95.0) as u8;
    let msg = format!("打包中 {}/{}", current, total);
    let _ = app.emit(
        EXPORT_PROGRESS_EVENT,
        ExportProgress::new(ExportStage::Zip, percent, msg, version_id),
    );
}

// ============== 共用工具 ==============

/// 创建 zip writer + 父目录
fn create_zip_writer(pack_path: &Path) -> Result<(zip::ZipWriter<File>, zip::write::SimpleFileOptions), String> {
    if let Some(parent) = pack_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建导出目录失败: {} ({})", parent.display(), e))?;
        }
    }
    let file = File::create(pack_path)
        .map_err(|e| format!("创建 zip 文件失败: {} ({})", pack_path.display(), e))?;
    let zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    Ok((zip, options))
}

/// 把单个文件写入 zip
fn write_file_entry(
    zip: &mut zip::ZipWriter<File>,
    options: zip::write::SimpleFileOptions,
    zip_path: &str,
    abs_path: &Path,
) -> Result<(), String> {
    zip.start_file(zip_path, options)
        .map_err(|e| format!("写入 zip 条目失败 '{}': {}", zip_path, e))?;
    let mut src = File::open(abs_path)
        .map_err(|e| format!("打开源文件失败 '{}': {}", abs_path.display(), e))?;
    let mut buf = Vec::new();
    src.read_to_end(&mut buf)
        .map_err(|e| format!("读取源文件失败 '{}': {}", abs_path.display(), e))?;
    zip.write_all(&buf)
        .map_err(|e| format!("写入 zip 内容失败 '{}': {}", zip_path, e))?;
    Ok(())
}

/// 把字符串写入 zip
fn write_string_entry(
    zip: &mut zip::ZipWriter<File>,
    options: zip::write::SimpleFileOptions,
    zip_path: &str,
    content: &str,
) -> Result<(), String> {
    zip.start_file(zip_path, options)
        .map_err(|e| format!("写入 zip 条目失败 '{}': {}", zip_path, e))?;
    zip.write_all(content.as_bytes())
        .map_err(|e| format!("写入 zip 内容失败 '{}': {}", zip_path, e))?;
    Ok(())
}

/// 从 instance 目录的 version json 解析版本依赖（复用 VersionSetup）
///
/// 返回 (minecraft_version, dependencies_map)
/// dependencies_map 含 `minecraft` + 加载器 key（`forge`/`fabric-loader`/`neoforge`/`quilt-loader`）
///
/// 复用 `VersionSetup::from_version_json`，与启动游戏、扫描版本列表走同一套检测逻辑，
/// 避免重复实现 maven 坐标解析和加载器识别。
fn parse_dependencies(instance_dir: &Path, version_id: &str) -> (Option<String>, HashMap<String, String>) {
    let mut deps = HashMap::new();
    let setup = match VersionSetup::from_version_json(instance_dir, version_id) {
        Some(s) => s,
        None => {
            log_warn!(
                "[Export] 无法解析版本 json: {}",
                instance_dir.join(format!("{}.json", version_id)).display()
            );
            return (None, deps);
        }
    };

    let mc_version = if setup.loader.original_version.is_empty() {
        None
    } else {
        Some(setup.loader.original_version.clone())
    };
    if let Some(v) = &mc_version {
        deps.insert("minecraft".to_string(), v.clone());
    }

    // 按 version_type 填充对应加载器版本（key 与 modrinth.index.json dependencies 字段一致）
    match setup.loader.version_type {
        VersionType::Forge => {
            if let Some(v) = &setup.loader.forge_version {
                deps.insert("forge".to_string(), v.clone());
            }
        }
        VersionType::NeoForge => {
            if let Some(v) = &setup.loader.neoforge_version {
                deps.insert("neoforge".to_string(), v.clone());
            }
        }
        VersionType::Fabric => {
            if let Some(v) = &setup.loader.fabric_version {
                deps.insert("fabric-loader".to_string(), v.clone());
            }
        }
        VersionType::Quilt => {
            if let Some(v) = &setup.loader.quilt_version {
                deps.insert("quilt-loader".to_string(), v.clone());
            }
        }
        _ => {}
    }

    (mc_version, deps)
}

/// 从 instance 目录的 version json 解析加载器类型和版本（复用 VersionSetup）
///
/// 返回 (VersionType, Option<版本号>)，供 CF/MMC/MCBBS 等 builder 复用。
fn parse_loader_info(instance_dir: &Path, version_id: &str) -> (VersionType, Option<String>) {
    let setup = match VersionSetup::from_version_json(instance_dir, version_id) {
        Some(s) => s,
        None => return (VersionType::Unknown, None),
    };
    let v = match setup.loader.version_type {
        VersionType::Forge => setup.loader.forge_version.clone(),
        VersionType::NeoForge => setup.loader.neoforge_version.clone(),
        VersionType::Fabric => setup.loader.fabric_version.clone(),
        VersionType::Quilt => setup.loader.quilt_version.clone(),
        VersionType::OptiFine => setup.loader.optifine_version.clone(),
        VersionType::LiteLoader => setup.loader.liteloader_version.clone(),
        _ => None,
    };
    (setup.loader.version_type, v)
}

// ============== Modrinth 格式 ==============

/// 构建 Modrinth 格式整合包（.mrpack）
fn build_modrinth_zip(
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
    log_info!("[Export] modrinth.index.json 已写入 ({} 字节)", index_str.len());

    // overrides/
    for (i, f) in override_files.iter().enumerate() {
        let zip_path = format!("overrides/{}", f.relative_path);
        write_file_entry(&mut zip, options, &zip_path, &f.abs_path)?;
        emit_zip_progress(app, &params.version_id, i + 1, total);
    }
    log_info!("[Export] overrides 文件写入完成");

    zip.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;
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

// ============== CurseForge 格式 ==============

/// 构建 CurseForge 格式整合包（.zip）
///
/// - manifest.json: 含 minecraft 版本 + 加载器 + files[]（projectID + fileID）
/// - modlist.html: 简单的 mod 列表
/// - overrides/: 其他文件
///
/// 联网获取到 projectID/fileID 的 mod 写入 files[]，未获取到的直接打包到 overrides。
fn build_curseforge_zip(
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
            !mod_infos
                .iter()
                .any(|m| m.relative_path == f.relative_path && m.project_id.is_some() && m.file_id.is_some())
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
    log_info!("[Export] manifest.json 已写入 ({} 字节)", manifest_str.len());

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

    zip.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;
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
        let name = m.relative_path.rsplit('/').next().unwrap_or(&m.relative_path);
        if !url.is_empty() {
            html.push_str(&format!(
                "  <li><a href=\"{}\">{}</a></li>\n",
                url, name
            ));
        } else {
            html.push_str(&format!("  <li>{}</li>\n", name));
        }
    }
    html.push_str("</ul>");
    html
}

// ============== HMCL 格式 ==============

/// 构建 HMCL 格式整合包（.zip）
///
/// - modpack.json: gameVersion + name
/// - minecraft/: 所有文件（含 mods）
///
/// HMCL 格式无 mods 下载列表，所有 mod 直接打包。
fn build_hmcl_zip(
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

    zip.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}

// ============== MultiMC 格式 ==============

/// 构建 MultiMC 格式整合包（.zip）
///
/// - mmc-pack.json: components[]（net.minecraft + 加载器）
/// - instance.cfg: name
/// - .minecraft/: 所有文件（含 mods）
fn build_mmc_zip(
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
    log_info!("[Export] mmc-pack.json 已写入 ({} 字节)", mmc_pack_str.len());

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

    zip.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}

// ============== MCBBS 格式 ==============

/// 构建 MCBBS 格式整合包(.zip）
///
/// - mcbbs.packmeta: addons[]（game + 加载器）+ launchInfo
/// - overrides/: 所有文件（含 mods）
fn build_mcbbs_zip(
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
    log_info!("[Export] mcbbs.packmeta 已写入 ({} 字节)", packmeta_str.len());

    // overrides/ 前缀
    let total = files.len();
    for (i, f) in files.iter().enumerate() {
        let zip_path = format!("overrides/{}", f.relative_path);
        write_file_entry(&mut zip, options, &zip_path, &f.abs_path)?;
        emit_zip_progress(app, &params.version_id, i + 1, total);
    }
    log_info!("[Export] overrides 文件写入完成");

    zip.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}

// ============== Compress 格式 ==============

/// 构建 Compress 格式整合包（.zip 兜底）
///
/// 直接打包 .minecraft/ 前缀，无 manifest 文件。
fn build_compress_zip(files: &[ExportFileInfo], pack_path: &Path, app: &AppHandle, version_id: &str) -> Result<(), String> {
    log_info!(
        "[Export] Compress 打包：总 {} 文件，全部打包到 .minecraft/",
        files.len()
    );

    let (mut zip, options) = create_zip_writer(pack_path)?;

    let total = files.len();
    for (i, f) in files.iter().enumerate() {
        let zip_path = format!(".minecraft/{}", f.relative_path);
        write_file_entry(&mut zip, options, &zip_path, &f.abs_path)?;
        emit_zip_progress(app, version_id, i + 1, total);
    }
    log_info!("[Export] .minecraft/ 文件写入完成");

    zip.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}

// ============== CF manifest 数据结构 ==============

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
