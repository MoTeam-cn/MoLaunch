//! 外部插件安装（文件夹 / ZIP）
//!
//! 支持扁平结构和单根目录结构，带 Zip Slip 路径遍历防护。
//! 跨盘符 rename 失败时自动回退到递归复制。

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

    let manifest_str = std::fs::read_to_string(&src_manifest_path)
        .map_err(log_err("Failed to read source manifest"))?;
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
    let prefix = determine_zip_prefix(&names)?;

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
    crate::utils::fs::copy_dir_recursive(src, dst).map_err(std::io::Error::other)
}

/// 探测 ZIP 前缀（扁平结构返回 ""，单根目录返回 "xxx/"）
///
/// - 若 ZIP 中存在根级文件（无 `/` 分隔），视为扁平结构
/// - 若所有文件都在同一根目录下，返回 "xxx/"
/// - 多根目录或无文件，视为扁平结构
fn determine_zip_prefix(names: &[String]) -> Result<String, String> {
    Ok(crate::utils::archive::determine_zip_prefix(names))
}

/// 安全解压 ZIP（防 Zip Slip：canonicalize 父目录后校验目标在 dst 内）
fn extract_zip_safely<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    prefix: &str,
    dst: &Path,
) -> std::io::Result<()> {
    crate::utils::archive::extract_zip_safely(archive, prefix, dst, |_| true)
        .map_err(std::io::Error::other)
}
