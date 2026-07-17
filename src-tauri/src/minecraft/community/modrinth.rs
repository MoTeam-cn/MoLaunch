//! Modrinth API 客户端
//!
//! 参考 PCL2 ResourceSearcher.GetModrinthAddress
//! API 文档: https://docs.modrinth.com/

use serde::Deserialize;

use super::common::fmt_elapsed;
use super::types::{
    ModLoaders, Platform, ReleaseType, ResourceProject, ResourceVersion, ResourceType,
};

/// Modrinth 官方 API 基地址
const MR_OFFICIAL_BASE: &str = "https://api.modrinth.com/v2";

/// Modrinth 镜像 API 基地址（MCIM 镜像源）
const MR_MIRROR_BASE: &str = "https://mod.mcimirror.top/modrinth/v2";

/// 根据 source 策略选择 Modrinth 基地址
///
/// source 策略（参考 PCL2 ToolDownloadMod）：
/// - 0=尽量镜像：强制走镜像
/// - 1=缓慢时换镜像：优先官方，失败后由调用方回退镜像
/// - 2=尽量官方：强制走官方
fn pick_base() -> (&'static str, u8) {
    let source = super::get_source_pref();
    match source {
        0 => (MR_MIRROR_BASE, source),
        2 => (MR_OFFICIAL_BASE, source),
        // 1=缓慢时换镜像：优先官方
        _ => (MR_OFFICIAL_BASE, source),
    }
}

/// Modrinth 搜索响应
#[derive(Debug, Deserialize)]
struct MrSearchResponse {
    hits: Vec<MrHit>,
    total_hits: u32,
}

/// Modrinth 搜索命中
#[derive(Debug, Deserialize)]
struct MrHit {
    #[serde(default)]
    project_id: String,
    #[serde(default)]
    slug: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    downloads: u64,
    #[serde(default)]
    icon_url: Option<String>,
    #[serde(default)]
    project_type: String,
    #[serde(default)]
    versions: Vec<String>,
    #[serde(default)]
    date_modified: Option<String>,
    #[serde(default)]
    display: Option<String>,
}

/// Modrinth 工程详情
#[derive(Debug, Deserialize)]
struct MrProject {
    id: String,
    slug: Option<String>,
    title: String,
    description: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    downloads: u64,
    icon_url: Option<String>,
    project_type: String,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
    updated: Option<String>,
}

/// Modrinth 版本
#[derive(Debug, Deserialize)]
struct MrVersion {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    version_number: String,
    #[serde(default)]
    date_published: String,
    #[serde(default)]
    downloads: u64,
    #[serde(default)]
    version_type: String,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
    #[serde(default)]
    files: Vec<MrFile>,
    #[serde(default)]
    dependencies: Vec<MrDependency>,
}

#[derive(Debug, Deserialize)]
struct MrFile {
    #[serde(default)]
    url: String,
    filename: Option<String>,
    primary: Option<bool>,
    size: Option<u64>,
    hashes: Option<MrHashes>,
}

#[derive(Debug, Deserialize)]
struct MrHashes {
    sha1: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MrDependency {
    project_id: Option<String>,
    dependency_type: Option<String>,
}

/// 将 Modrinth hit 转换为统一 ResourceProject
fn convert_hit(hit: &MrHit, rtype: ResourceType) -> ResourceProject {
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
        hit.project_type,
        hit.slug
    );

