//! 资源包管理
//!
//! - `list`：列出 resourcepacks 目录下顶层条目（.zip 文件 / 目录）
//!   - 默认扫全局 `{game_dir}/resourcepacks/`
//!   - 传入 `version_id` 时按版本隔离配置解析该版本的有效游戏目录
//! - `convert`：在 zip 与 folder 格式之间转换（folder → 打包为同名 .zip；zip → 解压为同名目录）

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::isolation::{get_effective_game_dir, IsolationMode};
use crate::state::AppState;
use crate::state::resolve_game_dir;

use super::types::{
    ResourcePackConvertParams, ResourcePackConvertResult, ResourcePackItem, ResourcePackListParams,
    ResourcePackListResult,
};

/// 解析资源包目录（同 screenshot::resolve_shots_dir 的语义）
async fn resolve_packs_dir(state: &AppState, version_id: Option<&str>) -> PathBuf {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    match version_id {
        None => game_dir.join("resourcepacks"),
        Some(vid) => {
            let global_mode = state.config.lock().await.isolation_mode;
            let isolation_mode =
                crate::commands::version::list::resolve_isolation_mode(&game_dir, vid, global_mode);
            let version_type =
                crate::commands::version::list::detect_version_type_from_dir(&game_dir, vid);
            let mode = IsolationMode::from_u32(isolation_mode);
            let effective_dir =
                get_effective_game_dir(&game_dir, vid, mode, version_type);
            effective_dir.join("resourcepacks")
        }
    }
}

/// 列出 resourcepacks 目录下顶层条目（.zip 文件 → zip；目录 → folder）
pub async fn list(
    state: &AppState,
    params: ResourcePackListParams,
) -> Result<serde_json::Value, String> {
    let packs_dir = resolve_packs_dir(state, params.version_id.as_deref()).await;

    log_info!("[ResourcePack] 列目录: {}", packs_dir.display());

    if !packs_dir.exists() {
        log_warn!(
            "[ResourcePack] resourcepacks 目录不存在: {}",
            packs_dir.display()
        );
        let result = ResourcePackListResult {
            items: Vec::new(),
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    let packs_dir_clone = packs_dir.clone();
    let items = tokio::task::spawn_blocking(move || -> Vec<ResourcePackItem> {
        let mut items: Vec<ResourcePackItem> = Vec::new();
        let read = match std::fs::read_dir(&packs_dir_clone) {
            Ok(r) => r,
            Err(_) => return items,
        };
        for entry in read.flatten() {
            let path = entry.path();
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if path.is_file() {
                if !name.to_lowercase().ends_with(".zip") {
                    continue;
                }
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                items.push(ResourcePackItem {
                    name,
                    path: path_to_string(&path),
                    format: "zip".to_string(),
                    size,
                });
            } else if path.is_dir() {
                let size = dir_total_size(&path);
                items.push(ResourcePackItem {
                    name,
                    path: path_to_string(&path),
                    format: "folder".to_string(),
                    size,
                });
            }
        }
        // 按名称排序，保证输出稳定
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    })
    .await
    .map_err(log_err("ResourcePack 列目录任务失败"))?;

    log_info!("[ResourcePack] 列出 {} 个资源包", items.len());

    let result = ResourcePackListResult { items };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 转换资源包格式（folder ↔ zip）
///
/// - folder → zip：把目录内容打包为同名 .zip（`MyPack` → `MyPack.zip`）
/// - zip → folder：解压到同名目录（`MyPack.zip` → `MyPack/`）
/// - 目标已存在时返回 success=false
pub async fn convert(
    state: &AppState,
    params: ResourcePackConvertParams,
) -> Result<serde_json::Value, String> {
    // convert 不需要 version_id（路径校验基于实际 packs_dir 解析）
    // 简化处理：用全局 game_dir/resourcepacks 作为基准目录
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    let packs_dir = game_dir.join("resourcepacks");

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
        return Ok(serde_json::to_value(&ResourcePackConvertResult {
            success: false,
            output_path: String::new(),
            message: format!(
                "源路径类型与目标格式不匹配（源为 {}，目标 {}）",
                src_kind, target_format
            ),
        })
        .map_err(|e| e.to_string())?);
    };

    // 目标已存在直接返回失败
    if output_path.exists() {
        return Ok(serde_json::to_value(&ResourcePackConvertResult {
            success: false,
            output_path: path_to_string(&output_path),
            message: format!("目标已存在: {}", output_path.display()),
        })
        .map_err(|e| e.to_string())?);
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
        return Ok(serde_json::to_value(&ResourcePackConvertResult {
            success: false,
            output_path: path_to_string(&output_path),
            message: e,
        })
        .map_err(|e| e.to_string())?);
    }

    log_info!("[ResourcePack] 转换成功: {}", output_path.display());

    let result = ResourcePackConvertResult {
        success: true,
        output_path: path_to_string(&output_path),
        message: "转换成功".to_string(),
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 递归计算目录总字节数
fn dir_total_size(dir: &Path) -> u64 {
    let mut total: u64 = 0;
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_file() {
            total += std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        } else if path.is_dir() {
            total += dir_total_size(&path);
        }
    }
    total
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
    let file =
        File::create(output_zip).map_err(|e| format!("创建 zip 失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();

    let mut entries: Vec<PathBuf> = Vec::new();
    collect_files(src_dir, &mut entries);

    for file_path in entries {
        let rel = file_path
            .strip_prefix(src_dir)
            .map_err(|e| format!("路径前缀剥离失败: {}", e))?;
        let rel_str = rel
            .to_str()
            .ok_or("路径包含非 UTF-8 字符")?;
        let zip_name = rel_str.replace('\\', "/");
        zip.start_file(&zip_name, options)
            .map_err(|e| format!("写入 zip 条目失败: {}", e))?;
        let mut f =
            File::open(&file_path).map_err(|e| format!("打开源文件失败: {}", e))?;
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
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("读取 zip 失败: {}", e))?;
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;
    archive
        .extract(output_dir)
        .map_err(|e| format!("解压失败: {}", e))?;
    Ok(())
}

/// 将路径转为字符串（UTF-8，丢失非 UTF-8 字符）
fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap_or("").to_string()
}
