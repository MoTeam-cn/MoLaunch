//! ZIP 安装与 Zip Slip 防护。

use super::super::{ProviderInfo, ProviderManifest};
use super::{ensure_provider_root, finalize_install, prepare_install_target};
use std::io::Read;
use std::path::Path;

pub async fn install_provider_from_zip(zip_path: String) -> Result<ProviderInfo, String> {
    let (manifest, temp_dir) = {
        let zip_file = std::path::PathBuf::from(&zip_path);
        if !zip_file.exists() {
            return Err(format!("ZIP 文件不存在: {}", zip_path));
        }
        let file = std::fs::File::open(&zip_file).map_err(|e| format!("打开 ZIP 失败: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {}", e))?;
        let names: Vec<String> = archive.file_names().map(String::from).collect();
        let prefix = determine_zip_prefix(&names)?;
        let manifest_entry = format!("{}manifest.json", prefix);
        let idx = names
            .iter()
            .position(|name| *name == manifest_entry)
            .ok_or_else(|| "ZIP 中未找到 manifest.json".to_string())?;
        let mut manifest_file = archive
            .by_index(idx)
            .map_err(|e| format!("读取 ZIP 内 manifest 失败: {}", e))?;
        let mut manifest_str = String::new();
        manifest_file
            .read_to_string(&mut manifest_str)
            .map_err(|e| format!("读取 manifest 内容失败: {}", e))?;
        drop(manifest_file);
        let manifest: ProviderManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;
        super::super::validate_provider_id(&manifest.id)?;
        let temp_dir = std::env::temp_dir().join(format!(
            "molaunch-provider-extract-{}-{}",
            manifest.id,
            std::process::id()
        ));
        if temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }
        crate::utils::fs::ensure_dir(&temp_dir)?;
        if let Err(e) = extract_zip_safely(&mut archive, &prefix, &temp_dir) {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(format!("解压失败: {}", e));
        }
        (manifest, temp_dir)
    };
    let (target_dir, skip) = prepare_install_target(&manifest)?;
    let result = if target_dir.exists() {
        let old_version = read_installed_version(&target_dir);
        if !version_changed(&manifest.version, old_version.as_deref()) {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Ok(super::build_provider_info(&manifest));
        }
        crate::utils::fs::ensure_dir(&target_dir)?;
        let added =
            super::files::merge_dir_incremental(&temp_dir, &target_dir, &skip, &temp_dir)?.1;
        finalize_install(&target_dir, &manifest, false, added, true)
    } else {
        ensure_provider_root()?;
        let added = super::files::copy_dir_recursive(&temp_dir, &target_dir, &skip, &temp_dir)?;
        finalize_install(&target_dir, &manifest, true, added, true)
    };
    let _ = std::fs::remove_dir_all(&temp_dir);
    result
}

fn read_installed_version(target_dir: &Path) -> Option<String> {
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(target_dir.join("manifest.json")).ok()?)
            .ok()?;
    value
        .get("version")
        .and_then(|v| v.as_str())
        .map(String::from)
}
fn version_changed(new: &str, old: Option<&str>) -> bool {
    old.map(|v| v != new).unwrap_or(true)
}

fn determine_zip_prefix(names: &[String]) -> Result<String, String> {
    let mut roots = std::collections::HashSet::new();
    let mut flat = false;
    for name in names {
        if name.contains('/') {
            if let Some(root) = name.split('/').next().filter(|v| !v.is_empty()) {
                roots.insert(root);
            }
        } else if !name.is_empty() {
            flat = true;
        }
    }
    if flat {
        return Ok(String::new());
    }
    if roots.len() == 1 {
        return Ok(format!("{}/", roots.into_iter().next().unwrap()));
    }
    Ok(String::new())
}

fn extract_zip_safely<R: Read + std::io::Seek>(
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
        let path = dst.join(rel);
        if rel.ends_with('/') {
            std::fs::create_dir_all(path).map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))?;
            if !parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {}", e))?
                .starts_with(&canonical_dst)
            {
                return Err(format!("Zip Slip 检测: {}", rel));
            }
        }
        let mut out = std::fs::File::create(path).map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut file, &mut out).map_err(|e| format!("写入文件失败: {}", e))?;
    }
    Ok(())
}
