//! Modrinth API 客户端
//!
//! 参考 PCL2 ResourceSearcher.GetModrinthAddress
//! API 文档: https://docs.modrinth.com/

use serde::Deserialize;

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

    let result = if is_official {
        // source=1 时官方请求加超时，超时触发回退；source=2 时不加超时
        if source == 1 {
            match tokio::time::timeout(
                std::time::Duration::from_secs(MR_OFFICIAL_TIMEOUT_SECS),
                crate::http::get_client().get(&url).send(),
            )
            .await
            {
                Ok(Ok(resp)) => resp.json::<T>().await.map_err(|e| {
                    crate::log_warn!("[Community] MR 响应解析失败: {} ({})", url, e);
                    format!("Modrinth 响应解析失败: {}", e)
                }),
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
            crate::http::get_client()
                .get(&url)
                .send()
                .await
                .map_err(|e| {
                    crate::log_warn!("[Community] MR 请求失败: {} ({:?})", url, e);
                    format!("Modrinth 请求失败: {}", e)
                })?
                .json::<T>()
                .await
                .map_err(|e| {
                    crate::log_warn!("[Community] MR 响应解析失败: {} ({})", url, e);
                    format!("Modrinth 响应解析失败: {}", e)
                })
        }
    } else {
        // 镜像请求不加超时
        crate::http::get_client()
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] MR 请求失败: {} ({:?})", url, e);
                format!("Modrinth 请求失败: {}", e)
            })?
            .json::<T>()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] MR 响应解析失败: {} ({})", url, e);
                format!("Modrinth 响应解析失败: {}", e)
            })
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] MR 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            // 策略 1（缓慢时换镜像）：官方失败时回退镜像
            if source == 1 && is_official {
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
                let value: T = resp.json().await
                    .map_err(|e| {
                        crate::log_warn!("[Community] MR 镜像响应解析失败: {} ({})", mirror_url, e);
                        format!("Modrinth 镜像响应解析失败: {}", e)
                    })?;
                crate::log_info!("[Community] MR 镜像请求成功: {} ({})", mirror_url, fmt_elapsed(start));
                return Ok(value);
            }
            Err(e)
        }
    }
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

/// 简单 URL 编码参数列表
fn urlencode_params(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// 格式化耗时
fn fmt_elapsed(start: std::time::Instant) -> String {
    let ms = start.elapsed().as_millis();
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}
