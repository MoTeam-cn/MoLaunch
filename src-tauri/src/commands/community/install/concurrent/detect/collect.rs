//! 整合包格式检测主流程（收集 zip 条目并按优先级扫描各层级）

use crate::log_info;

use super::super::DetectedModpack;
use super::super::super::types::ModpackFormat;
use super::rules::try_detect_at_root;

/// 检测整合包格式
///
/// 识别优先级：mcbbs.packmeta → mmc-pack.json → modrinth.index.json →
/// manifest.json（有 addons 为 Mcbbs，否则 Curseforge）→ modpack.json →
/// modpack.zip/mrpack（LauncherPack）→ .minecraft/ 目录（Compress）。
/// 根目录命中时 `archive_base_folder` 为 `""`，子目录命中时为 `"子目录/"`。
pub(crate) fn detect_modpack_format(
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
            format: ModpackFormat::LauncherPack,
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
                format: ModpackFormat::LauncherPack,
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
    for (_, name) in &entry_names {
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
        if parts.len() >= 2
            && parts[0] != ".minecraft"
            && parts.len() >= 3
            && parts[1] == ".minecraft"
        {
            let prefix = format!("{}/.minecraft/", parts[0]);
            minecraft_prefix = Some(prefix);
            break;
        }
    }
    if let Some(prefix) = minecraft_prefix {
        log_info!(
            "[Community] 检测到普通压缩包整合包（.minecraft 前缀: {}）",
            prefix
        );
        return Ok(DetectedModpack {
            format: ModpackFormat::Compress,
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