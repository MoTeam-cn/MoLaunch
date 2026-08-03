//! Modrinth SHA1 批量文件查询
//!
//! 按 hash 反查文件对应的工程/下载信息（`/v2/version_files`）。

use super::super::types::{FileDownloadInfo, ResourceProject, ResourceType};
use super::convert::convert_project;
use super::http::{mr_get, mr_post};
use super::types::{MrFile, MrProject};

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

    #[derive(serde::Deserialize)]
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
        super::super::cache::set_project("MR", &p.id, &project);
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

    #[derive(serde::Deserialize)]
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
