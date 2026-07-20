//! Modrinth 响应到统一资源模型的转换
//!
//! 包含：
//! - convert_hit：MR 搜索命中 → ResourceProject
//! - convert_project：MR 工程详情 → ResourceProject
//! - convert_version：MR 版本 → ResourceVersion
//! - build_facets：构建 MR facets 查询参数（参考 Modrinth API 文档）

use super::super::types::{
    ModLoaders, Platform, ReleaseType, ResourceProject, ResourceType, ResourceVersion,
};
use super::types::{MrHit, MrProject, MrVersion};

/// 将 Modrinth hit 转换为统一 ResourceProject
pub(crate) fn convert_hit(hit: &MrHit, rtype: ResourceType) -> ResourceProject {
    let mod_loaders = hit
        .categories
        .iter()
        .map(|c| ModLoaders::from_str(c))
        .fold(0u32, |a, b| a | b);

    let game_versions: Vec<String> = hit
        .versions
        .iter()
        .filter(|v| v.contains('.') || v.contains("w"))
        .cloned()
        .collect();

    let website = format!(
        "https://modrinth.com/{}/{}",
        hit.project_type, hit.slug
    );

    // 分类标签中文化
    let tags: Vec<String> = hit
        .categories
        .iter()
        .filter_map(|c| {
            // 先尝试翻译，翻译不了就保留原文（但过滤掉加载器标签，加载器单独显示）
            match super::super::tags::translate_modrinth_tag(c) {
                Some(label) => Some(label.to_string()),
                None => {
                    // 加载器标签不放入 tags
                    if matches!(c.as_str(), "fabric" | "forge" | "neoforge" | "quilt" | "liteloader") {
                        None
                    } else {
                        Some(c.clone())
                    }
                }
            }
        })
        .collect();

    ResourceProject {
        platform: Platform::Modrinth,
        resource_type: rtype,
        id: hit.project_id.clone(),
        slug: hit.slug.clone(),
        raw_name: if let Some(ref d) = hit.display {
            if !d.is_empty() {
                d.clone()
            } else {
                hit.title.clone()
            }
        } else {
            hit.title.clone()
        },
        // mcmod.cn 中文译名
        translated_name: super::super::mcmod::lookup_mr(&hit.slug)
            .unwrap_or_default()
            .to_string(),
        description: hit.description.clone(),
        website,
        last_update: hit.date_modified.clone().unwrap_or_default(),
        download_count: hit.downloads,
        mod_loaders,
        tags,
        logo_url: hit.icon_url.clone(),
        game_versions,
    }
}

/// 将 Modrinth project 转换为统一 ResourceProject
pub(crate) fn convert_project(p: &MrProject, rtype: ResourceType) -> ResourceProject {
    let mut mod_loaders = p
        .loaders
        .iter()
        .map(|l| ModLoaders::from_str(l))
        .fold(0u32, |a, b| a | b);
    mod_loaders |= p
        .categories
        .iter()
        .map(|c| ModLoaders::from_str(c))
        .fold(0u32, |a, b| a | b);

    let game_versions: Vec<String> = p
        .game_versions
        .iter()
        .filter(|v| v.contains('.') || v.contains("w"))
        .cloned()
        .collect();

    let slug = p.slug.clone().unwrap_or_default();
    let website = format!(
        "https://modrinth.com/{}/{}",
        p.project_type, slug
    );

    // mcmod.cn 中文译名
    let translated_name = super::super::mcmod::lookup_mr(&slug)
        .unwrap_or_default()
        .to_string();

    // 分类标签中文化
    let tags: Vec<String> = p
        .categories
        .iter()
        .filter_map(|c| match super::super::tags::translate_modrinth_tag(c) {
            Some(label) => Some(label.to_string()),
            None => {
                if matches!(c.as_str(), "fabric" | "forge" | "neoforge" | "quilt" | "liteloader") {
                    None
                } else {
                    Some(c.clone())
                }
            }
        })
        .collect();

    ResourceProject {
        platform: Platform::Modrinth,
        resource_type: rtype,
        id: p.id.clone(),
        slug,
        raw_name: p.title.clone(),
        translated_name,
        description: p.description.clone().unwrap_or_default(),
        website,
        last_update: p.updated.clone().unwrap_or_default(),
        download_count: p.downloads,
        mod_loaders,
        tags,
        logo_url: p.icon_url.clone(),
        game_versions,
    }
}

