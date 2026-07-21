//! 社区资源搜索命令

use crate::minecraft::community::tags::get_categories as fetch_category_tags;
use crate::minecraft::community::{search, ResourceType, SearchParams, SearchResult};
use serde::{Deserialize, Serialize};

/// 前端搜索请求参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: String,
    pub resource_type: ResourceType,
    pub game_version: Option<String>,
    pub mod_loader: u32,
    /// 0=全部, 1=仅CurseForge, 2=仅Modrinth
    pub source: u32,
    /// 分类标签（"CFId/MrSlug" 格式）
    pub category: Option<String>,
    pub page: u32,
}

/// 分类标签响应
#[derive(Debug, Serialize)]
pub struct CategoryTagInfo {
    pub combined: String, // "CFId/MrSlug"
    pub label: String,
}

/// 搜索社区资源
#[tauri::command]
pub async fn search_resources(req: SearchRequest) -> Result<SearchResult, String> {
    // 解析 "CFId/MrSlug" 格式，根据来源选择对应 ID
    let category_id = req.category.as_ref().and_then(|c| {
        if c.is_empty() {
            return None;
        }
        let parts: Vec<&str> = c.split('/').collect();
        let cf_id = parts.first().copied().unwrap_or("");
        let mr_slug = parts.get(1).copied().unwrap_or("");
        // 1=仅CF, 2=仅MR, 其他=用 CF ID
        let id = if req.source == 2 {
            mr_slug.to_string()
        } else {
            cf_id.to_string()
        };
        if id.is_empty() {
            None
        } else {
            Some(id)
        }
    });

    let params = SearchParams {
        query: req.query,
        resource_type: req.resource_type,
        game_version: req.game_version,
        mod_loader: req.mod_loader,
        source: req.source,
        category: category_id,
        page: req.page,
    };

    search(params).await
}

/// 获取指定资源类型的分类标签列表
#[tauri::command]
pub async fn get_category_tags(
    resource_type: ResourceType,
) -> Result<Vec<CategoryTagInfo>, String> {
    let tags = fetch_category_tags(resource_type);
    Ok(tags
        .into_iter()
        .map(|t| CategoryTagInfo {
            combined: format!("{}/{}", t.curseforge_id, t.modrinth_slug),
            label: t.label.to_string(),
        })
        .collect())
}
