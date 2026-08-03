//! 插件系统共享 helper（目录定位 / ID 校验 / manifest 读取）

use std::path::PathBuf;

use crate::error_util::log_err;

use super::types::ExternalPluginManifest;

/// 获取外部插件根目录（`<base_dir>/plugins/`）
pub(crate) fn plugins_root() -> PathBuf {
    crate::storage::Storage::instance()
        .base_dir()
        .join("plugins")
}

/// 校验插件 ID 合法性（kebab-case，仅允许小写字母、数字、连字符）
pub(crate) fn is_valid_plugin_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

/// 读取插件 manifest.json
///
/// 校验 plugin_id 合法性 + manifest.id 与目录名一致。
pub(crate) fn read_plugin_manifest(plugin_id: &str) -> Result<ExternalPluginManifest, String> {
    if !is_valid_plugin_id(plugin_id) {
        return Err(format!("Invalid plugin id: {}", plugin_id));
    }

    let plugin_dir = plugins_root().join(plugin_id);
    let manifest_path = plugin_dir.join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!(
            "Plugin manifest not found: {}",
            manifest_path.display()
        ));
    }

    let manifest_str = std::fs::read_to_string(&manifest_path)
        .map_err(log_err("Failed to read plugin manifest"))?;
    let manifest: ExternalPluginManifest =
        serde_json::from_str(&manifest_str).map_err(|e| format!("Invalid manifest.json: {}", e))?;

    // 校验 manifest.id 与目录名一致
    if manifest.id != plugin_id {
        return Err(format!(
            "manifest.id ({}) 与目录名 ({}) 不一致",
            manifest.id, plugin_id
        ));
    }

    Ok(manifest)
}
