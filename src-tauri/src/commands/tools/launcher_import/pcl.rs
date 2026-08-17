//! PCL2 / PCL2CE 启动器探测与实例枚举
//!
//! - PCL2：读取注册表 `HKCU\SOFTWARE\PCL\LaunchFolders`（格式 `名称>路径|名称>路径`），
//!   实例布局为 `{folder}/versions/{name}/{name}.json`（版本隔离）或 folder 本身为 `.minecraft`（共享）。
//! - PCL2CE：读取 `{data_dir}/PCLCE/config.v1.json` 的 LaunchFolders，布局同 PCL2。

use std::path::{Path, PathBuf};

use crate::log_debug;

use super::detect::{data_dir, has_own_json, instance_from_dir, sorted_subdirs};
use crate::commands::tools::types::{ImportableInstance, LauncherKind, LauncherSource};

/// 探测 PCL2（Windows 注册表）
pub(super) fn detect_pcl2() -> LauncherSource {
    let raw = read_pcl_registry_folders();
    match raw {
        Some(raw) => {
            let folders = parse_pcl_folders(&raw);
            collect_source(LauncherKind::Pcl2, folders)
        }
        None => empty_source(LauncherKind::Pcl2, "未检测到 PCL2 注册表配置".to_string()),
    }
}

/// 探测 PCL2CE（config.v1.json）
pub(super) fn detect_pcl2ce() -> LauncherSource {
    let config_path = data_dir().map(|d| d.join("PCLCE/config.v1.json"));
    let Some(config_path) = config_path.filter(|p| p.is_file()) else {
        return empty_source(LauncherKind::Pcl2Ce, "未检测到 PCL2CE 配置文件".to_string());
    };

    let json = super::detect::read_text_file(&config_path)
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    let Some(json) = json else {
        log_debug!("[LauncherImport] PCL2CE 配置文件解析失败");
        return empty_source(LauncherKind::Pcl2Ce, "PCL2CE 配置文件解析失败".to_string());
    };

    let folders: Vec<(String, PathBuf)> = json
        .get("LaunchFolders")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let name = item.get("Name")?.as_str()?.to_string();
                    let path = item.get("Path")?.as_str()?.to_string();
                    let path = PathBuf::from(path);
                    if !path.is_dir() {
                        return None;
                    }
                    Some((name, path))
                })
                .collect()
        })
        .unwrap_or_default();

    collect_source(LauncherKind::Pcl2Ce, folders)
}

/// 读取 PCL2 注册表 LaunchFolders
#[cfg(target_os = "windows")]
fn read_pcl_registry_folders() -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(r"SOFTWARE\PCL").ok()?;
    key.get_value::<String, _>("LaunchFolders").ok()
}

#[cfg(not(target_os = "windows"))]
fn read_pcl_registry_folders() -> Option<String> {
    None
}

/// 解析 LaunchFolders 字符串：`名称>路径|名称>路径`
fn parse_pcl_folders(raw: &str) -> Vec<(String, PathBuf)> {
    raw.split('|')
        .filter_map(|entry| {
            let (name, path) = entry.split_once('>')?;
            let path = PathBuf::from(path.trim());
            if !path.is_dir() {
                return None;
            }
            Some((name.trim().to_string(), path))
        })
        .collect()
}

/// 从文件夹集合收集实例（版本隔离子目录或 folder 本身）
fn collect_source(kind: LauncherKind, folders: Vec<(String, PathBuf)>) -> LauncherSource {
    let base_path = folders
        .first()
        .map(|(_, p)| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut instances = Vec::new();
    for (name, folder) in folders {
        instances.extend(collect_from_folder(&name, &folder));
    }
    // 多个 LaunchFolder 可能匹配同一实例（共享 .minecraft 布局），按路径去重
    let mut seen = std::collections::HashSet::new();
    instances.retain(|i| seen.insert(i.path.clone()));
    instances.sort_by(|a, b| a.name.cmp(&b.name));

    LauncherSource {
        kind,
        label: kind.label().to_string(),
        base_path,
        instances,
    }
}

/// 枚举单个文件夹下的实例
fn collect_from_folder(folder_name: &str, folder: &Path) -> Vec<ImportableInstance> {
    let versions_dir = folder.join("versions");
    if versions_dir.is_dir() {
        // 版本隔离布局：versions/{name}/{name}.json
        let mut instances: Vec<ImportableInstance> = sorted_subdirs(&versions_dir)
            .iter()
            .filter(|d| has_own_json(d))
            .map(|d| {
                let name = d.file_name().unwrap_or_default().to_string_lossy();
                instance_from_dir(&name, d)
            })
            .collect();
        // 该 folder 可能同时是共享 .minecraft（非隔离），不重复枚举
        instances.sort_by(|a, b| a.name.cmp(&b.name));
        return instances;
    }

    // 共享布局：folder 本身为实例
    if folder.join("launcher_profiles.json").is_file() || folder.join("options.txt").is_file() {
        vec![instance_from_dir(folder_name, folder)]
    } else {
        Vec::new()
    }
}

fn empty_source(kind: LauncherKind, reason: String) -> LauncherSource {
    log_debug!("[LauncherImport] {}: {}", kind.label(), reason);
    LauncherSource {
        kind,
        label: kind.label().to_string(),
        base_path: String::new(),
        instances: Vec::new(),
    }
}
