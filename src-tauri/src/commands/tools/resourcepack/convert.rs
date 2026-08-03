//! 资源包格式转换（convert）

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::{resolve_packs_dir, path_to_string};
use super::super::types::{ResourcePackConvertParams, ResourcePackConvertResult};

/// 转换资源包格式（folder ↔ zip）
///
/// - folder → zip：把目录内容打包为同名 .zip（`MyPack` → `MyPack.zip`）
/// - zip → folder：解压到同名目录（`MyPack.zip` → `MyPack/`）
/// - 目标已存在时返回 success=false
pub async fn convert(
    state: &AppState,
    params: ResourcePackConvertParams,
) -> Result<serde_json::Value, String> {
    // 与 list 保持一致：按 version_id 解析版本隔离目录，避免版本隔离模式下全局 resourcepacks 不存在
    let packs_dir = resolve_packs_dir(state, params.version_id.as_deref()).await;

    // 目录不存在时给出明确提示（而非 canonicalize 抛 os error 2）
    if !packs_dir.exists() {
        return Err(format!(
            "resourcepacks 目录不存在: {}（请在游戏中放置资源包后再转换）",
            packs_dir.display()
        ));
    }

    // 路径安全：源路径规范化后必须在 resourcepacks 目录内
    let packs_canon = match packs_dir.canonicalize() {
        Ok(c) => c,
        Err(e) => return Err(format!("resourcepacks 目录解析失败: {}", e)),
    };
    let src = PathBuf::from(&params.path);
    let src_canon = match src.canonicalize() {
        Ok(c) => c,
        Err(e) => return Err(format!("源路径解析失败: {}", e)),
    };
    if !src_canon.starts_with(&packs_canon) {
        return Err("路径不在 resourcepacks 目录内".to_string());
    }

    let target_format = params.target_format.to_lowercase();
    if target_format != "zip" && target_format != "folder" {
        return Err("target_format 必须为 zip 或 folder".to_string());
    }

    log_info!(
        "[ResourcePack] 转换: src={}, target={}",
        src_canon.display(),
        target_format
    );

    let is_src_zip = src_canon.is_file()
        && src_canon
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.eq_ignore_ascii_case("zip"))
            .unwrap_or(false);
    let is_src_dir = src_canon.is_dir();

    // 确定输出路径与转换动作
    let (output_path, action) = if target_format == "zip" && is_src_dir {
        // folder → zip：用目录名作为 zip 名（避免 file_stem 误判含点的目录名）
        let name = src_canon
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("pack")
            .to_string();
        (packs_dir.join(format!("{}.zip", name)), "folder_to_zip")
    } else if target_format == "folder" && is_src_zip {
        // zip → folder：去掉 .zip 后缀作为目录名
        let stem = src_canon
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("pack")
            .to_string();
        (packs_dir.join(stem), "zip_to_folder")
    } else {
        let src_kind = if is_src_dir { "folder" } else { "zip" };
        return serde_json::to_value(&ResourcePackConvertResult {
            success: false,
            output_path: String::new(),
            message: format!(
                "源路径类型与目标格式不匹配（源为 {}，目标 {}）",
                src_kind, target_format
            ),
        })
        .map_err(|e| e.to_string());
    };

    // 目标已存在直接返回失败
    if output_path.exists() {
        return serde_json::to_value(&ResourcePackConvertResult {
            success: false,
            output_path: path_to_string(&output_path),
            message: format!("目标已存在: {}", output_path.display()),
        })
        .map_err(|e| e.to_string());
    }

    let output_path_clone = output_path.clone();
    let src_canon_clone = src_canon.clone();
    let action_clone = action.to_string();
    let convert_result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        match action_clone.as_str() {
            "folder_to_zip" => zip_directory(&src_canon_clone, &output_path_clone),
            "zip_to_folder" => unzip_to_dir(&src_canon_clone, &output_path_clone),
            _ => Err("未知转换类型".to_string()),
        }
    })
    .await
    .map_err(log_err("ResourcePack 转换任务失败"))?;

    if let Err(e) = convert_result {
        log_warn!("[ResourcePack] 转换失败: {}", e);
        return serde_json::to_value(&ResourcePackConvertResult {
            success: false,
            output_path: path_to_string(&output_path),
            message: e,
        })
        .map_err(|e| e.to_string());
    }

    log_info!("[ResourcePack] 转换成功: {}", output_path.display());

    let result = ResourcePackConvertResult {
        success: true,
        output_path: path_to_string(&output_path),
        message: "转换成功".to_string(),
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 递归收集目录下所有文件路径
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_file() {
            out.push(path);
        } else if path.is_dir() {
            collect_files(&path, out);
        }
    }
}

/// 将目录内容打包为 zip（保留相对路径，zip 内使用正斜杠）
fn zip_directory(src_dir: &Path, output_zip: &Path) -> Result<(), String> {
    let file = File::create(output_zip).map_err(|e| format!("创建 zip 失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();

    let mut entries: Vec<PathBuf> = Vec::new();
    collect_files(src_dir, &mut entries);

    for file_path in entries {
        let rel = file_path
            .strip_prefix(src_dir)
            .map_err(|e| format!("路径前缀剥离失败: {}", e))?;
        let rel_str = rel.to_str().ok_or("路径包含非 UTF-8 字符")?;
        let zip_name = rel_str.replace('\\', "/");
        zip.start_file(&zip_name, options)
            .map_err(|e| format!("写入 zip 条目失败: {}", e))?;
        let mut f = File::open(&file_path).map_err(|e| format!("打开源文件失败: {}", e))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .map_err(|e| format!("读取源文件失败: {}", e))?;
        zip.write_all(&buf)
            .map_err(|e| format!("写入 zip 内容失败: {}", e))?;
    }

    zip.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    Ok(())
}

/// 解压 zip 到目录
fn unzip_to_dir(src_zip: &Path, output_dir: &Path) -> Result<(), String> {
    let file = File::open(src_zip).map_err(|e| format!("打开 zip 失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("读取 zip 失败: {}", e))?;
    std::fs::create_dir_all(output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;
    archive
        .extract(output_dir)
        .map_err(|e| format!("解压失败: {}", e))?;
    Ok(())
}