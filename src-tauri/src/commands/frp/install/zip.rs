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

/// 探测 ZIP 顶层前缀（实现统一在 utils/archive.rs）
fn determine_zip_prefix(names: &[String]) -> Result<String, String> {
    Ok(crate::utils::archive::determine_zip_prefix(names))
}

/// 安全解压（实现统一在 utils/archive.rs，此处保留薄封装以稳定调用点）
fn extract_zip_safely<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    prefix: &str,
    dst: &Path,
) -> Result<(), String> {
    crate::utils::archive::extract_zip_safely(archive, prefix, dst, |_| true)
}

#[cfg(test)]
#[path = "zip_test.rs"]
mod zip_test;
