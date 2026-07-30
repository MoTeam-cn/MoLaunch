//! 厂商安装/卸载
//!
//! 从 `provider.rs` 拆分，职责：
//! - 外部厂商安装（文件夹 / ZIP + Zip Slip 防护）
//! - 外部厂商卸载（路径遍历防护）
//!
//! frpc 二进制下载逻辑见 [`super::binary`]。
//! 厂商列表/状态/启禁见 [`super::provider`]。

use super::provider::{
    is_external_frpc_ready, read_providers_state, write_providers_state, SYSTEM_DEFAULT_ID,
};
use super::{ensure_dir, providers_root, validate_provider_id, ProviderInfo, ProviderManifest};
use crate::log_info;
use std::path::{Path, PathBuf};

// ============================================================
// 安装 / 卸载
// ============================================================

/// 从文件夹安装外部厂商
///
/// 源目录必须包含 manifest.json。安装后校验 manifest.json 存在。
pub async fn install_provider_from_dir(source_dir: String) -> Result<ProviderInfo, String> {
    let src = Path::new(&source_dir);
    if !src.is_dir() {
        return Err(format!("源目录不存在: {}", source_dir));
    }
    let manifest_path = src.join("manifest.json");
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
    let manifest: ProviderManifest = serde_json::from_str(&content)
        .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;

    validate_provider_id(&manifest.id)?;
    let target_dir = providers_root().join(&manifest.id);
    if target_dir.exists() {
        return Err(format!("厂商已存在: {}", manifest.id));
    }

    ensure_dir(&providers_root())?;
    copy_dir_recursive(src, &target_dir)?;

    let installed_manifest_path = target_dir.join("manifest.json");
    if !installed_manifest_path.exists() {
        let _ = std::fs::remove_dir_all(&target_dir);
        return Err("安装校验失败：manifest.json 不存在".to_string());
    }

    log_info!("[Frp] 厂商已安装: {} ({})", manifest.name, manifest.id);
    Ok(build_provider_info(&manifest))
}

/// 从 ZIP 安装外部厂商
///
/// 支持扁平结构（根直接含 manifest.json）和单根目录结构。
/// 解压带 Zip Slip 防护（canonicalize 父目录后校验目标在 dst 内）。
pub async fn install_provider_from_zip(zip_path: String) -> Result<ProviderInfo, String> {
    let zip_file = PathBuf::from(&zip_path);
    if !zip_file.exists() {
        return Err(format!("ZIP 文件不存在: {}", zip_path));
    }

    let file = std::fs::File::open(&zip_file)
        .map_err(|e| format!("打开 ZIP 失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解析 ZIP 失败: {}", e))?;

    let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
    let prefix = determine_zip_prefix(&names);

    let manifest_entry = if prefix.is_empty() {
        "manifest.json".to_string()
    } else {
        format!("{}manifest.json", prefix)
    };
    let manifest_idx = names
        .iter()
        .position(|n| *n == manifest_entry)
        .ok_or_else(|| "ZIP 中未找到 manifest.json".to_string())?;
    let mut manifest_file = archive
        .by_index(manifest_idx)
        .map_err(|e| format!("读取 ZIP 内 manifest 失败: {}", e))?;
    let mut manifest_str = String::new();
    std::io::Read::read_to_string(&mut manifest_file, &mut manifest_str)
        .map_err(|e| format!("读取 manifest 内容失败: {}", e))?;
    drop(manifest_file);

    let manifest: ProviderManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;
    validate_provider_id(&manifest.id)?;

    let target_dir = providers_root().join(&manifest.id);
    if target_dir.exists() {
        return Err(format!("厂商已存在: {}", manifest.id));
    }

    ensure_dir(&providers_root())?;
    ensure_dir(&target_dir)?;
    if let Err(e) = extract_zip_safely(&mut archive, &prefix, &target_dir) {
        let _ = std::fs::remove_dir_all(&target_dir);
        return Err(format!("解压失败: {}", e));
    }

    let installed_manifest_path = target_dir.join("manifest.json");
    if !installed_manifest_path.exists() {
        let _ = std::fs::remove_dir_all(&target_dir);
        return Err("安装校验失败：manifest.json 不存在".to_string());
    }

    log_info!("[Frp] 厂商已从 ZIP 安装: {} ({})", manifest.name, manifest.id);
    Ok(build_provider_info(&manifest))
}

/// 卸载外部厂商
///
/// 不允许卸载系统默认厂商。删除目录前用 canonicalize 校验路径不逃逸 providers/ 根。
pub async fn uninstall_provider(provider_id: String) -> Result<(), String> {
    if provider_id == SYSTEM_DEFAULT_ID {
        return Err("不能卸载系统默认厂商".to_string());
    }
    validate_provider_id(&provider_id)?;
    let dir = providers_root().join(&provider_id);
    if !dir.exists() {
        return Err(format!("厂商不存在: {}", provider_id));
    }
    let canonical_root = providers_root()
        .canonicalize()
        .map_err(|e| format!("路径校验失败: {}", e))?;
    let canonical_dir = dir
        .canonicalize()
        .map_err(|e| format!("路径校验失败: {}", e))?;
    if !canonical_dir.starts_with(&canonical_root) {
        return Err("路径遍历检测".to_string());
    }
    std::fs::remove_dir_all(&dir).map_err(|e| format!("卸载失败: {}", e))?;

    let mut state = read_providers_state();
    state.remove(&provider_id);
    write_providers_state(&state)?;

    log_info!("[Frp] 厂商已卸载: {}", provider_id);
    Ok(())
}

// ============================================================
// 内部辅助
// ============================================================

/// 从 manifest + 启用状态构建 ProviderInfo
fn build_provider_info(manifest: &ProviderManifest) -> ProviderInfo {
    let state = read_providers_state();
    let frpc_ready = is_external_frpc_ready(&manifest.id, manifest);
    let enabled = state.get(&manifest.id).copied().unwrap_or(true);
    ProviderInfo {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        author: manifest.author.clone(),
        builtin: false,
        auth_type: manifest.auth.auth_type.clone(),
        frpc_ready,
        enabled,
        distribution: manifest.binary.distribution.clone(),
        homepage: manifest.homepage.clone(),
    }
}

/// 递归复制源目录到目标目录
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
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
            has_flat_files = true;
        }
    }
    if has_flat_files {
        return String::new();
    }
    if root_dirs.len() == 1 {
        let root = root_dirs.iter().next().unwrap();
        return format!("{}/", root);
    }
    String::new()
}

/// 安全解压 ZIP（防 Zip Slip：canonicalize 父目录后校验目标在 dst 内）
fn extract_zip_safely<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    prefix: &str,
    dst: &Path,
) -> Result<(), String> {
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = file.name().to_string();
        if !name.starts_with(prefix) {
            continue;
        }
        let rel = &name[prefix.len()..];
        if rel.is_empty() {
            continue;
        }
        if rel.ends_with('/') {
            std::fs::create_dir_all(dst.join(rel))
                .map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }
        let file_path = dst.join(rel);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建父目录失败: {}", e))?;
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {}", e))?;
            if !canonical_parent.starts_with(&canonical_dst) {
                return Err(format!("Zip Slip 检测: {}", rel));
            }
        }
        let mut out = std::fs::File::create(&file_path)
            .map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut file, &mut out)
            .map_err(|e| format!("写入文件失败: {}", e))?;
    }
    Ok(())
}