    // 分类标签中文化（参考 PCL2 ResourceProject.vb:310-378）
    let tags: Vec<String> = hit
        .categories
        .iter()
        .filter_map(|c| {
            // 先尝试翻译，翻译不了就保留原文（但过滤掉加载器标签，加载器单独显示）
            match super::tags::translate_modrinth_tag(c) {
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
        // mcmod.cn 中文译名（参考 PCL2 ResourceProject.TranslatedName）
        translated_name: super::mcmod::lookup_mr(&hit.slug)
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
fn convert_project(p: &MrProject, rtype: ResourceType) -> ResourceProject {
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

    // mcmod.cn 中文译名（参考 PCL2 ResourceProject.TranslatedName）
    let translated_name = super::mcmod::lookup_mr(&slug)
        .unwrap_or_default()
        .to_string();

    // 分类标签中文化（参考 PCL2 ResourceProject.vb:310-378）
    let tags: Vec<String> = p
        .categories
        .iter()
        .filter_map(|c| match super::tags::translate_modrinth_tag(c) {
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
fn convert_version(v: &MrVersion) -> ResourceVersion {
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
/// ignore_quilt=true 时过滤 Quilt 加载器（参考 PCL2 ToolDownloadIgnoreQuilt）
fn build_facets(
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
    let ignore_quilt = super::get_ignore_quilt();
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

/// 发送 GET 请求
///
/// source=1（缓慢时换镜像）时，若官方请求失败/超时，自动回退到镜像源重试
/// source=2 时官方请求不设超时（让其自然完成），失败直接报错
/// source=0 时直接走镜像
async fn mr_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let (base, source) = pick_base();
    let url = format!("{}{}", base, path);

    // 官方请求超时（参考 PCL2 DlModRequest：Modrinth 官方默认 20s）
    const MR_OFFICIAL_TIMEOUT_SECS: u64 = 20;

    let is_official = base == MR_OFFICIAL_BASE;
    let start = std::time::Instant::now();

    /// 解析响应：404 单独处理（避免空 body 触发 "EOF while parsing" 警告混淆）
    async fn parse_resp<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
        url: &str,
    ) -> Result<T, String> {
        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            // 404 视为正常 "未找到"，记 INFO 不报警告（用户查不到 mod 不算异常）
            crate::log_info!("[Community] MR 资源不存在 (404): {}", url);
            return Err(format!("Modrinth 资源不存在: {}", url));
        }
        if !status.is_success() {
            let code = status.as_u16();
            crate::log_warn!("[Community] MR 响应非 2xx: {} ({})", url, code);
            return Err(format!("Modrinth 响应异常: HTTP {}", code));
        }
        resp.json::<T>().await.map_err(|e| {
            crate::log_warn!("[Community] MR 响应解析失败: {} ({})", url, e);
            format!("Modrinth 响应解析失败: {}", e)
        })
    }

    let result = if is_official {
        // source=1 时官方请求加超时，超时触发回退；source=2 时不加超时
        if source == 1 {
            match tokio::time::timeout(
                std::time::Duration::from_secs(MR_OFFICIAL_TIMEOUT_SECS),
                crate::http::get_client().get(&url).send(),
            )
            .await
            {
                Ok(Ok(resp)) => parse_resp::<T>(resp, &url).await,
                Ok(Err(e)) => {
                    crate::log_warn!("[Community] MR 请求失败: {} ({:?})", url, e);
                    Err(format!("Modrinth 请求失败: {}", e))
                }
                Err(_) => {
                    crate::log_warn!(
                        "[Community] MR 官方请求超时（{}s），回退镜像",
                        MR_OFFICIAL_TIMEOUT_SECS
                    );
                    Err(format!("Modrinth 官方请求超时（{}s）", MR_OFFICIAL_TIMEOUT_SECS))
                }
            }
        } else {
            // source=2：官方请求不加超时
            let resp = crate::http::get_client()
                .get(&url)
                .send()
                .await
                .map_err(|e| {
                    crate::log_warn!("[Community] MR 请求失败: {} ({:?})", url, e);
                    format!("Modrinth 请求失败: {}", e)
                })?;
            parse_resp::<T>(resp, &url).await
        }
    } else {
        // 镜像请求不加超时
        let resp = crate::http::get_client()
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] MR 请求失败: {} ({:?})", url, e);
                format!("Modrinth 请求失败: {}", e)
            })?;
        parse_resp::<T>(resp, &url).await
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] MR 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            // 策略 1（缓慢时换镜像）：官方失败时回退镜像
            // 但 404 表示资源真的不存在，重试镜像也无意义（镜像也是 404）
            let is_not_found = e.starts_with("Modrinth 资源不存在");
            if source == 1 && is_official && !is_not_found {
                crate::log_warn!("[Community] MR 官方请求失败，回退镜像: {}", e);
                let mirror_url = format!("{}{}", MR_MIRROR_BASE, path);
                let resp = crate::http::get_client()
                    .get(&mirror_url)
                    .send()
                    .await
                    .map_err(|e| {
                        crate::log_warn!("[Community] MR 镜像请求失败: {} ({:?})", mirror_url, e);
                        format!("Modrinth 镜像请求失败: {}", e)
                    })?;
                let value: T = parse_resp::<T>(resp, &mirror_url).await?;
                crate::log_info!("[Community] MR 镜像请求成功: {} ({})", mirror_url, fmt_elapsed(start));
                return Ok(value);
            }
            Err(e)
        }
    }
}

/// 发送 POST 请求（参考 PCL2 `DlModRequest` 对 MR POST 接口的处理）
///
/// 与 `mr_get` 一致的 source 策略和 404 处理。
/// 用于 `/v2/version_files` 批量按 hash 查询本地 mod 对应的工程。
async fn mr_post<T: serde::de::DeserializeOwned>(
    path: &str,
    body: String,
) -> Result<T, String> {
    let (base, source) = pick_base();
    let url = format!("{}{}", base, path);

    const MR_OFFICIAL_TIMEOUT_SECS: u64 = 20;
    let is_official = base == MR_OFFICIAL_BASE;
    let start = std::time::Instant::now();

    /// 解析 POST 响应（复用 mr_get 的 404 优雅处理逻辑）
    async fn parse_post_resp<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
        url: &str,
    ) -> Result<T, String> {
        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            crate::log_info!("[Community] MR POST 资源不存在 (404): {}", url);
            return Err(format!("Modrinth 资源不存在: {}", url));
        }
        if !status.is_success() {
            let code = status.as_u16();
            crate::log_warn!("[Community] MR POST 响应非 2xx: {} ({})", url, code);
            return Err(format!("Modrinth 响应异常: HTTP {}", code));
        }
        resp.json::<T>().await.map_err(|e| {
            crate::log_warn!("[Community] MR POST 响应解析失败: {} ({})", url, e);
            format!("Modrinth 响应解析失败: {}", e)
        })
    }

    let result = if is_official && source == 1 {
        // source=1：官方请求加超时
        match tokio::time::timeout(
            std::time::Duration::from_secs(MR_OFFICIAL_TIMEOUT_SECS),
            crate::http::get_client()
                .post(&url)
                .header("Content-Type", "application/json")
                .body(body.clone())
                .send(),
        )
        .await
        {
            Ok(Ok(resp)) => parse_post_resp::<T>(resp, &url).await,
            Ok(Err(e)) => {
                crate::log_warn!("[Community] MR POST 请求失败: {} ({:?})", url, e);
                Err(format!("Modrinth 请求失败: {}", e))
            }
            Err(_) => {
                crate::log_warn!(
                    "[Community] MR POST 官方请求超时（{}s），回退镜像",
                    MR_OFFICIAL_TIMEOUT_SECS
                );
                Err(format!(
                    "Modrinth 官方请求超时（{}s）",
                    MR_OFFICIAL_TIMEOUT_SECS
                ))
            }
        }
    } else {
        // source=2 官方不加超时 / source=0 直接镜像
        let resp = crate::http::get_client()
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body.clone())
            .send()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] MR POST 请求失败: {} ({:?})", url, e);
                format!("Modrinth 请求失败: {}", e)
            })?;
        parse_post_resp::<T>(resp, &url).await
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] MR POST 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            let is_not_found = e.starts_with("Modrinth 资源不存在");
            if source == 1 && is_official && !is_not_found {
                crate::log_warn!("[Community] MR POST 官方请求失败，回退镜像: {}", e);
                let mirror_url = format!("{}{}", MR_MIRROR_BASE, path);
                let resp = crate::http::get_client()
                    .post(&mirror_url)
                    .header("Content-Type", "application/json")
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| {
                        crate::log_warn!(
                            "[Community] MR POST 镜像请求失败: {} ({:?})",
                            mirror_url,
                            e
                        );
                        format!("Modrinth 镜像请求失败: {}", e)
                    })?;
                let value: T = parse_post_resp::<T>(resp, &mirror_url).await?;
                crate::log_info!(
                    "[Community] MR POST 镜像请求成功: {} ({})",
                    mirror_url,
                    fmt_elapsed(start)
                );
                return Ok(value);
            }
            Err(e)
        }
    }
}

/// 按 SHA1 批量查询 Modrinth 工程详情
///
/// 参考 PCL2 `LocalResourceOnlineLoad` 步骤 1-3：
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
        // 校验 file.hashes.sha1 与查询的 sha1 一致（防 MR 返回错位，参考 PCL2 第 148 行）
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

/// 简单 URL 编码参数列表
fn urlencode_params(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}