/// 将 Modrinth version 转换为统一 ResourceVersion
pub(crate) fn convert_version(v: &MrVersion) -> ResourceVersion {
    let mod_loaders = v
        .loaders
        .iter()
        .map(|l| ModLoaders::from_str(l))
        .fold(0u32, |a, b| a | b);

    let game_versions: Vec<String> = v
        .game_versions
        .iter()
        .filter(|gv| gv.contains('.') || gv.contains("w"))
        .cloned()
        .collect();

    // 取 primary 文件，没有则取第一个
    let file = v
        .files
        .iter()
        .find(|f| f.primary.unwrap_or(false))
        .or_else(|| v.files.first());

    let (file_name, download_url, hash, size) = if let Some(f) = file {
        (
            f.filename.clone().unwrap_or_default(),
            f.url.clone(),
            f.hashes.as_ref().and_then(|h| h.sha1.clone()),
            f.size.unwrap_or(0),
        )
    } else {
        (String::new(), String::new(), None, 0)
    };

    let dependencies: Vec<String> = v
        .dependencies
        .iter()
        .filter(|d| d.dependency_type.as_deref() == Some("required"))
        .filter_map(|d| d.project_id.clone())
        .collect();

    ResourceVersion {
        id: v.id.clone(),
        display: v.name.clone(),
        version: v.version_number.clone(),
        release_date: v.date_published.clone(),
        download_count: v.downloads,
        mod_loaders,
        game_versions,
        release_type: ReleaseType::from_modrinth(&v.version_type),
        file_name,
        download_url,
        hash,
        size,
        dependencies,
    }
}

/// 构建 Modrinth facets 参数
/// 格式: [["project_type:mod"],["categories:'forge'"],["versions:'1.20.1'"]]
///
/// ignore_quilt=true 时过滤 Quilt 加载器
pub(crate) fn build_facets(
    rtype: ResourceType,
    game_version: Option<&str>,
    mod_loader: u32,
    category: Option<&str>,
) -> String {
    let mut facets: Vec<Vec<String>> = Vec::new();

    // project_type
    facets.push(vec![format!("project_type:{}", rtype.modrinth_project_type())]);

    // category
    if let Some(c) = category {
        if !c.is_empty() {
            facets.push(vec![format!("categories:'{}'", c)]);
        }
    }

    // mod_loader (OR 组合)
    // 读取 ignore_quilt 配置，true 时从查询条件中移除 Quilt
    let ignore_quilt = super::super::get_ignore_quilt();
    let mut loaders = Vec::new();
    if mod_loader & ModLoaders::FORGE != 0 { loaders.push("categories:'forge'".to_string()); }
    if mod_loader & ModLoaders::NEOFORGE != 0 { loaders.push("categories:'neoforge'".to_string()); }
    if mod_loader & ModLoaders::FABRIC != 0 { loaders.push("categories:'fabric'".to_string()); }
    if !ignore_quilt && mod_loader & ModLoaders::QUILT != 0 { loaders.push("categories:'quilt'".to_string()); }
    if mod_loader & ModLoaders::LITELOADER != 0 { loaders.push("categories:'liteloader'".to_string()); }
    if !loaders.is_empty() {
        facets.push(loaders);
    }

    // game_version
    if let Some(v) = game_version {
        if !v.is_empty() {
            facets.push(vec![format!("versions:'{}'", v)]);
        }
    }

    serde_json::to_string(&facets).unwrap_or_else(|_| "[]".to_string())
}
