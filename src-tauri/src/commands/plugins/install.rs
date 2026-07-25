//! 外部插件安装（文件夹 / ZIP）
//!
//! - `install_external_plugin_from_dir`：递归复制源目录到 `plugins/<id>/`
//! - `install_external_plugin_from_zip`：从 ZIP 文件路径安装
//!
//! ZIP 结构支持：
//! - 扁平结构（根直接包含 manifest.json）
//! - 单根目录结构（ZIP 内有一个根目录，其下包含 manifest.json）
//!
//! 安全：ZIP 解压带 Zip Slip 路径遍历防护（canonicalize 父目录后校验目标在 dst 内），
//! 跨盘符 rename 失败时自动回退到递归复制。
//!
//! 注：原 2 个分散的 plugins Tauri 命令已聚合为 `plugins_manager` 一个 IPC 入口，
//! 通过请求体的 `action` 字段分发。子模块函数已去掉 `#[tauri::command]` 标注，
//! 由 `utils::plugins_manager::dispatch` 反序列化参数后调用。

use super::{is_valid_plugin_id, plugins_root, read_plugin_manifest};
use crate::error_util::log_err;
use crate::log_info;
use std::path::{Path, PathBuf};

/// 从源目录安装外部插件
///
/// 安装前校验插件 ID 合法性（kebab-case：小写字母 + 数字 + 连字符，
/// 不以连字符开头 / 结尾）。返回安装后的插件 ID。
pub async fn install_external_plugin_from_dir(source_dir: String) -> Result<String, String> {
    let src = PathBuf::from(&source_dir);

    // 校验源目录存在且为目录
    if !src.is_dir() {
        return Err(format!("Source directory not found: {}", source_dir));
    }

    // 读取源目录的 manifest.json 确定插件 ID
    let src_manifest_path = src.join("manifest.json");
    if !src_manifest_path.exists() {
        return Err(format!(
            "manifest.json not found in source: {}",
            src_manifest_path.display()
        ));
    }

    let manifest_str = std::fs::read_to_string(&src_manifest_path).map_err(log_err("Failed to read source manifest"))?;
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest_str).map_err(|e| format!("Invalid manifest.json: {}", e))?;
    let plugin_id = manifest
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "manifest.json missing 'id' field".to_string())?
        .to_string();

    // 校验插件 ID 合法性
    if !is_valid_plugin_id(&plugin_id) {
        return Err(format!("Invalid plugin id: {}", plugin_id));
    }

    // 目标目录
    let dst = plugins_root().join(&plugin_id);
    if dst.exists() {
        return Err(format!("Plugin already exists: {}", plugin_id));
    }

    // 创建目标目录
    std::fs::create_dir_all(&dst).map_err(log_err("Failed to create plugin directory"))?;

    // 递归复制
    if let Err(e) = copy_dir_recursive(&src, &dst) {
        // 失败时清理
        let _ = std::fs::remove_dir_all(&dst);
        return Err(format!("Failed to copy plugin files: {}", e));
    }

    // 校验安装结果（manifest.id 与目录名一致等）
    if let Err(e) = read_plugin_manifest(&plugin_id) {
        let _ = std::fs::remove_dir_all(&dst);
        return Err(format!("Install verification failed: {}", e));
    }

    log_info!("插件 {} 已从目录安装", plugin_id);

    Ok(plugin_id)
}

