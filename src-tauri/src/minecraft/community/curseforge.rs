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

use super::common::fmt_elapsed;

use super::types::{
    Platform, ReleaseType, ResourceProject, ResourceVersion, ResourceType,
};

/// CurseForge 官方 API 基地址
const CF_OFFICIAL_BASE: &str = "https://api.curseforge.com/v1";

/// CurseForge 镜像 API 基地址（MCIM 镜像源）
const CF_MIRROR_BASE: &str = "https://mod.mcimirror.top/curseforge/v1";

/// 读取 CurseForge 配置，返回 (base_url, api_key, source_pref)
///
/// source 策略（参考 PCL2 ToolDownloadMod）：
/// - 0=尽量镜像：强制走镜像（即使配置了 API Key 也不用）
/// - 1=缓慢时换镜像：优先官方（若有 API Key），失败后由 cf_get 回退镜像
/// - 2=尽量官方：优先官方（若有 API Key），否则镜像
///
/// 异步：首次调用会触发 SDK DES 解密 api_key 并缓存，后续直接读缓存
async fn get_cf_config() -> (String, Option<String>, u8) {
    let (enabled, api_key) = super::secure_storage::get_config_async().await;
    let source = super::get_source_pref();

    // 0=尽量镜像：强制走镜像（忽略 API Key 配置）
    if source == 0 {
        crate::log_debug!("[Community] CF 走镜像源（source=0 强制镜像）");
        return (CF_MIRROR_BASE.to_string(), None, source);
    }

    // 1=缓慢时换镜像 / 2=尽量官方：有 API Key 则走官方
    if enabled {
        if let Some(ref key) = api_key {
            if !key.is_empty() {
                crate::log_debug!("[Community] CF 走官方 API（source={}, API Key 已配置）", source);
                return (CF_OFFICIAL_BASE.to_string(), api_key, source);
            }
        }
        crate::log_warn!("[Community] CF 已启用 API Key 但未配置 key，回退到镜像");
    }
    (CF_MIRROR_BASE.to_string(), None, source)
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
    /// 文件的 MurmurHash2 指纹（与请求 /fingerprints/432 时传入的指纹一致）
    ///
    /// 参考 PCL2 `Project("file")("fileFingerprint")`：用于反查 exactMatches[i] 对应哪个本地指纹。
    /// 注意：CF 的 fileFingerprint 是 uint32 number，不是字符串。
    #[serde(default)]
    file_fingerprint: u32,
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
///
/// source=1（缓慢时换镜像）时，若官方请求失败/超时，自动回退到镜像源重试
async fn cf_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let (base, api_key, source) = get_cf_config().await;
    let url = format!("{}{}", base, path);

    // 官方请求超时（参考 PCL2 DlModRequest：CF 官方默认 10s）
    const CF_OFFICIAL_TIMEOUT_SECS: u64 = 10;

    let is_official = base == CF_OFFICIAL_BASE;
    let start = Instant::now();

    let result = if is_official {
        // 官方请求加超时，超时视为"缓慢"，触发回退
        let req = build_cf_request(&url, api_key.as_deref());
        match tokio::time::timeout(
            std::time::Duration::from_secs(CF_OFFICIAL_TIMEOUT_SECS),
            req.send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp.json::<T>().await.map_err(|e| {
                crate::log_warn!("[Community] CF 响应解析失败: {} ({})", url, e);
                format!("CurseForge 响应解析失败: {}", e)
            }),
            Ok(Err(e)) => {
                crate::log_warn!("[Community] CF 请求失败: {} ({:?})", url, e);
                Err(format!("CurseForge 请求失败: {}", e))
            }
            Err(_) => {
                crate::log_warn!(
                    "[Community] CF 官方请求超时（{}s），{}",
                    CF_OFFICIAL_TIMEOUT_SECS,
                    if source == 1 { "回退镜像" } else { "报错" }
                );
                Err(format!("CurseForge 官方请求超时（{}s）", CF_OFFICIAL_TIMEOUT_SECS))
            }
        }
    } else {
        // 镜像请求不加超时（镜像本身可能较慢，让其自然完成）
        let req = build_cf_request(&url, api_key.as_deref());
        req.send().await
            .map_err(|e| {
                crate::log_warn!("[Community] CF 请求失败: {} ({:?})", url, e);
                format!("CurseForge 请求失败: {}", e)
            })?
            .json::<T>()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] CF 响应解析失败: {} ({})", url, e);
                format!("CurseForge 响应解析失败: {}", e)
            })
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] CF 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            // 策略 1（缓慢时换镜像）：官方失败时回退镜像
            if source == 1 && is_official {
                crate::log_warn!("[Community] CF 官方请求失败，回退镜像: {}", e);
                let mirror_url = format!("{}{}", CF_MIRROR_BASE, path);
                let req = build_cf_request(&mirror_url, None);
                let resp = req.send().await
                    .map_err(|e| {
                        crate::log_warn!("[Community] CF 镜像请求失败: {} ({:?})", mirror_url, e);
                        format!("CurseForge 镜像请求失败: {}", e)
                    })?;
                let value: T = resp.json().await
                    .map_err(|e| {
                        crate::log_warn!("[Community] CF 镜像响应解析失败: {} ({})", mirror_url, e);
                        format!("CurseForge 镜像响应解析失败: {}", e)
                    })?;
                crate::log_info!("[Community] CF 镜像请求成功: {} ({})", mirror_url, fmt_elapsed(start));
                return Ok(value);
            }
            Err(e)
        }
    }
}

