//! 共享辅助：zip I/O（writer/条目写入）、进度推送、版本依赖/加载器解析
//!
//! 各格式 builder 通过 `super::helpers::*` 复用，避免重复实现。

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use tauri::{AppHandle, Emitter};

use crate::log_warn;
use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;

use crate::commands::version::export::types::{ExportProgress, ExportStage};
use crate::commands::version::export::EXPORT_PROGRESS_EVENT;

/// 按 zip 写入进度 emit 事件（50-95% 区间，按文件数线性插值）
///
/// `current` 为当前已写入的文件数（从 0 开始），`total` 为待写入文件总数。
/// 区间设计：扫描+联网已占 0-50%，zip 写入占 50-95%，剩余 5% 留给完成事件。
pub(super) fn emit_zip_progress(app: &AppHandle, version_id: &str, current: usize, total: usize) {
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

/// 创建 zip writer + 父目录
pub(crate) fn create_zip_writer(
    pack_path: &Path,
) -> Result<(zip::ZipWriter<File>, zip::write::SimpleFileOptions), String> {
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
pub(super) fn write_file_entry(
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
pub(crate) fn write_string_entry(
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
pub(super) fn parse_dependencies(
    instance_dir: &Path,
    version_id: &str,
) -> (Option<String>, HashMap<String, String>) {
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
pub(super) fn parse_loader_info(
    instance_dir: &Path,
    version_id: &str,
) -> (VersionType, Option<String>) {
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
