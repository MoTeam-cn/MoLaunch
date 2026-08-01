//! CurseForge 指纹查询：fingerprint_search / fingerprint_search_with_downloads
//!
//! - `fingerprint_search`：按 MurmurHash2 指纹批量查询工程详情（整合包安装 mod 解析）
//! - `fingerprint_search_with_downloads`：批量查询文件下载信息（导出整合包专用，省一次 API 调用）

use serde::Deserialize;

use super::super::types::{FileDownloadInfo, ResourceProject, ResourceType};
use super::convert::convert_project;
use super::http::cf_post;
use super::types::CfFile;

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
        data: Vec<super::types::CfModEntry>,
    }
    let mods_resp: ModsResp = cf_post("/mods", body2).await?;

    // 构建 modId → ResourceProject 映射
    let mut modid_to_project: std::collections::HashMap<i64, ResourceProject> =
        std::collections::HashMap::new();
    for entry in &mods_resp.data {
        let project = convert_project(entry, ResourceType::Mod);
        super::super::cache::set_project("CF", &entry.id.to_string(), &project);
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

/// 按 MurmurHash2 指纹批量查询 CurseForge 文件下载信息（导出整合包专用）
///
/// 与 `fingerprint_search` 不同，本函数不查询工程详情（省一次 API 调用），
/// 直接从 `/v1/fingerprints/432` 响应的 `exactMatches[i].file` 中提取
/// `downloadUrl` / `fileLength`，用于直接写入 `modrinth.index.json` 的 files 数组。
///
/// CF API 不返回 SHA1/SHA512，由调用方（导出模块）本地计算后填入。
///
/// 返回 `fingerprint → FileDownloadInfo` 映射（未查到的不在 map 中）。
pub async fn fingerprint_search_with_downloads(
    fingerprints: Vec<u32>,
) -> Result<std::collections::HashMap<u32, FileDownloadInfo>, String> {
    if fingerprints.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // POST /v1/fingerprints/432 请求体：{"fingerprints": [...]}
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
        /// CF project id（modId），用于导出 CF 格式整合包时写入 manifest.files[].projectID
        #[serde(default)]
        id: Option<i64>,
        #[serde(default)]
        file: Option<CfFile>,
    }

    let resp: FingerprintResp = cf_post("/fingerprints/432", body).await?;
    crate::log_info!(
        "[Community] CF fingerprint (with downloads) 查询命中 {} / {} 个",
        resp.data.exact_matches.len(),
        fingerprints.len()
    );

    if resp.data.exact_matches.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // 构建 fingerprint → FileDownloadInfo 映射
    // 用 exactMatches[i].file.fileFingerprint 反查本地指纹
    let mut result: std::collections::HashMap<u32, FileDownloadInfo> =
        std::collections::HashMap::new();
    for m in &resp.data.exact_matches {
        let Some(file) = m.file.as_ref() else {
            continue;
        };
        let Some(download_url) = file.download_url.as_ref() else {
            continue;
        };
        if download_url.is_empty() {
            continue;
        }
        // CF 不返回 SHA1/SHA512，由调用方本地计算后填入
        // project_id 来自 exactMatches[i].id（即 modId），file_id 来自 file.id
        result.insert(
            file.file_fingerprint,
            FileDownloadInfo {
                download_url: download_url.clone(),
                file_size: file.file_length,
                sha1: String::new(),
                sha512: None,
                project_id: m.id,
                file_id: Some(file.id),
            },
        );
    }

    crate::log_info!(
        "[Community] CF fingerprint (with downloads) 完成：{} 个文件获取到下载地址",
        result.len()
    );

    Ok(result)
}