/// 从 ZIP 文件路径安装外部插件
///
/// 支持扁平结构和单根目录结构两种 ZIP 格式。带 Zip Slip 路径遍历防护。
/// 跨盘符 rename 失败时自动回退到递归复制。
pub async fn install_external_plugin_from_zip(zip_path: String) -> Result<String, String> {
    let zip_file = PathBuf::from(&zip_path);
    if !zip_file.exists() {
        return Err(format!("ZIP file not found: {}", zip_path));
    }

    // 打开 ZIP
    let file = std::fs::File::open(&zip_file).map_err(log_err("Failed to open ZIP file"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(log_err("Failed to read ZIP archive"))?;

    // 探测 ZIP 前缀（扁平结构 "" 或单根目录 "xxx/"）
    let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
    let prefix = determine_zip_prefix(&names);

    // 从 ZIP 中读取 manifest.json 确定 plugin_id
    let manifest_entry = if prefix.is_empty() {
        "manifest.json".to_string()
    } else {
        format!("{}manifest.json", prefix)
    };

    let manifest_idx = names
        .iter()
        .position(|n| *n == manifest_entry)
        .ok_or_else(|| "manifest.json not found in ZIP".to_string())?;

    let mut manifest_file = archive
        .by_index(manifest_idx)
        .map_err(log_err("Failed to read manifest from ZIP"))?;
    let mut manifest_str = String::new();
    std::io::Read::read_to_string(&mut manifest_file, &mut manifest_str)
        .map_err(log_err("Failed to read manifest content"))?;
    drop(manifest_file);

    let manifest: serde_json::Value =
        serde_json::from_str(&manifest_str).map_err(|e| format!("Invalid manifest.json: {}", e))?;
    let plugin_id = manifest
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "manifest.json missing 'id' field".to_string())?
        .to_string();

    if !is_valid_plugin_id(&plugin_id) {
        return Err(format!("Invalid plugin id: {}", plugin_id));
    }

    // 目标目录
    let dst = plugins_root().join(&plugin_id);
    if dst.exists() {
        return Err(format!("Plugin already exists: {}", plugin_id));
    }

    std::fs::create_dir_all(&dst).map_err(log_err("Failed to create plugin directory"))?;

    // 安全解压（防 Zip Slip）
    if let Err(e) = extract_zip_safely(&mut archive, &prefix, &dst) {
        // 失败时清理
        let _ = std::fs::remove_dir_all(&dst);
        return Err(format!("Failed to extract ZIP: {}", e));
    }

    // 校验安装结果
    if let Err(e) = read_plugin_manifest(&plugin_id) {
        let _ = std::fs::remove_dir_all(&dst);
        return Err(format!("Install verification failed: {}", e));
    }

    log_info!("插件 {} 已从 ZIP 安装", plugin_id);

    Ok(plugin_id)
}

/// 递归复制源目录到目标目录
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}

/// 探测 ZIP 前缀（扁平结构返回 ""，单根目录返回 "xxx/"）
///
/// - 若 ZIP 中存在根级文件（无 `/` 分隔），视为扁平结构
/// - 若所有文件都在同一根目录下，返回 "xxx/"
/// - 多根目录或无文件，视为扁平结构
fn determine_zip_prefix(names: &[String]) -> String {
    let mut root_dirs = std::collections::HashSet::new();
    let mut has_flat_files = false;

    for name in names {
        if name.contains('/') {
            let root = name.split('/').next().unwrap_or("");
            if !root.is_empty() {
                root_dirs.insert(root.to_string());
            }
        } else if !name.is_empty() {
            // 扁平结构（根直接有文件）
            has_flat_files = true;
        }
    }

    // 如果有扁平文件，视为扁平结构
    if has_flat_files {
        return String::new();
    }

    // 如果只有一个根目录，返回 "xxx/"
    if root_dirs.len() == 1 {
        let root = root_dirs.iter().next().unwrap();
        return format!("{}/", root);
    }

    // 多根或无根，视为扁平结构
    String::new()
}

/// 安全解压 ZIP（防 Zip Slip：canonicalize 父目录后校验目标在 dst 内）
fn extract_zip_safely<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    prefix: &str,
    dst: &Path,
) -> std::io::Result<()> {
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let name = file.name().to_string();

        // 跳过非前缀的项
        if !name.starts_with(prefix) {
            continue;
        }

        // 去掉前缀得到相对路径
        let rel = &name[prefix.len()..];
        if rel.is_empty() {
            continue;
        }

        // 目录项
        if rel.ends_with('/') {
            let dir_path = dst.join(rel);
            std::fs::create_dir_all(&dir_path)?;
            continue;
        }

        // 文件项
        let file_path = dst.join(rel);

        // Zip Slip 校验：canonicalize 父目录，确保目标在 dst 内
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
            let canonical_parent = parent.canonicalize()?;
            if !canonical_parent.starts_with(&canonical_dst) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("Zip Slip detected: {}", rel),
                ));
            }
        }

        let mut out = std::fs::File::create(&file_path)?;
        std::io::copy(&mut file, &mut out)?;
    }

    Ok(())
}
