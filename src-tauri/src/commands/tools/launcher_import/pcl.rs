//! PCL2 / PCL2CE 启动器探测与实例枚举
//!
//! - PCL2：读取注册表 `HKCU\SOFTWARE\PCL\LaunchFolders`（格式 `名称>路径|名称>路径`），
//!   实例布局为 `{folder}/versions/{name}/{name}.json`（版本隔离）或 folder 本身为 `.minecraft`（共享）。
//! - PCL2CE：读取 `{data_dir}/PCLCE/config.v1.json` 的 LaunchFolders，布局同 PCL2。

use std::path::{Path, PathBuf};

use crate::log_debug;

use super::detect::{
    data_dir, has_own_json, home_dir, instance_from_dir, program_files, read_text_file,
    sorted_subdirs,
};
use crate::commands::tools::types::{ImportableInstance, LauncherKind, LauncherSource};

/// 探测 PCL2（Windows 注册表 LaunchFolders + Setup.ini 兜底）
pub(super) fn detect_pcl2() -> LauncherSource {
    let mut folders: Vec<(String, PathBuf)> = read_pcl_registry_folders()
        .map(|raw| parse_pcl_folders(&raw))
        .unwrap_or_default();

    // Setup.ini 兜底：未配置"启动文件夹"（注册表无 LaunchFolders）时，
    // 从注册表 CacheDownloadFolder 或常见位置发现 PCL 根目录，解析 LaunchFolderSelect
    // 定位共享 .minecraft / 版本隔离布局的游戏数据目录
    if folders.is_empty() {
        if let Some(folder) = detect_pcl2_setup() {
            folders.push(folder);
        }
    }

    if folders.is_empty() {
        return empty_source(
            LauncherKind::Pcl2,
            "未检测到 PCL2 配置（注册表无 LaunchFolders，未找到 Setup.ini）".to_string(),
        );
    }
    collect_source(LauncherKind::Pcl2, folders)
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
    read_pcl_registry_value::<String>("LaunchFolders")
}

#[cfg(not(target_os = "windows"))]
fn read_pcl_registry_folders() -> Option<String> {
    None
}

/// 读取 PCL2 注册表任意值（HKCU\SOFTWARE\PCL\{name}）
#[cfg(target_os = "windows")]
fn read_pcl_registry_value<T: winreg::types::FromRegValue>(name: &str) -> Option<T> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(r"SOFTWARE\PCL").ok()?;
    key.get_value::<T, _>(name).ok()
}

#[cfg(not(target_os = "windows"))]
fn read_pcl_registry_value<T>(_name: &str) -> Option<T> {
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

/// Setup.ini 兜底探测：返回（folder 名, 游戏数据目录）
///
/// 覆盖未配置"启动文件夹"的 PCL2 用户：从注册表 `CacheDownloadFolder`（PCL2 新版）
/// 或常见位置扫描发现 PCL 根目录（含 `Setup.ini`），再解析 `LaunchFolderSelect`
/// 定位游戏数据目录（`$` = PCL 根目录的父目录，PCL2 约定 Minecraft 文件夹在启动器上一级）。
fn detect_pcl2_setup() -> Option<(String, PathBuf)> {
    let root = find_pcl_root()?;
    let game_data = resolve_launch_folder(&root)?;
    if !game_data.is_dir() {
        log_debug!(
            "[LauncherImport] PCL2 Setup.ini 定位的游戏目录不存在: {}",
            game_data.display()
        );
        return None;
    }
    Some(("PCL2".to_string(), game_data))
}

/// 发现 PCL2 根目录（含 Setup.ini）：
/// 1. 注册表 `CacheDownloadFolder`（形如 `{PCL根}\MyDownload\`，取父目录）；
/// 2. 常见位置（桌面/文档/下载/Program Files）扫描含 PCL 特征 Setup.ini 的目录。
fn find_pcl_root() -> Option<PathBuf> {
    // 注册表线索优先（PCL2 新版必定写入 CacheDownloadFolder）
    if let Some(download) = read_pcl_registry_value::<String>("CacheDownloadFolder") {
        let download = PathBuf::from(download.trim_end_matches(['\\', '/']));
        if let Some(parent) = download.parent() {
            if parent.join("Setup.ini").is_file() {
                log_debug!(
                    "[LauncherImport] PCL2 根目录（注册表 CacheDownloadFolder）: {}",
                    parent.display()
                );
                return Some(parent.to_path_buf());
            }
        }
    }

    let mut scan_bases: Vec<PathBuf> = Vec::new();
    if let Some(home) = home_dir() {
        scan_bases.push(home.join("Desktop"));
        scan_bases.push(home.join("Documents"));
        scan_bases.push(home.join("Downloads"));
    }
    if let Some(system) = program_files() {
        scan_bases.push(system);
    }
    for base in scan_bases {
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let dir = entry.path();
                if dir.is_dir() && dir.join("Setup.ini").is_file() && looks_like_pcl_dir(&dir) {
                    log_debug!(
                        "[LauncherImport] PCL2 根目录（常见位置扫描）: {}",
                        dir.display()
                    );
                    return Some(dir);
                }
            }
        }
    }
    None
}

