//! 各关键文件格式识别规则（try_detect_at_root）

use std::io::Read;

use crate::log_info;

use super::super::super::types::ModpackFormat;
use super::super::DetectedModpack;

/// 尝试在指定 base_folder 下识别关键文件
///
/// `entry_index` 为关键文件在 zip 中的索引，`entry_name` 为关键文件名（不含 base_folder 前缀）。
/// 命中返回 `Some(DetectedModpack)`，否则返回 `None`。
pub(super) fn try_detect_at_root(
    archive: &mut zip::ZipArchive<std::fs::File>,
    entry_index: usize,
    entry_name: &str,
    base_folder: &str,
) -> Result<Option<DetectedModpack>, String> {
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
