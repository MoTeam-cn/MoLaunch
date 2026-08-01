//! 导出选项定义 + 动态子选项扫描
//!
//! 提供 ~20 个静态选项 + 资源包/存档/光影包的动态子选项扫描。
//! 选项可见性根据实例目录实际文件决定。

use std::path::Path;

use super::types::ExportOption;

mod basic;
mod misc;
mod mods;
mod packs;
mod world;

/// 全局排除规则（附加在所有规则末尾，避免打包无用文件）
pub const GLOBAL_EXCLUDES: &[&str] = &[
    "!*.log",
    "!*.dat_old",
    "!*.BakaCoreInfo",
    "!hmclversion.cfg",
    "!log4j2.xml",
];

/// 子选项黑名单（资源包/存档/光影包扫描时跳过这些名称）
const SUB_OPTION_BLACKLIST: &[&str] = &["Quark Programmer Art.zip", "+ EuphoriaPatches_"];

/// 构建所有导出选项（静态 + 动态子选项）
pub fn build_all_options(instance_dir: &Path) -> Vec<ExportOption> {
    let mut opts = Vec::new();
    let rules_suffix = format!("|{}", GLOBAL_EXCLUDES.join("|"));
    basic::push(&mut opts, instance_dir, &rules_suffix);
    mods::push(&mut opts, instance_dir, &rules_suffix);
    packs::push(&mut opts, instance_dir, &rules_suffix);
    world::push(&mut opts, instance_dir, &rules_suffix);
    misc::push(&mut opts, instance_dir, &rules_suffix);
    opts
}

/// 检查实例目录下是否存在指定文件或目录之一
pub(super) fn has_file_or_dir(instance_dir: &Path, names: &[&str]) -> bool {
    names.iter().any(|n| instance_dir.join(n).exists())
}

/// 扫描子文件夹/压缩包，生成动态子选项（用于资源包/存档/光影包）
pub(super) fn scan_sub_options(
    instance_dir: &Path,
    folders: &[&str],
    accept_compressed: bool,
    accept_folder: bool,
) -> Vec<ExportOption> {
    let mut result = Vec::new();
    let parent_id = folders[0]; // 用第一个文件夹名作为 parent id 后缀

    for folder in folders {
        let target = instance_dir.join(folder);
        if !target.is_dir() {
            continue;
        }

        let entries = match std::fs::read_dir(&target) {
            Ok(e) => e.flatten().collect::<Vec<_>>(),
            Err(_) => continue,
        };

        // 压缩包文件
        if accept_compressed {
            for entry in &entries {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = entry.file_name();
                let name = match file_name.to_str() {
                    Some(n) => n,
                    None => continue,
                };
                if !name.ends_with(".zip") && !name.ends_with(".rar") {
                    continue;
                }
                if SUB_OPTION_BLACKLIST.iter().any(|b| name.contains(b)) {
                    continue;
                }
                let rule = format!("{}/{}", folder, name);
                result.push(ExportOption {
                    id: format!("{}_file_{}", parent_id, name),
                    title: name.to_string(),
                    description: None,
                    rules: Some(escape_glob_chars(&rule)),
                    show_rules: None,
                    default_checked: true,
                    checked: true,
                    parent: Some(parent_id.to_string()),
                    enabled: true,
                    visible: true,
                });
            }
        }

        // 子文件夹
        if accept_folder {
            let mut subdirs: Vec<_> = entries.into_iter().filter(|e| e.path().is_dir()).collect();
            subdirs.sort_by_key(|e| {
                e.metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });
            for entry in subdirs.into_iter().rev() {
                let path = entry.path();
                let file_name = entry.file_name();
                let name = match file_name.to_str() {
                    Some(n) => n,
                    None => continue,
                };
                if SUB_OPTION_BLACKLIST.iter().any(|b| name.contains(b)) {
                    continue;
                }
                // 跳过空文件夹
                if std::fs::read_dir(&path)
                    .map(|mut r| r.next().is_none())
                    .unwrap_or(true)
                {
                    continue;
                }
                let rule = format!("{}/{}/", folder, name);
                let description = if parent_id == "saves" {
                    entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| {
                            let secs = d.as_secs();
                            format!("修改时间: {}", format_timestamp(secs))
                        })
                } else {
                    None
                };
                result.push(ExportOption {
                    id: format!("{}_dir_{}", parent_id, name),
                    title: name.to_string(),
                    description,
                    rules: Some(escape_glob_chars(&rule)),
                    show_rules: None,
                    default_checked: true,
                    checked: true,
                    parent: Some(parent_id.to_string()),
                    enabled: true,
                    visible: true,
                });
            }
        }
    }

    result
}

/// 检查是否有 Licence 文件（LICEN* 通配）
pub(super) fn has_licence_file(instance_dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(instance_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if let Some(name) = file_name.to_str() {
                if name.to_uppercase().starts_with("LICEN") {
                    return true;
                }
            }
        }
    }
    false
}

/// 转义 glob 特殊字符（选项规则中的文件名可能含 `[` `]` 等）
fn escape_glob_chars(s: &str) -> String {
    // 用 `[x]` 包裹特殊字符进行转义
    s.replace('[', "[[]")
        .replace(']', "[]]")
        .replace('?', "[?]")
}

/// 简单时间戳格式化（yyyy/MM/dd HH:mm）
fn format_timestamp(secs: u64) -> String {
    // 直接用 SystemTime 转 chrono Local 时间格式化
    let time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs);
    let dt: chrono::DateTime<chrono::Local> = time.into();
    dt.format("%Y/%m/%d %H:%M").to_string()
}
