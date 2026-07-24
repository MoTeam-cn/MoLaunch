//! CurseForge API 客户端
//!
//! API 文档: https://docs.curseforge.com/
//!
//! 支持两种模式：
//! - 镜像模式（默认）：走 MCIM 镜像源，无需 API Key
//! - 官方模式：用户配置 API Key 后走官方 API，速度更快且支持完整字段
//!
//! 模块结构：
//! - types.rs: CF API 响应数据结构（CfModEntry / CfFile 等）
//! - convert.rs: CF 响应到统一资源模型的转换（convert_project / convert_version / parse_cf_download_url）
//! - http.rs: HTTP 请求层（cf_get / cf_post + source 策略回退）
//! - mod.rs: 公共 API（search / get_project / get_versions / fingerprint_search / batch_get_mod_slugs）

mod convert;
pub(crate) mod http;
mod types;

use serde::Deserialize;

use super::common::urlencode_params;
use super::types::{ResourceProject, ResourceType, ResourceVersion};
use convert::{convert_project, convert_version};
use http::{cf_get, cf_post};
use types::{CfFile, CfFilesResponse, CfModEntry, CfSearchResponse};

/// 按 MurmurHash2 指纹批量查询 CurseForge 工程详情
///
/// 步骤 1-3：
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
        /// 直接作为 ProjectId 用于 `POST /v1/mods` 查询。
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
    let mut result: std::collections::HashMap<u32, ResourceProject> =
        std::collections::HashMap::new();
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
    let projects = resp
        .data
        .iter()
        .map(|e| convert_project(e, rtype))
        .collect();

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
        struct Resp {
            data: CfModEntry,
        }
        cf_get::<Resp>(&path).await?.data
    } else {
        let slug_encoded = urlencoding::encode(project_id);
        let path = format!("/mods/search?gameId=432&slug={}", slug_encoded);
        let resp: CfSearchResponse = cf_get(&path).await?;
        resp.data
            .into_iter()
            .next()
            .ok_or_else(|| format!("CurseForge 未找到 slug={} 的 mod", project_id))?
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
/// manifest 提供 project_id 列表 → 调 `POST /v1/mods`（请求体 `{"modIds":[...]}`）
/// 批量查询 → 拿到每个 mod 的 slug → 查 mcmod 译名 → 应用文件名格式。
///
/// CF 官方 API `GET /v1/mods?modIds=...` 对 modIds 参数有数量限制（超 50 个会返回
/// 空响应，body 为空导致 EOF 解析失败）。改用 `POST /v1/mods` 与 fingerprint_search
/// 一致，请求体 `{"modIds":[...]}`，与 `POST /v1/mods/files` 同属 CF 官方推荐的
/// 批量查询接口，支持大批量 ID。仍按 50 个一批分批查询，避免单次请求体过大。
///
/// 失败时返回空 map（不阻断下载，只是文件名不应用格式）。
pub async fn batch_get_mod_slugs(mod_ids: &[i64]) -> std::collections::HashMap<i64, String> {
    if mod_ids.is_empty() {
        return std::collections::HashMap::new();
    }

    // CF POST /mods 支持大批量 ID（官方推荐批量查询接口），
    // 按 250 个一批分批查询，平衡请求数与单次请求体大小
    const BATCH_SIZE: usize = 250;
    let mut map: std::collections::HashMap<i64, String> = std::collections::HashMap::new();

    #[derive(Deserialize)]
    struct Resp {
        #[serde(default)]
        data: Vec<CfModEntry>,
    }

    for chunk in mod_ids.chunks(BATCH_SIZE) {
        // POST /v1/mods 请求体：{"modIds": [284754, 427597, ...]}
        let body = serde_json::json!({ "modIds": chunk }).to_string();

        match cf_post::<Resp>("/mods", body).await {
            Ok(resp) => {
                for e in resp.data {
                    if let Some(s) = e.slug {
                        map.insert(e.id, s);
                    }
                }
            }
            Err(e) => {
                crate::log_warn!(
                    "[Community] CF 批量查询 mod info 部分失败 ({} 个): {}",
                    chunk.len(),
                    e
                );
            }
        }
    }

    crate::log_info!("[Community] CF 批量查询 mod info 完成: {} 条", map.len());
    map
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
