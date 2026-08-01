//! Modrinth API 客户端（API 文档: https://docs.modrinth.com/）

mod convert;
mod http;
mod types;

use serde::Deserialize;

use super::common::urlencode_params;
use super::types::{FileDownloadInfo, ResourceProject, ResourceType, ResourceVersion};
use convert::{build_facets, convert_hit, convert_project, convert_version};
use http::{mr_get, mr_post};
use types::{MrFile, MrProject, MrSearchResponse, MrVersion};

/// 按 SHA1 批量查询 Modrinth 工程详情
///
/// 步骤 1-3：
/// 1. POST `/v2/version_files` 用 SHA1 查 version 和 project_id
/// 2. 收集所有 project_id
/// 3. GET `/v2/projects?ids=[...]` 批量查询工程详情
///
/// 返回 `sha1 → ResourceProject` 映射（未查到的不在 map 中）
pub async fn version_files_search(
    sha1s: Vec<String>,
    rtype: ResourceType,
) -> Result<std::collections::HashMap<String, ResourceProject>, String> {
    if sha1s.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let _ = rtype;

    // 步骤 1：POST /version_files 批量查询
    let body = serde_json::json!({
        "hashes": sha1s,
        "algorithm": "sha1"
    })
    .to_string();

    // 响应结构：{ "<sha1>": { "id": version_id, "project_id": "...", "files": [{ "hashes": { "sha1": "..." }}], ... } }
    type VersionFilesResp = std::collections::HashMap<String, MrVersionFileEntry>;

    #[derive(Deserialize)]
    struct MrVersionFileEntry {
        #[serde(default)]
        project_id: String,
        #[serde(default)]
        files: Vec<MrFile>,
    }

    let resp: VersionFilesResp = mr_post("/version_files", body).await?;
    crate::log_info!(
        "[Community] MR version_files 查询命中 {} / {} 个",
        resp.len(),
        sha1s.len()
    );

    if resp.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // 步骤 2：构建 sha1 → project_id 映射，收集所有 project_id
    let mut sha1_to_project_id: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut project_ids: Vec<String> = Vec::new();
    for (sha1, entry) in &resp {
        if entry.project_id.is_empty() {
            continue;
        }
        // 校验 file.hashes.sha1 与查询的 sha1 一致（防 MR 返回错位）
        let sha1_match = entry.files.iter().any(|f| {
            f.hashes
                .as_ref()
                .and_then(|h| h.sha1.as_deref())
                .map(|s| s == sha1)
                .unwrap_or(false)
        });
        if !sha1_match {
            continue;
        }
        sha1_to_project_id.insert(sha1.clone(), entry.project_id.clone());
        if !project_ids.contains(&entry.project_id) {
            project_ids.push(entry.project_id.clone());
        }
    }

    if project_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // 步骤 3：GET /projects?ids=[...] 批量查询工程详情
    let ids_json = serde_json::to_string(&project_ids).unwrap_or_else(|_| "[]".to_string());
    let encoded = urlencoding::encode(&ids_json).to_string();
    let path = format!("/projects?ids={}", encoded);

    let projects: Vec<MrProject> = match mr_get::<Vec<MrProject>>(&path).await {
        Ok(p) => p,
        Err(e) => {
            crate::log_warn!("[Community] MR 批量查询 projects 失败: {}", e);
            return Ok(std::collections::HashMap::new());
        }
    };

    // 构建 project_id → ResourceProject 映射
    let mut pid_to_project: std::collections::HashMap<String, ResourceProject> =
        std::collections::HashMap::new();
    for p in &projects {
        let project = convert_project(p, ResourceType::Mod);
        super::cache::set_project("MR", &p.id, &project);
        pid_to_project.insert(p.id.clone(), project);
    }

    // 步骤 4：构建 sha1 → ResourceProject 映射返回
    let mut result: std::collections::HashMap<String, ResourceProject> =
        std::collections::HashMap::new();
    for (sha1, pid) in &sha1_to_project_id {
        if let Some(project) = pid_to_project.get(pid) {
            result.insert(sha1.clone(), project.clone());
        }
    }

    crate::log_info!(
        "[Community] MR version_files 批量查询完成：{} 个工程 → {} 个本地文件",
        pid_to_project.len(),
        result.len()
    );

    Ok(result)
}

