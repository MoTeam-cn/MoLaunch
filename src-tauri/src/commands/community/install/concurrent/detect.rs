//! 整合包格式检测

use crate::log_info;

use super::DetectedModpack;

/// 检测整合包格式
///
/// 识别优先级：mcbbs.packmeta → mmc-pack.json → modrinth.index.json →
/// manifest.json（有 addons 为 Mcbbs，否则 Curseforge）→ modpack.json →
/// modpack.zip/mrpack（LauncherPack）→ .minecraft/ 目录（Compress）。
/// 根目录命中时 `archive_base_folder` 为 `""`，子目录命中时为 `"子目录/"`。
pub fn detect_modpack_format(
    archive: &mut zip::ZipArchive<std::fs::File>,
) -> Result<DetectedModpack, String> {
    // 收集所有条目名及其索引
    let entry_names: Vec<(usize, String)> = (0..archive.len())
        .map(|i| {
            let name = archive
                .by_index(i)
                .map(|e| e.name().to_string())
                .unwrap_or_default();
            (i, name)
        })
        .collect();

    // 按优先级顺序的关键文件名（manifest.json 需进一步判断 addons 字段）
    const PRIORITY: &[&str] = &[
        "mcbbs.packmeta",
        "mmc-pack.json",
        "modrinth.index.json",
        "manifest.json",
        "modpack.json",
    ];

    // 第一遍：扫描根目录（路径不含 /），按优先级顺序查找
    for key in PRIORITY {
        for &(i, ref name) in &entry_names {
            if name.contains('/') {
                continue;
            }
            if name == *key {
                if let Some(detected) = try_detect_at_root(archive, i, name, "")? {
                    return Ok(detected);
                }
            }
        }
    }

    // 第二遍：扫描一级子目录（路径形如 "subfolder/关键文件"），按优先级顺序查找
    for key in PRIORITY {
        for &(i, ref name) in &entry_names {
            let parts: Vec<&str> = name.split('/').collect();
            if parts.len() != 2 {
                continue;
            }
            if parts[1] == *key {
                let base = format!("{}/", parts[0]);
                if let Some(detected) = try_detect_at_root(archive, i, parts[1], &base)? {
                    return Ok(detected);
                }
            }
        }
    }

    // 第三遍：扫描根目录/一级子目录的 `modpack.zip` / `modpack.mrpack` → LauncherPack
    for &(i, ref name) in &entry_names {
        let base = if let Some(stripped) = name.strip_prefix("modpack.zip") {
            if stripped.is_empty() {
                ""
            } else {
                continue;
            }
        } else if let Some(stripped) = name.strip_prefix("modpack.mrpack") {
            if stripped.is_empty() {
                ""
            } else {
                continue;
            }
        } else {
            continue;
        };
        let _ = i;
        log_info!("[Community] 检测到带启动器整合包（内嵌 {}）", name);
        return Ok(DetectedModpack {
            format: super::super::types::ModpackFormat::LauncherPack,
            archive_base_folder: base.to_string(),
            manifest_content: None,
            index_content: None,
            hmcl_content: None,
            mmc_content: None,
            mmc_cfg_content: None,
            launcher_inner_path: Some(name.clone()),
        });
    }
    for &(i, ref name) in &entry_names {
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() != 2 {
            continue;
        }
        if parts[1] == "modpack.zip" || parts[1] == "modpack.mrpack" {
            let _ = i;
            let base = format!("{}/", parts[0]);
            log_info!("[Community] 检测到带启动器整合包（内嵌 {}）", name);
            return Ok(DetectedModpack {
                format: super::super::types::ModpackFormat::LauncherPack,
                archive_base_folder: base,
                manifest_content: None,
                index_content: None,
                hmcl_content: None,
                mmc_content: None,
                mmc_cfg_content: None,
                launcher_inner_path: Some(name.clone()),
            });
        }
    }

    // 第四遍：Compress 兜底，扫描 `.minecraft/` 目录前缀
    let mut minecraft_prefix: Option<String> = None;
    for &(_, ref name) in &entry_names {
        if let Some(rest) = name.strip_prefix(".minecraft/") {
            if !rest.is_empty() {
                minecraft_prefix = Some(".minecraft/".to_string());
                break;
            }
        }
        if let Some(rest) = name.strip_prefix("/.minecraft/") {
            if !rest.is_empty() {
                minecraft_prefix = Some("/.minecraft/".to_string());
                break;
            }
        }
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() >= 2 && parts[0] != ".minecraft" {
            if parts.len() >= 3 && parts[1] == ".minecraft" {
                let prefix = format!("{}/.minecraft/", parts[0]);
                minecraft_prefix = Some(prefix);
                break;
            }
        }
    }
    if let Some(prefix) = minecraft_prefix {
        log_info!(
            "[Community] 检测到普通压缩包整合包（.minecraft 前缀: {}）",
            prefix
        );
        return Ok(DetectedModpack {
            format: super::super::types::ModpackFormat::Compress,
            archive_base_folder: prefix,
            manifest_content: None,
            index_content: None,
            hmcl_content: None,
            mmc_content: None,
            mmc_cfg_content: None,
            launcher_inner_path: None,
        });
    }

    Err("无法识别的整合包格式：未找到 manifest.json / modrinth.index.json / modpack.json / mmc-pack.json / mcbbs.packmeta / modpack.zip / .minecraft/ 目录".to_string())
}

