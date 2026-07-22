//! 外部插件沙箱内部访问（列出 / 读取 / 卸载）
//!
//! - `list_external_plugins`：扫描 `<base_dir>/plugins/<plugin_id>/` 目录
//! - `read_external_plugin_file`：读取插件文件内容（路径遍历防护）
//! - `uninstall_external_plugin`：卸载插件（删除目录）
//!
//! 共享类型与 helper 在 `super::` 中（`ExternalPluginEntry` / `is_valid_plugin_id` /
//! `plugins_root` / `read_plugin_manifest`）。

use super::{is_valid_plugin_id, plugins_root, read_plugin_manifest, ExternalPluginEntry};
use crate::log_info;

/// 列出所有已安装的外部插件
///
/// 扫描 `<base_dir>/plugins/` 目录，读取每个插件的 `manifest.json`，
/// 要求 manifest.id 与目录名一致。manifest 损坏的插件会被跳过（日志记录）。
#[tauri::command]
pub async fn list_external_plugins() -> Result<Vec<ExternalPluginEntry>, String> {
    let root = plugins_root();
    if !root.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(&root).map_err(|e| e.to_string())?;

    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let id = match path.file_name().and_then(|n| n.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        if !is_valid_plugin_id(&id) {
            continue;
        }

        match read_plugin_manifest(&id) {
            Ok(manifest) => {
                entries.push(ExternalPluginEntry {
                    manifest,
                    dir: path.display().to_string(),
                });
            }
            Err(e) => {
                log_info!("跳过插件 {}（manifest 无效）: {}", id, e);
            }
        }
    }

    Ok(entries)
}

/// 读取外部插件文件内容
///
/// 安全限制：`file_path` 必须是相对路径，且解析后必须位于插件目录内。
/// 使用 `canonicalize` + `starts_with` 双重校验防止 `../` 路径遍历攻击。
#[tauri::command]
pub async fn read_external_plugin_file(
    plugin_id: String,
    file_path: String,
) -> Result<String, String> {
    if !is_valid_plugin_id(&plugin_id) {
        return Err(format!("Invalid plugin id: {}", plugin_id));
    }

    // 禁止绝对路径 / 显式相对路径前缀
    if file_path.starts_with('/') || file_path.starts_with('\\') || file_path.contains("..") {
        return Err(format!("Invalid file path: {}", file_path));
    }

    let plugin_dir = plugins_root().join(&plugin_id);
    let target = plugin_dir.join(&file_path);

    // canonicalize + starts_with 双重校验防止 `../` 路径遍历
    let canonical_plugin_dir = plugin_dir.canonicalize().map_err(|e| e.to_string())?;
    let canonical_target = target.canonicalize().map_err(|e| e.to_string())?;

    if !canonical_target.starts_with(&canonical_plugin_dir) {
        return Err(format!("Path traversal denied: {}", file_path));
    }

    std::fs::read_to_string(&canonical_target).map_err(|e| e.to_string())
}

/// 卸载外部插件（删除插件目录）
///
/// 二次 canonicalize 校验确保不会逃逸出 `plugins/` 根目录。
#[tauri::command]
pub async fn uninstall_external_plugin(plugin_id: String) -> Result<(), String> {
    if !is_valid_plugin_id(&plugin_id) {
        return Err(format!("Invalid plugin id: {}", plugin_id));
    }

    let plugin_dir = plugins_root().join(&plugin_id);

    if !plugin_dir.exists() {
        return Ok(()); // 幂等：已不存在视为成功
    }

    // 二次 canonicalize 校验
    let canonical_root = plugins_root().canonicalize().map_err(|e| e.to_string())?;
    let canonical_dir = plugin_dir.canonicalize().map_err(|e| e.to_string())?;

    if !canonical_dir.starts_with(&canonical_root) {
        return Err(format!("Path traversal denied: {}", plugin_id));
    }

    std::fs::remove_dir_all(&canonical_dir).map_err(|e| e.to_string())?;
    log_info!("插件 {} 已卸载", plugin_id);

    Ok(())
}
