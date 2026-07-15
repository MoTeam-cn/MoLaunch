//! CurseForge API 客户端
//!
//! 参考 PCL2 ResourceSearcher.GetCurseForgeAddress
//! API 文档: https://docs.curseforge.com/
//!
//! 支持两种模式：
//! - 镜像模式（默认）：走 MCIM 镜像源，无需 API Key
//! - 官方模式：用户配置 API Key 后走官方 API，速度更快且支持完整字段

use serde::Deserialize;
use std::time::Instant;

use super::types::{
    Platform, ReleaseType, ResourceProject, ResourceVersion, ResourceType,
};

/// CurseForge 官方 API 基地址
const CF_OFFICIAL_BASE: &str = "https://api.curseforge.com/v1";

/// CurseForge 镜像 API 基地址（MCIM 镜像源）
const CF_MIRROR_BASE: &str = "https://mod.mcimirror.top/curseforge/v1";

/// 读取 CurseForge 配置，返回 (base_url, api_key)
/// 启用且配置了 API Key 时走官方 API，否则走镜像
///
/// 异步：首次调用会触发 SDK DES 解密 api_key 并缓存，后续直接读缓存
async fn get_cf_config() -> (String, Option<String>) {
    let (enabled, api_key) = super::secure_storage::get_config_async().await;
    if enabled {
        if let Some(ref key) = api_key {
            if !key.is_empty() {
                crate::log_debug!("[Community] CF 使用官方 API（API Key 已配置）");
                return (CF_OFFICIAL_BASE.to_string(), api_key);
            }
        }
        crate::log_warn!("[Community] CF 已启用 API Key 但未配置 key，回退到镜像");
    }
    (CF_MIRROR_BASE.to_string(), None)
}

/// CurseForge 搜索响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfSearchResponse {
    data: Vec<CfModEntry>,
    pagination: CfPagination,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfPagination {
    total_count: u32,
}

/// CurseForge 工程条目（搜索结果和详情共用）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfModEntry {
    id: i64,
    slug: Option<String>,
    name: String,
    summary: Option<String>,
    #[serde(default)]
    download_count: u64,
    #[serde(default)]
    date_released: String,
    #[serde(default)]
    logo: Option<CfLogo>,
    #[serde(default)]
    latest_files: Vec<CfFile>,
    links: Option<CfLinks>,
    #[serde(default)]
    categories: Vec<CfCategory>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfLogo {
    #[serde(default)]
    thumbnail_url: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfLinks {
    #[serde(default)]
    website_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfCategory {
    #[serde(default)]
    id: Option<u32>,
    #[serde(default)]
    name: Option<String>,
}

/// CurseForge 文件
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfFile {
    id: i64,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    file_date: String,
    #[serde(default)]
    download_count: u64,
    #[serde(default)]
    release_type: u32,
    #[serde(default)]
    download_url: Option<String>,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    hashes: Vec<CfHash>,
    #[serde(default)]
    file_length: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfHash {
    #[serde(default)]
    algo: u32, // 1=SHA1, 2=MD5
    value: String,
}

/// CurseForge 版本列表响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfFilesResponse {
    data: Vec<CfFile>,
}

/// 将 CurseForge 工程条目转换为统一 ResourceProject
fn convert_project(entry: &CfModEntry, rtype: ResourceType) -> ResourceProject {
    let mod_loaders = entry
        .latest_files
        .iter()
        .map(|f| f.game_versions.iter().map(|v| super::types::ModLoaders::from_str(v)).fold(0u32, |a, b| a | b))
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
                if let Some(label) = super::tags::translate_curseforge_tag(id) {
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
            .and_then(|s| super::mcmod::lookup_cf(s))
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
fn convert_version(file: &CfFile) -> ResourceVersion {
    let mod_loaders = file
        .game_versions
        .iter()
        .map(|v| super::types::ModLoaders::from_str(v))
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
fn parse_cf_download_url(url: &Option<String>, file_name: &str, file_id: i64) -> String {
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

/// 发送 GET 请求并附加 API Key header（如果配置了）
async fn cf_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let (base, api_key) = get_cf_config().await;
    let url = format!("{}{}", base, path);
    let mut req = crate::http::get_client().get(&url);
    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
        req = req.header("Accept", "application/json");
    }
    let start = Instant::now();
    let resp = req
        .send()
        .await
        .map_err(|e| {
            crate::log_warn!("[Community] CF 请求失败: {} ({:?})", url, e);
            format!("CurseForge 请求失败: {}", e)
        })?;
    let result: T = resp
        .json()
        .await
        .map_err(|e| {
            crate::log_warn!("[Community] CF 响应解析失败: {} ({})", url, e);
            format!("CurseForge 响应解析失败: {}", e)
        })?;
    crate::log_info!("[Community] CF 请求成功: {} ({})", url, fmt_elapsed(start));
    Ok(result)
}

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
    let projects = resp.data.iter().map(|e| convert_project(e, rtype)).collect();

    Ok((projects, total))
}

/// 获取工程详情
pub async fn get_project(project_id: &str, rtype: ResourceType) -> Result<ResourceProject, String> {
    // 检查缓存
    if let Some(cached) = super::cache::get_project("CF", project_id) {
        crate::log_info!("[Community] CF 工程详情命中缓存: {}", project_id);
        return Ok(cached);
    }

    let path = format!("/mods/{}", project_id);

    #[derive(Deserialize)]
    struct Resp {
        data: CfModEntry,
    }

    let resp: Resp = cf_get(&path).await?;
    let project = convert_project(&resp.data, rtype);
    super::cache::set_project("CF", project_id, &project);
    Ok(project)
}

/// 获取工程版本列表
pub async fn get_versions(project_id: &str) -> Result<Vec<ResourceVersion>, String> {
    // 检查缓存
    if let Some(cached) = super::cache::get_versions("CF", project_id) {
        crate::log_info!("[Community] CF 版本列表命中缓存: {}", project_id);
        return Ok(cached);
    }

    let path = format!("/mods/{}/files?pageSize=10000", project_id);

    let resp: CfFilesResponse = cf_get(&path).await?;
    let versions: Vec<ResourceVersion> = resp.data.iter().map(convert_version).collect();
    super::cache::set_versions("CF", project_id, &versions);
    Ok(versions)
}

/// CurseForge modLoaderType 参数值
fn curseforge_loader_type(flags: u32) -> Option<u32> {
    // CurseForge modLoaderType: 1=Forge, 2=Cauldron, 3=LiteLoader, 4=Fabric, 5=Quilt, 6=NeoForge
    if flags & super::types::ModLoaders::FORGE != 0 {
        Some(1)
    } else if flags & super::types::ModLoaders::NEOFORGE != 0 {
        Some(6)
    } else if flags & super::types::ModLoaders::FABRIC != 0 {
        Some(4)
    } else if flags & super::types::ModLoaders::QUILT != 0 {
        Some(5)
    } else if flags & super::types::ModLoaders::LITELOADER != 0 {
        Some(3)
    } else {
        None
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

/// 格式化耗时
fn fmt_elapsed(start: Instant) -> String {
    let ms = start.elapsed().as_millis();
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}
