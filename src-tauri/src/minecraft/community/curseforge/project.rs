//! CurseForge 工程与版本查询：get_project / get_versions / batch_get_mod_slugs
//!
//! get_project / get_versions 命中 `super::super::cache` 时直接返回缓存，避免重复请求。
//! batch_get_mod_slugs 用 POST /mods 批量查询 modId → slug 映射，按 250 一批分页。

use serde::Deserialize;

use super::super::types::{ResourceProject, ResourceType, ResourceVersion};
use super::convert::{convert_project, convert_version};
use super::http::{cf_get, cf_post};
use super::types::{CfFilesResponse, CfModEntry, CfSearchResponse};

/// 获取工程详情
pub async fn get_project(project_id: &str, rtype: ResourceType) -> Result<ResourceProject, String> {
    // 检查缓存
    if let Some(cached) = super::super::cache::get_project("CF", project_id) {
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
    super::super::cache::set_project("CF", project_id, &project);
    Ok(project)
}

/// 获取工程版本列表
pub async fn get_versions(project_id: &str) -> Result<Vec<ResourceVersion>, String> {
    // 检查缓存
    if let Some(cached) = super::super::cache::get_versions("CF", project_id) {
        crate::log_info!("[Community] CF 版本列表命中缓存: {}", project_id);
        return Ok(cached);
    }

    let path = format!("/mods/{}/files?pageSize=10000", project_id);

    let resp: CfFilesResponse = cf_get(&path).await?;
    let versions: Vec<ResourceVersion> = resp.data.iter().map(convert_version).collect();
    super::super::cache::set_versions("CF", project_id, &versions);
    Ok(versions)
}

/// 批量查询 mod 工程信息，返回 `modId → slug` 映射
///
/// 整合包安装时按 `community_filename_format` 重命名：manifest 的 project_id →
/// `POST /v1/mods`（`{"modIds":[...]}`）批量查 slug → 查 mcmod 译名 → 应用文件名格式。
/// 用 POST 而非 GET（GET 对 modIds 数量有限制，超 50 返回空响应致 EOF）；仍按 50 一批分页。
/// 失败返回空 map（不阻断下载，仅文件名不应用格式）。
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