/// 尝试在指定 base_folder 下识别关键文件
///
/// `entry_index` 为关键文件在 zip 中的索引，`entry_name` 为关键文件名（不含 base_folder 前缀）。
/// 命中返回 `Some(DetectedModpack)`，否则返回 `None`。
fn try_detect_at_root(
    archive: &mut zip::ZipArchive<std::fs::File>,
    entry_index: usize,
    entry_name: &str,
    base_folder: &str,
) -> Result<Option<DetectedModpack>, String> {
    use super::super::types::ModpackFormat;
    use std::io::Read;

    if entry_name == "mcbbs.packmeta" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 mcbbs.packmeta 失败: {}", e))?;
        log_info!("[Community] 检测到 MCBBS 整合包（mcbbs.packmeta）");
        return Ok(Some(DetectedModpack {
            format: ModpackFormat::Mcbbs,
            archive_base_folder: base_folder.to_string(),
            manifest_content: Some(s),
            index_content: None,
            hmcl_content: None,
            mmc_content: None,
            mmc_cfg_content: None,
            launcher_inner_path: None,
        }));
    }

    if entry_name == "mmc-pack.json" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 mmc-pack.json 失败: {}", e))?;
        log_info!("[Community] 检测到 MMC 整合包（mmc-pack.json）");

        // 顺带读取同 base_folder 下的 instance.cfg（用于配置迁移）
        let cfg_path = format!("{}instance.cfg", base_folder);
        let mut mmc_cfg_content: Option<String> = None;
        for i in 0..archive.len() {
            let Ok(mut entry) = archive.by_index(i) else {
                continue;
            };
            if entry.name() == cfg_path {
                let mut cfg = String::new();
                if entry.read_to_string(&mut cfg).is_ok() {
                    mmc_cfg_content = Some(cfg);
                }
                break;
            }
        }
        if mmc_cfg_content.is_some() {
            log_info!("[Community] MMC instance.cfg 已加载，将迁移配置到 setup.ini");
        }

        return Ok(Some(DetectedModpack {
            format: ModpackFormat::Mmc,
            archive_base_folder: base_folder.to_string(),
            manifest_content: None,
            index_content: None,
            hmcl_content: None,
            mmc_content: Some(s),
            mmc_cfg_content,
            launcher_inner_path: None,
        }));
    }

    if entry_name == "modrinth.index.json" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 modrinth.index.json 失败: {}", e))?;
        log_info!("[Community] 检测到 Modrinth 整合包（modrinth.index.json）");
        return Ok(Some(DetectedModpack {
            format: ModpackFormat::Modrinth,
            archive_base_folder: base_folder.to_string(),
            manifest_content: None,
            index_content: Some(s),
            hmcl_content: None,
            mmc_content: None,
            mmc_cfg_content: None,
            launcher_inner_path: None,
        }));
    }

    if entry_name == "manifest.json" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
        // 有 addons 字段 → MCBBS，无 → CurseForge
        let has_addons = serde_json::from_str::<serde_json::Value>(&s)
            .ok()
            .and_then(|v| v.get("addons").map(|a| !a.is_null()))
            .unwrap_or(false);
        if has_addons {
            log_info!("[Community] 检测到 MCBBS 整合包（manifest.json 含 addons）");
            Ok(Some(DetectedModpack {
                format: ModpackFormat::Mcbbs,
                archive_base_folder: base_folder.to_string(),
                manifest_content: Some(s),
                index_content: None,
                hmcl_content: None,
                mmc_content: None,
                mmc_cfg_content: None,
                launcher_inner_path: None,
            }))
        } else {
            log_info!("[Community] 检测到 CurseForge 整合包（manifest.json）");
            Ok(Some(DetectedModpack {
                format: ModpackFormat::Curseforge,
                archive_base_folder: base_folder.to_string(),
                manifest_content: Some(s),
                index_content: None,
                hmcl_content: None,
                mmc_content: None,
                mmc_cfg_content: None,
                launcher_inner_path: None,
            }))
        }
    } else if entry_name == "modpack.json" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 modpack.json 失败: {}", e))?;
        log_info!("[Community] 检测到 HMCL 整合包（modpack.json）");
        Ok(Some(DetectedModpack {
            format: ModpackFormat::Hmcl,
            archive_base_folder: base_folder.to_string(),
            manifest_content: None,
            index_content: None,
            hmcl_content: Some(s),
            mmc_content: None,
            mmc_cfg_content: None,
            launcher_inner_path: None,
        }))
    } else {
        Ok(None)
    }
}