/// 构造 CF GET 请求（附加 API Key header）
fn build_cf_request(url: &str, api_key: Option<&str>) -> reqwest::RequestBuilder {
    let mut req = crate::http::get_client().get(url);
    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
        req = req.header("Accept", "application/json");
    }
    req
}

/// 构造 CF POST 请求（附加 API Key header + JSON body）
fn build_cf_post_request(
    url: &str,
    api_key: Option<&str>,
    body: String,
) -> reqwest::RequestBuilder {
    let mut req = crate::http::get_client()
        .post(url)
        .header("Content-Type", "application/json")
        .body(body);
    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
        req = req.header("Accept", "application/json");
    }
    req
}

/// 发送 POST 请求（参考 PCL2 DlModRequest 对 CF POST 接口的处理）
///
/// 与 `cf_get` 一致的 source 策略：
/// - source=1 时官方失败回退镜像
/// - source=0 强制镜像
/// - source=2 强制官方
async fn cf_post<T: serde::de::DeserializeOwned>(
    path: &str,
    body: String,
) -> Result<T, String> {
    let (base, api_key, source) = get_cf_config().await;
    let url = format!("{}{}", base, path);

    const CF_OFFICIAL_TIMEOUT_SECS: u64 = 15;
    let is_official = base == CF_OFFICIAL_BASE;
    let start = Instant::now();

    let result = if is_official {
        let req = build_cf_post_request(&url, api_key.as_deref(), body.clone());
        match tokio::time::timeout(
            std::time::Duration::from_secs(CF_OFFICIAL_TIMEOUT_SECS),
            req.send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp.json::<T>().await.map_err(|e| {
                crate::log_warn!("[Community] CF POST 响应解析失败: {} ({})", url, e);
                format!("CurseForge 响应解析失败: {}", e)
            }),
            Ok(Err(e)) => {
                crate::log_warn!("[Community] CF POST 请求失败: {} ({:?})", url, e);
                Err(format!("CurseForge 请求失败: {}", e))
            }
            Err(_) => {
                crate::log_warn!(
                    "[Community] CF POST 官方请求超时（{}s），{}",
                    CF_OFFICIAL_TIMEOUT_SECS,
                    if source == 1 { "回退镜像" } else { "报错" }
                );
                Err(format!(
                    "CurseForge 官方请求超时（{}s）",
                    CF_OFFICIAL_TIMEOUT_SECS
                ))
            }
        }
    } else {
        let req = build_cf_post_request(&url, api_key.as_deref(), body.clone());
        req.send()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] CF POST 请求失败: {} ({:?})", url, e);
                format!("CurseForge 请求失败: {}", e)
            })?
            .json::<T>()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] CF POST 响应解析失败: {} ({})", url, e);
                format!("CurseForge 响应解析失败: {}", e)
            })
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] CF POST 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            // 策略 1：官方失败时回退镜像
            if source == 1 && is_official {
                crate::log_warn!("[Community] CF POST 官方请求失败，回退镜像: {}", e);
                let mirror_url = format!("{}{}", CF_MIRROR_BASE, path);
                let req = build_cf_post_request(&mirror_url, None, body);
                let resp = req.send().await.map_err(|e| {
                    crate::log_warn!(
                        "[Community] CF POST 镜像请求失败: {} ({:?})",
                        mirror_url,
                        e
                    );
                    format!("CurseForge 镜像请求失败: {}", e)
                })?;
                let value: T = resp.json().await.map_err(|e| {
                    crate::log_warn!(
                        "[Community] CF POST 镜像响应解析失败: {} ({})",
                        mirror_url,
                        e
                    );
                    format!("CurseForge 镜像响应解析失败: {}", e)
                })?;
                crate::log_info!(
                    "[Community] CF POST 镜像请求成功: {} ({})",
                    mirror_url,
                    fmt_elapsed(start)
                );
                return Ok(value);
            }
            Err(e)
        }
    }
}

