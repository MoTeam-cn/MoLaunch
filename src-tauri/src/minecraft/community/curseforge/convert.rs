//! CurseForge 响应到统一资源模型的转换

use super::super::mcmod::lookup_cf;
use super::super::tags::translate_curseforge_tag;
use super::super::types::{
    ModLoaders, Platform, ReleaseType, ResourceProject, ResourceType, ResourceVersion,
};
use super::types::{CfFile, CfModEntry};

/// CurseForge 依赖排除列表（平台基础库 ID）
///
/// Fabric API（306612）与 Quilt API（634179）通常作为平台基础库由加载器自动加载，
/// 无需在版本详情中重复提示用户安装，故从 required 依赖列表中排除。
const CF_EXCLUDED_DEPENDENCY_IDS: [i64; 2] = [306612, 634179];

/// 将 CurseForge 工程条目转换为统一 ResourceProject
pub(crate) fn convert_project(entry: &CfModEntry, rtype: ResourceType) -> ResourceProject {
    let mod_loaders = entry
        .latest_files
        .iter()
        .map(|f| {
            f.game_versions
                .iter()
                .map(|v| ModLoaders::from_str(v))
                .fold(0u32, |a, b| a | b)
        })
        .fold(0u32, |a, b| a | b);

    let game_versions = entry
        .latest_files
        .iter()
        .flat_map(|f| f.game_versions.iter().cloned())
        .filter(|v| v.contains('.') || v.contains("w"))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let logo_url = entry
        .logo
        .as_ref()
        .and_then(|l| l.thumbnail_url.clone().or_else(|| l.url.clone()));

    let website = entry
        .links
        .as_ref()
        .and_then(|l| l.website_url.clone())
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string();

    let tags = entry
        .categories
        .iter()
        .filter_map(|c| {
            // 优先用 ID 翻译；翻译不了就保留原 name
            if let Some(id) = c.id {
                if let Some(label) = translate_curseforge_tag(id) {
                    return Some(label.to_string());
                }
            }
            // 加载器标签不放入 tags（CF 用 name 区分）
            c.name.clone().filter(|n| !n.is_empty())
        })
        .collect();

    ResourceProject {
        platform: Platform::CurseForge,
        resource_type: rtype,
        id: entry.id.to_string(),
        slug: entry.slug.clone().unwrap_or_default(),
        raw_name: entry.name.clone(),
        // mcmod.cn 中文译名
        translated_name: entry
            .slug
            .as_ref()
            .and_then(|s| lookup_cf(s))
            .unwrap_or_default()
            .to_string(),
        description: entry.summary.clone().unwrap_or_default(),
        website,
        last_update: entry.date_released.clone(),
        download_count: entry.download_count,
        mod_loaders,
        tags,
        logo_url,
        game_versions,
    }
}

/// 将 CurseForge 文件转换为统一 ResourceVersion
///
/// 版本号 fallback：
/// CurseForge API 不直接提供 mod 版本号字段（`Version = Nothing`），
/// 用 `Display`（即 `displayName`）作为 fallback 进行版本对比。
/// 这里从 `display_name` 提取版本号，提取失败则用 `display_name` 本身。
pub(crate) fn convert_version(file: &CfFile, rtype: ResourceType) -> ResourceVersion {
    let mod_loaders = file
        .game_versions
        .iter()
        .map(|v| ModLoaders::from_str(v))
        .fold(0u32, |a, b| a | b);

    let mut game_versions: Vec<String> = file
        .game_versions
        .iter()
        .filter(|v| v.contains('.') || v.contains("w"))
        .cloned()
        .collect();

    // 整合包老文件的 game_versions 经常缺失（或只有 "Minecraft 1.12" 这类无点值被过滤掉），
    // 此时从 display_name 兜底提取 MC 版本号（如 "RLCraft 1.12.2 - Beta v2.8.1.zip" → 1.12.2）
    if rtype == ResourceType::ModPack && game_versions.is_empty() {
        let mc = crate::minecraft::community::version_extract::extract_mc_version_from_name(
            &file.display_name,
        );
        if !mc.is_empty() {
            game_versions.push(mc);
        }
    }

    let hash = file
        .hashes
        .iter()
        .find(|h| h.algo == 1)
        .or_else(|| file.hashes.first())
        .map(|h| h.value.clone());

    let download_url = parse_cf_download_url(&file.download_url, &file.file_name, file.id);

    // 版本号 fallback：从 display_name 提取
    // CurseForge 的 displayName 通常类似 "jei-1.20.1-15.2.0.27.jar"
    let version =
        crate::minecraft::community::version_extract::extract_version_from_name(&file.display_name);

    // 提取 required 依赖（relationType=3），排除平台基础库（见 CF_EXCLUDED_DEPENDENCY_IDS）
    let dependencies: Vec<String> = file
        .dependencies
        .iter()
        .filter(|d| d.relation_type == 3 && !CF_EXCLUDED_DEPENDENCY_IDS.contains(&d.mod_id))
        .map(|d| d.mod_id.to_string())
        .collect();

    ResourceVersion {
        id: file.id.to_string(),
        display: file.display_name.clone(),
        version,
        release_date: file.file_date.clone(),
        download_count: file.download_count,
        mod_loaders,
        game_versions,
        release_type: ReleaseType::from_curseforge(file.release_type),
        file_name: file.file_name.clone(),
        download_url,
        hash,
        size: file.file_length,
        dependencies,
    }
}

/// 构造 CurseForge 下载 URL
pub(crate) fn parse_cf_download_url(url: &Option<String>, file_name: &str, file_id: i64) -> String {
    if let Some(ref u) = url {
        if !u.is_empty() {
            return u.clone();
        }
    }
    // Fallback: 从 file_id 构造 edge.forgecdn.net URL
    let id_str = file_id.to_string();
    if id_str.len() >= 6 {
        let (p1, p2) = id_str.split_at(id_str.len() - 4);
        format!("https://edge.forgecdn.net/files/{}/{}", p1, p2)
    } else {
        format!("https://edge.forgecdn.net/files/0/{}", file_name)
    }
}
