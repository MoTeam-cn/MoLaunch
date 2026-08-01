//! CurseForge 搜索：search + curseforge_loader_type
//!
//! search 按关键词/分类/加载器/版本查询工程列表，分页大小 40，按下载量降序。

use super::super::common::urlencode_params;
use super::super::types::{ResourceProject, ResourceType};
use super::convert::convert_project;
use super::http::cf_get;
use super::types::CfSearchResponse;

/// CurseForge 搜索
pub async fn search(
    query: &str,
    rtype: ResourceType,
    game_version: Option<&str>,
    mod_loader: u32,
    category: Option<&str>,
    page: u32,
) -> Result<(Vec<ResourceProject>, u32), String> {
    let class_id = rtype.curseforge_class_id();
    let index = page * 40;

    let mut params = vec![
        ("gameId", "432".to_string()),
        ("classId", class_id.to_string()),
        ("sortField", "2".to_string()), // 按下载量
        ("sortOrder", "desc".to_string()),
        ("pageSize", "40".to_string()),
        ("index", index.to_string()),
    ];

    if !query.is_empty() {
        params.push(("searchFilter", query.to_string()));
    }
    if let Some(v) = game_version {
        if !v.is_empty() {
            params.push(("gameVersion", v.to_string()));
        }
    }
    if mod_loader > 0 {
        // CurseForge modLoaderType 参数
        if let Some(ml) = curseforge_loader_type(mod_loader) {
            params.push(("modLoaderType", ml.to_string()));
        }
    }
    if let Some(c) = category {
        if !c.is_empty() {
            params.push(("categoryId", c.to_string()));
        }
    }

    let path = format!("/mods/search?{}", urlencode_params(&params));
    let resp: CfSearchResponse = cf_get(&path).await?;

    let total = resp.pagination.total_count;
    let projects = resp
        .data
        .iter()
        .map(|e| convert_project(e, rtype))
        .collect();

    Ok((projects, total))
}

/// CurseForge modLoaderType 参数值
fn curseforge_loader_type(flags: u32) -> Option<u32> {
    // CurseForge modLoaderType: 1=Forge, 2=Cauldron, 3=LiteLoader, 4=Fabric, 5=Quilt, 6=NeoForge
    if flags & super::super::types::ModLoaders::FORGE != 0 {
        Some(1)
    } else if flags & super::super::types::ModLoaders::NEOFORGE != 0 {
        Some(6)
    } else if flags & super::super::types::ModLoaders::FABRIC != 0 {
        Some(4)
    } else if flags & super::super::types::ModLoaders::QUILT != 0 {
        Some(5)
    } else if flags & super::super::types::ModLoaders::LITELOADER != 0 {
        Some(3)
    } else {
        None
    }
}