/// 判断 Setup.ini 是否属于 PCL2（含 PCL 特征键，避免误判其他软件的配置文件）
fn looks_like_pcl_dir(dir: &Path) -> bool {
    let content = read_text_file(&dir.join("Setup.ini")).unwrap_or_default();
    content.contains("LaunchFolderSelect")
        || content.contains("LaunchArgumentIndieV2")
        || content.contains("UiLauncherTheme")
}

/// 解析 Setup.ini 的 `LaunchFolderSelect`，返回游戏数据目录
///
/// 规则（PCL2 路径变量）：
/// - `$` 前缀：PCL 根目录的父目录（Minecraft 文件夹约定在启动器上一级）；
/// - 绝对路径：直接使用；
/// - 其余相对路径：相对 PCL 根目录。
fn resolve_launch_folder(root: &Path) -> Option<PathBuf> {
    let setup = read_text_file(&root.join("Setup.ini"))?;
    let select = setup
        .lines()
        .find_map(|l| l.strip_prefix("LaunchFolderSelect:"))?;
    let select = select.trim().trim_end_matches(['\\', '/']).trim();
    if select.is_empty() {
        return Some(root.parent()?.to_path_buf());
    }
    if let Some(rest) = select.strip_prefix('$') {
        let rest = rest.trim_start_matches(['\\', '/']).trim();
        return Some(if rest.is_empty() {
            root.parent()?.to_path_buf()
        } else {
            root.parent()?.join(rest)
        });
    }
    let p = PathBuf::from(select);
    Some(if p.is_absolute() {
        p
    } else {
        root.join(select)
    })
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
///
/// - 版本隔离布局：`versions/{name}/{name}.json` 各版本目录为实例；
/// - 共享布局：folder 本身为 .minecraft 根（含游戏数据特征 saves/mods 等）时追加为实例，
///   避免与版本隔离枚举冲突（纯 versions 容器根目录不算共享实例）。
fn collect_from_folder(folder_name: &str, folder: &Path) -> Vec<ImportableInstance> {
    let mut instances: Vec<ImportableInstance> = Vec::new();

    let versions_dir = folder.join("versions");
    if versions_dir.is_dir() {
        instances.extend(
            sorted_subdirs(&versions_dir)
                .iter()
                .filter(|d| has_own_json(d))
                .map(|d| {
                    let name = d.file_name().unwrap_or_default().to_string_lossy();
                    instance_from_dir(&name, d)
                }),
        );
    }

    // 共享布局：folder 本身有游戏数据（saves/mods/config 等）且带启动器标记文件
    let has_game_data = [
        "saves",
        "mods",
        "config",
        "resourcepacks",
        "shaderpacks",
        "options.txt",
    ]
    .iter()
    .any(|n| folder.join(n).exists());
    if has_game_data
        && (folder.join("launcher_profiles.json").is_file() || folder.join("options.txt").is_file())
    {
        instances.push(instance_from_dir(folder_name, folder));
    }

    instances.sort_by(|a, b| a.name.cmp(&b.name));
    instances
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_root(name: &str, select: &str) -> (PathBuf, PathBuf) {
        let tmp = std::env::temp_dir().join(format!("molaunch_pcl_{}", name));
        let root = tmp.join("PCL");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("Setup.ini"), format!("LaunchFolderSelect:{}\n", select)).unwrap();
        (tmp, root)
    }

    #[test]
    fn resolve_launch_folder_dollar_prefix() {
        // `$.minecraft\` → PCL 根目录的父目录 + .minecraft（PCL2 默认布局）
        let (tmp, root) = setup_root("dollar", "$.minecraft\\");
        assert_eq!(resolve_launch_folder(&root).unwrap(), tmp.join(".minecraft"));
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_launch_folder_absolute() {
        // 绝对路径直接使用
        let (tmp, root) = setup_root("abs", "C:\\custom\\mc");
        assert_eq!(
            resolve_launch_folder(&root).unwrap(),
            PathBuf::from("C:\\custom\\mc")
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_launch_folder_relative() {
        // 无前缀相对路径 → 相对 PCL 根目录
        let (tmp, root) = setup_root("rel", ".minecraft");
        assert_eq!(resolve_launch_folder(&root).unwrap(), root.join(".minecraft"));
        std::fs::remove_dir_all(&tmp).ok();
    }
}