/// 按 MurmurHash2 指纹批量查询 CurseForge 工程详情
///
/// 参考 PCL2 `LocalResourceOnlineLoad` 步骤 1-3：
/// 1. POST `/v1/fingerprints/432` 用指纹查 modId 和文件信息
/// 2. 从响应提取所有 modId
/// 3. POST `/v1/mods` 批量查询工程详情
///
/// 返回 `fingerprint → ResourceProject` 映射（未查到的指纹不在 map 中）
pub async fn fingerprint_search(
    fingerprints: Vec<u32>,
    rtype: ResourceType,
) -> Result<std::collections::HashMap<u32, ResourceProject>, String> {
    if fingerprints.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let _ = rtype; // 预留：未来可能按 rtype 过滤

    // 步骤 1：POST /fingerprints/432 批量查询指纹
    let body = serde_json::json!({
        "fingerprints": fingerprints
    })
    .to_string();

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FingerprintResp {
        #[serde(default)]
        data: FingerprintData,
    }

    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct FingerprintData {
        #[serde(default)]
        exact_matches: Vec<ExactMatch>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ExactMatch {
        /// 匹配到的 modId（CF API 中 exactMatches[i].id 就是 modId）
        ///
        /// 参考 PCL2 `Project("id").ToString` 直接作为 ProjectId 用于 `POST /v1/mods` 查询。
        #[serde(default)]
        id: Option<i64>,
        /// 匹配到的文件详情（含 fileFingerprint 反查指纹）
        #[serde(default)]
        file: Option<CfFile>,
    }

    let resp: FingerprintResp = cf_post("/fingerprints/432", body).await?;
    crate::log_info!(
        "[Community] CF fingerprint 查询命中 {} / {} 个",
        resp.data.exact_matches.len(),
        fingerprints.len()
    );

    if resp.data.exact_matches.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // 步骤 2：收集所有 modId，构建 fingerprint → modId 映射
    //
    // 参考 PCL2 `LocalResourceLoaders.vb` 第 223-234 行：
    //   Dim ProjectId = Project("id").ToString                         ' modId
    //   Dim Hash As UInteger = Project("file")("fileFingerprint")     ' 指纹
    //
    // CF 返回的 exactMatches 中，每条的 `id` 是 modId，`file.fileFingerprint` 是
    // 本地文件的 MurmurHash2 指纹（与请求时传入的指纹一致），用它反查本地文件。
    let mut fp_to_modid: std::collections::HashMap<u32, i64> = std::collections::HashMap::new();
    let mut mod_ids: Vec<i64> = Vec::new();
    for m in &resp.data.exact_matches {
        if let (Some(id), Some(file)) = (m.id, m.file.as_ref()) {
            let fp = file.file_fingerprint;
            fp_to_modid.insert(fp, id);
            if !mod_ids.contains(&id) {
                mod_ids.push(id);
            }
        }
    }

    if mod_ids.is_empty() {
        crate::log_warn!(
            "[Community] CF fingerprint 响应中无法提取任何 modId（exactMatches={} 但 id/file 字段缺失）",
            resp.data.exact_matches.len()
        );
        return Ok(std::collections::HashMap::new());
    }

    // 步骤 3：POST /mods 批量查询工程详情
    let body2 = serde_json::json!({ "modIds": mod_ids }).to_string();
    #[derive(Deserialize)]
    struct ModsResp {
        #[serde(default)]
        data: Vec<CfModEntry>,
    }
    let mods_resp: ModsResp = cf_post("/mods", body2).await?;

    // 构建 modId → ResourceProject 映射
    let mut modid_to_project: std::collections::HashMap<i64, ResourceProject> =
        std::collections::HashMap::new();
    for entry in &mods_resp.data {
        let project = convert_project(entry, ResourceType::Mod);
        super::cache::set_project("CF", &entry.id.to_string(), &project);
        modid_to_project.insert(entry.id, project);
    }

    // 步骤 4：构建 fingerprint → ResourceProject 映射返回
    let mut result: std::collections::HashMap<u32, ResourceProject> = std::collections::HashMap::new();
    for (fp, mod_id) in &fp_to_modid {
        if let Some(project) = modid_to_project.get(mod_id) {
            result.insert(*fp, project.clone());
        }
    }

    crate::log_info!(
        "[Community] CF fingerprint 批量查询完成：{} 个工程 → {} 个本地文件",
        modid_to_project.len(),
        result.len()
    );

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

    // CF API /mods/<id> 只接受数字 modId。
    // 非数字（slug）走 search 接口：GET /mods/search?gameId=432&slug=<slug>
    let entry: CfModEntry = if project_id.chars().all(|c| c.is_ascii_digit()) {
        let path = format!("/mods/{}", project_id);
        #[derive(Deserialize)]
        struct Resp { data: CfModEntry }
        cf_get::<Resp>(&path).await?.data
    } else {
        let slug_encoded = urlencoding::encode(project_id);
        let path = format!("/mods/search?gameId=432&slug={}", slug_encoded);
        let resp: CfSearchResponse = cf_get(&path).await?;
        resp.data.into_iter().next().ok_or_else(|| {
            format!("CurseForge 未找到 slug={} 的 mod", project_id)
        })?
    };

    let project = convert_project(&entry, rtype);
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

/// 批量查询 mod 工程信息，返回 `modId → slug` 映射
///
/// 用于整合包安装时按 `community_filename_format` 重命名 mod 文件：
/// manifest 提供 project_id 列表 → 调 `GET /v1/mods?modIds=...` 批量查询 →
/// 拿到每个 mod 的 slug → 查 mcmod 译名 → 应用文件名格式。
///
/// 失败时返回空 map（不阻断下载，只是文件名不应用格式）。
pub async fn batch_get_mod_slugs(mod_ids: &[i64]) -> std::collections::HashMap<i64, String> {
    if mod_ids.is_empty() {
        return std::collections::HashMap::new();
    }

    let ids_query = mod_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let path = format!("/mods?modIds={}", ids_query);

    #[derive(Deserialize)]
    struct Resp {
        data: Vec<CfModEntry>,
    }

    match cf_get::<Resp>(&path).await {
        Ok(resp) => {
            let map: std::collections::HashMap<i64, String> = resp
                .data
                .into_iter()
                .filter_map(|e| e.slug.map(|s| (e.id, s)))
                .collect();
            crate::log_info!("[Community] CF 批量查询 mod info 成功: {} 条", map.len());
            map
        }
        Err(e) => {
            crate::log_warn!("[Community] CF 批量查询 mod info 失败: {}", e);
            std::collections::HashMap::new()
        }
    }
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
