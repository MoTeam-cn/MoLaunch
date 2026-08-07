//! 文件夹安装与增量合并职责。

use super::super::{ProviderInfo, ProviderManifest};
use super::{build_provider_info, ensure_provider_root, finalize_install, prepare_install_target};
use crate::log_info;
use std::path::Path;

pub async fn install_provider_from_dir(source_dir: String) -> Result<ProviderInfo, String> {
    let src = Path::new(&source_dir);
    if !src.is_dir() {
        return Err(format!("源目录不存在: {}", source_dir));
    }
    let content = std::fs::read_to_string(src.join("manifest.json"))
        .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
    let manifest: ProviderManifest =
        serde_json::from_str(&content).map_err(|e| format!("解析 manifest.json 失败: {}", e))?;
    let (target_dir, skip) = prepare_install_target(&manifest)?;
    let (is_install, added) = if target_dir.exists() {
        let old_version = read_installed_version(&target_dir);
        if !version_changed(&manifest.version, old_version.as_deref()) {
            log_info!(
                "[Frp] 厂商已是最新版本，跳过更新: {} (版本 {})",
                manifest.id,
                manifest.version
            );
            return Ok(build_provider_info(&manifest));
        }
        log_info!(
            "[Frp] 厂商版本变化，执行增量更新: {} ({} -> {})",
            manifest.id,
            old_version.unwrap_or_default(),
            manifest.version
        );
        crate::utils::fs::ensure_dir(&target_dir)?;
        (
            false,
            merge_dir_incremental(src, &target_dir, &skip, src)?.1,
        )
    } else {
        ensure_provider_root()?;
        (true, copy_dir_recursive(src, &target_dir, &skip, src)?)
    };
    finalize_install(&target_dir, &manifest, is_install, added, false)
}

fn read_installed_version(target_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(target_dir.join("manifest.json")).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&content).ok()?;
    manifest
        .get("version")
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn version_changed(new_version: &str, old_version: Option<&str>) -> bool {
    old_version.map(|old| old != new_version).unwrap_or(true)
}

pub(super) fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    skip: &std::collections::HashSet<String>,
    base: &Path,
) -> Result<u32, String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let mut count = 0;
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if path.is_dir() {
            count += copy_dir_recursive(&path, &dst_path, skip, base)?;
        } else {
            let rel = relative_path(&path, base);
            if skip.contains(&rel) {
                continue;
            }
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            std::fs::copy(&path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
            count += 1;
        }
    }
    Ok(count)
}

pub(super) fn merge_dir_incremental(
    src: &Path,
    dst: &Path,
    skip: &std::collections::HashSet<String>,
    base: &Path,
) -> Result<(u32, u32), String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let mut updated = 0;
    let mut added = 0;
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if path.is_dir() {
            let (u, a) = merge_dir_incremental(&path, &dst_path, skip, base)?;
            updated += u;
            added += a;
        } else {
            if skip.contains(&relative_path(&path, base)) {
                continue;
            }
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            if !dst_path.exists() {
                std::fs::copy(&path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
                added += 1;
            } else if std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?
                != std::fs::read(&dst_path).map_err(|e| format!("读取目标文件失败: {}", e))?
            {
                std::fs::copy(&path, &dst_path).map_err(|e| format!("覆盖文件失败: {}", e))?;
                updated += 1;
            }
        }
    }
    Ok((updated, added))
}

fn relative_path(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
