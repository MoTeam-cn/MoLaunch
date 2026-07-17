//! CurseForge 响应到统一资源模型的转换
//!
//! 包含：
//! - convert_project：CF 工程条目 → ResourceProject
//! - convert_version：CF 文件 → ResourceVersion
//! - parse_cf_download_url：构造 CF 下载 URL（参考 PCL2 ParseCurseForgeDownloadUrls）

use super::super::mcmod::lookup_cf;
use super::super::tags::translate_curseforge_tag;
use super::super::types::{
    ModLoaders, Platform, ReleaseType, ResourceProject, ResourceType, ResourceVersion,
};
use super::types::{CfFile, CfModEntry};

/// 将 CurseForge 工程条目转换为统一 ResourceProject
pub(crate) fn convert_project(entry: &CfModEntry, rtype: ResourceType) -> ResourceProject {
    let mod_loaders = entry
        .latest_files
        .iter()
        .map(|f| f.game_versions.iter().map(|v| ModLoaders::from_str(v)).fold(0u32, |a, b| a | b))
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
            // 优先用 ID 翻译；翻译不了就保留原 name（参考 PCL2 ResourceProject.vb:199-274）
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
        // mcmod.cn 中文译名（参考 PCL2 ResourceProject.TranslatedName）
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
pub(crate) fn convert_version(file: &CfFile) -> ResourceVersion {
    let mod_loaders = file
        .game_versions
        .iter()
        .map(|v| ModLoaders::from_str(v))
        .fold(0u32, |a, b| a | b);

    let game_versions = file
        .game_versions
        .iter()
        .filter(|v| v.contains('.') || v.contains("w"))
        .cloned()
        .collect();

    let hash = file
        .hashes
        .iter()
        .find(|h| h.algo == 1)
        .or_else(|| file.hashes.first())
        .map(|h| h.value.clone());

    let download_url = parse_cf_download_url(&file.download_url, &file.file_name, file.id);

    ResourceVersion {
        id: file.id.to_string(),
        display: file.display_name.clone(),
        version: String::new(), // CurseForge 无版本号字段
        release_date: file.file_date.clone(),
        download_count: file.download_count,
        mod_loaders,
        game_versions,
        release_type: ReleaseType::from_curseforge(file.release_type),
        file_name: file.file_name.clone(),
        download_url,
        hash,
        size: file.file_length,
        dependencies: Vec::new(),
    }
}

/// 构造 CurseForge 下载 URL（参考 PCL2 ParseCurseForgeDownloadUrls）
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