/// 按 SHA1 批量查询 Modrinth 文件下载信息（导出整合包专用）
///
/// 与 `version_files_search` 不同，本函数不查询工程详情（省一次 API 调用），
/// 直接从 `/v2/version_files` 响应中提取 `files[0]` 的下载 URL、大小、SHA1/SHA512，
/// 用于直接写入 `modrinth.index.json` 的 files 数组。
///
/// 返回 `sha1 → FileDownloadInfo` 映射（未查到的不在 map 中）。
pub async fn version_files_search_with_downloads(
    sha1s: Vec<String>,
) -> Result<std::collections::HashMap<String, FileDownloadInfo>, String> {
    if sha1s.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // POST /v2/version_files 请求体：{"hashes": [...], "algorithm": "sha1"}
    let body = serde_json::json!({
        "hashes": sha1s,
        "algorithm": "sha1"
    })
    .to_string();

    // 响应结构：{ "<sha1>": { "files": [{ "url": "...", "size": N, "hashes": { "sha1": "...", "sha512": "..." }}] } }
    type VersionFilesResp = std::collections::HashMap<String, MrVersionFileEntry>;

    #[derive(Deserialize)]
    struct MrVersionFileEntry {
        #[serde(default)]
        files: Vec<MrFile>,
    }

    let resp: VersionFilesResp = mr_post("/version_files", body).await?;
    crate::log_info!(
        "[Community] MR version_files (with downloads) 查询命中 {} / {} 个",
        resp.len(),
        sha1s.len()
    );

    if resp.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // 构建 sha1 → FileDownloadInfo 映射
    let mut result: std::collections::HashMap<String, FileDownloadInfo> =
        std::collections::HashMap::new();
    for (sha1, entry) in &resp {
        // 选取 primary 文件，若无 primary 则取第一个
        let file = entry
            .files
            .iter()
            .find(|f| f.primary.unwrap_or(false))
            .or_else(|| entry.files.first());
        let Some(file) = file else { continue };

        if file.url.is_empty() {
            continue;
        }

        // 校验 file.hashes.sha1 与查询的 sha1 一致（防 MR 返回错位）
        let file_sha1 = file
            .hashes
            .as_ref()
            .and_then(|h| h.sha1.as_deref())
            .unwrap_or("");
        if !file_sha1.is_empty() && file_sha1 != sha1 {
            continue;
        }

        result.insert(
            sha1.clone(),
            FileDownloadInfo {
                download_url: file.url.clone(),
                file_size: file.size.unwrap_or(0),
                sha1: sha1.clone(),
                sha512: file.hashes.as_ref().and_then(|h| h.sha512.clone()),
                // Modrinth 不使用 CF 的 project_id/file_id 概念
                project_id: None,
                file_id: None,
            },
        );
    }

    crate::log_info!(
        "[Community] MR version_files (with downloads) 完成：{} 个文件获取到下载地址",
        result.len()
    );

    Ok(result)
}

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
    if let Some(cached) = super::cache::get_project("MR", project_id) {
        crate::log_info!("[Community] MR 工程详情命中缓存: {}", project_id);
        return Ok(cached);
    }

    let path = format!("/project/{}", project_id);
    let resp: MrProject = mr_get(&path).await?;
    let project = convert_project(&resp, rtype);
    super::cache::set_project("MR", project_id, &project);
    Ok(project)
}

/// 获取工程版本列表
pub async fn get_versions(project_id: &str) -> Result<Vec<ResourceVersion>, String> {
    // 检查缓存
    if let Some(cached) = super::cache::get_versions("MR", project_id) {
        crate::log_info!("[Community] MR 版本列表命中缓存: {}", project_id);
        return Ok(cached);
    }

    let path = format!("/project/{}/version", project_id);
    let resp: Vec<MrVersion> = mr_get(&path).await?;
    let versions: Vec<ResourceVersion> = resp.iter().map(convert_version).collect();
    super::cache::set_versions("MR", project_id, &versions);
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
                    super::cache::set_project("MR", &p.id, &project);
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
