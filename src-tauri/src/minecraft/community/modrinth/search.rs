//! Modrinth 搜索与工程查询
//!
//! 搜索、工程详情/版本列表、Slug 批量直查等查询类 API。

use super::super::common::urlencode_params;
use super::super::types::{ResourceProject, ResourceType, ResourceVersion};
use super::convert::{build_facets, convert_hit, convert_project, convert_version};
use super::http::mr_get;
use super::types::{MrProject, MrSearchResponse, MrVersion};

/// Modrinth 搜索
pub async fn search(
    query: &str,
    rtype: ResourceType,
    game_version: Option<&str>,
    mod_loader: u32,
    category: Option<&str>,
    page: u32,
) -> Result<(Vec<ResourceProject>, u32), String> {
    let limit = 40u32;
    let offset = page * limit;
    let facets = build_facets(rtype, game_version, mod_loader, category);

    let mut params = vec![
        ("limit", limit.to_string()),
        ("offset", offset.to_string()),
        ("index", "relevance".to_string()),
        ("facets", facets),
    ];
    if !query.is_empty() {
        params.push(("query", query.to_string()));
    }

    let path = format!("/search?{}", urlencode_params(&params));
    let resp: MrSearchResponse = mr_get(&path).await?;

    let total = resp.total_hits;
    let projects = resp.hits.iter().map(|h| convert_hit(h, rtype)).collect();

    Ok((projects, total))
}

/// 获取工程详情
pub async fn get_project(project_id: &str, rtype: ResourceType) -> Result<ResourceProject, String> {
    // 检查缓存
    if let Some(cached) = super::super::cache::get_project("MR", project_id) {
        crate::log_info!("[Community] MR 工程详情命中缓存: {}", project_id);
        return Ok(cached);
    }

    let path = format!("/project/{}", project_id);
    let resp: MrProject = mr_get(&path).await?;
    let project = convert_project(&resp, rtype);
    super::super::cache::set_project("MR", project_id, &project);
    Ok(project)
}

/// 获取工程版本列表
pub async fn get_versions(project_id: &str) -> Result<Vec<ResourceVersion>, String> {
    // 检查缓存
    if let Some(cached) = super::super::cache::get_versions("MR", project_id) {
        crate::log_info!("[Community] MR 版本列表命中缓存: {}", project_id);
        return Ok(cached);
    }

    let path = format!("/project/{}/version", project_id);
    let resp: Vec<MrVersion> = mr_get(&path).await?;
    let versions: Vec<ResourceVersion> = resp.iter().map(convert_version).collect();
    super::super::cache::set_versions("MR", project_id, &versions);
    Ok(versions)
}

/// 通过 Slug 列表批量查询 Modrinth 工程（中文搜索专用）
///
/// `GET /v2/projects?ids=[...]` 接受 project_id，slug 可作别名传入实现批量拉取。
/// 中文搜索：moddata.txt 匹配出 MR slug 后调本函数绕过 MR 搜索 API 中文支持不佳。
///
/// - `slugs`：slug 列表（最多 100，超出截断）
/// - `rtype`：资源类型（填充 ResourceProject.resource_type）
/// - 返回：工程列表（失败返回空 Vec，不阻断搜索）
pub async fn get_projects_by_slugs(slugs: &[String], rtype: ResourceType) -> Vec<ResourceProject> {
    if slugs.is_empty() {
        return Vec::new();
    }

    // Modrinth /projects 限制单次最多 100 个 id，本地保护性截断
    let slugs_slice: Vec<&String> = slugs.iter().take(100).collect();
    let ids_json = serde_json::to_string(&slugs_slice).unwrap_or_else(|_| "[]".to_string());
    let encoded = urlencoding::encode(&ids_json).to_string();
    let path = format!("/projects?ids={}", encoded);

    match mr_get::<Vec<MrProject>>(&path).await {
        Ok(projects) => {
            let result: Vec<ResourceProject> = projects
                .iter()
                .map(|p| {
                    let project = convert_project(p, rtype);
                    super::super::cache::set_project("MR", &p.id, &project);
                    project
                })
                .collect();
            crate::log_info!(
                "[Community] MR Slug 直查成功: {} / {} 个",
                result.len(),
                slugs_slice.len()
            );
            result
        }
        Err(e) => {
            crate::log_warn!("[Community] MR Slug 直查失败: {}", e);
            Vec::new()
        }
    }
}

/// 批量查询 project 信息，返回 `project_id → slug` 映射
///
/// 用于整合包安装时按 `community_filename_format` 重命名 mod 文件：
/// 从 MR 整合包 files[].downloads URL 提取 project_id →
/// 调 `GET /projects?ids=[...]` 批量查询 → 拿 slug → 查 mcmod 译名 → 应用文件名格式。
///
/// 失败时返回空 map（不阻断下载，只是文件名不应用格式）。
pub async fn batch_get_project_slugs(
    project_ids: &[String],
) -> std::collections::HashMap<String, String> {
    if project_ids.is_empty() {
        return std::collections::HashMap::new();
    }

    // Modrinth API: GET /projects?ids=["id1","id2"]
    let ids_json = serde_json::to_string(project_ids).unwrap_or_else(|_| "[]".to_string());
    let encoded = urlencoding::encode(&ids_json).to_string();
    let path = format!("/projects?ids={}", encoded);

    match mr_get::<Vec<MrProject>>(&path).await {
        Ok(projects) => {
            let map: std::collections::HashMap<String, String> = projects
                .into_iter()
                .filter_map(|p| p.slug.map(|s| (p.id, s)))
                .collect();
            crate::log_info!("[Community] MR 批量查询 projects 成功: {} 条", map.len());
            map
        }
        Err(e) => {
            crate::log_warn!("[Community] MR 批量查询 projects 失败: {}", e);
            std::collections::HashMap::new()
        }
    }
}
