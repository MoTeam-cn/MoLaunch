//! 社区资源下载安装 - CurseForge 整合包处理
//!
//! 包含 CF manifest.json 数据结构与 install_cf_mods 安装流程。
//! install_cf_mods 流程：POST /v1/mods/files 批量查询下载信息 →
//! 批量查询 project slug（用于 mcmod 译名 + 应用 community_filename_format）→
//! 并发下载到 mods 目录。

use crate::log_info;
use crate::minecraft::community::secure_storage;
use crate::state::AppState;
use serde::Deserialize;

/// CF manifest.json 结构
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CfManifest {
    pub(super) minecraft: CfMinecraft,
    #[serde(default)]
    pub(super) files: Vec<CfManifestFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CfMinecraft {
    pub(super) version: String,
    #[serde(default)]
    pub(super) mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CfModLoader {
    pub(super) id: String,
    pub(super) primary: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // project_id / required 暂未参与依赖过滤，未来按 optional=false 跳过非必要 mod 时启用
pub(super) struct CfManifestFile {
    pub(super) project_id: i64,
    pub(super) file_id: i64,
    #[serde(default)]
    pub(super) required: bool,
}

/// POST /v1/mods/files 批量查询响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CfFilesBatchResponse {
    pub(super) data: Vec<CfFileEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(super) struct CfFileEntry {
    pub(super) file_id: i64,
    #[serde(default)]
    pub(super) file_name: String,
    #[serde(default)]
    pub(super) download_url: Option<String>,
    #[serde(default)]
    pub(super) file_length: u64,
}

/// 安装 CF 整合包依赖 mods
///
/// POST /v1/mods/files 批量查询所有 file_id 的下载信息，然后并发下载到 mods 目录。
/// 文件名按用户设置的 `community_filename_format` 重命名（查询 mod slug → mcmod 译名 → 应用格式）。
pub(super) async fn install_cf_mods(
    state: &AppState,
    manifest_files: &[CfManifestFile],
    mods_dir: &std::path::Path,
    max_threads: usize,
    _instance_dir: &std::path::Path,
) -> Result<(), String> {
    if manifest_files.is_empty() {
        log_info!("[Community] CF manifest 无依赖 mods");
        return Ok(());
    }

    // 1. 批量查询下载信息
    let file_ids: Vec<i64> = manifest_files.iter().map(|f| f.file_id).collect();
    log_info!("[Community] CF 批量查询 {} 个文件", file_ids.len());

    let (_enabled, _api_key) = secure_storage::get_config_async().await;
    // 通过 cf_post 走 source 策略（source=0 强制镜像，source=1 回退，source=2 官方）
    // cf_post 内部自动获取 API Key 并处理 HTTP 错误
    let batch: CfFilesBatchResponse = crate::minecraft::community::curseforge::http::cf_post(
        "/mods/files",
        serde_json::json!({ "fileIds": file_ids }).to_string(),
    )
    .await
    .map_err(|e| format!("CF 批量查询失败: {}", e))?;

    log_info!("[Community] CF 批量查询返回 {} 个文件", batch.data.len());

    // 2. 批量查询 mod info 拿 slug（用于查 mcmod 译名 + 应用 community_filename_format）
    //    manifest 提供 project_id 列表，调用 CF /mods?modIds=... 批量查询
    let project_ids: Vec<i64> = manifest_files.iter().map(|f| f.project_id).collect();
    let mod_slug_map =
        crate::minecraft::community::curseforge::batch_get_mod_slugs(&project_ids).await;

    // 3. 构造 file_id → 译名 映射（通过 manifest 关联 file_id 与 project_id）
    let mut file_translated: std::collections::HashMap<i64, Option<String>> =
        std::collections::HashMap::new();
    for mf in manifest_files {
        let slug = mod_slug_map.get(&mf.project_id);
        let translated = slug
            .and_then(|s| crate::minecraft::community::mcmod::lookup_cf(s).map(|n| n.to_string()));
        file_translated.insert(mf.file_id, translated);
    }

    // 4. 读取用户设置的文件名格式
    let filename_format = state.config.lock().await.community_filename_format;

    // 5. 构造下载列表（CF 通常只有一个 download_url，包装为单元素数组）
    let mut download_list: Vec<(Vec<String>, String, u64)> = Vec::with_capacity(batch.data.len());
    let mut total_bytes: u64 = 0;
    for entry in &batch.data {
        let primary_url = entry
            .download_url
            .clone()
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| {
                super::helpers::construct_cf_edge_url(entry.file_id, &entry.file_name)
            });
        // 构造官方 + 镜像双 URL 列表，交给 DownloadManager 自动 fallback
        let urls = crate::minecraft::sources::cdn_urls(&primary_url);
        // 应用 community_filename_format（无译名时 apply_filename_format 返回原名）
        let translated = file_translated.get(&entry.file_id).cloned().flatten();
        let final_name = super::helpers::apply_filename_format(
            &entry.file_name,
            translated.as_deref(),
            filename_format,
        );
        if final_name != entry.file_name {
            log_info!(
                "[Community] CF mod 文件名重命名: {} → {}",
                entry.file_name,
                final_name
            );
        }
        let target = mods_dir.join(&final_name);
        download_list.push((
            urls,
            target.to_string_lossy().to_string(),
            entry.file_length,
        ));
        total_bytes += entry.file_length;
    }

    log_info!(
        "[Community] CF 下载 {} 个文件，总大小 {}",
        download_list.len(),
        super::helpers::format_bytes(total_bytes)
    );

    // 6. 并发下载
    super::concurrent::download_files_concurrent(
        state,
        2,
        &download_list,
        max_threads,
        total_bytes,
    )
    .await?;

    log_info!("[Community] CF mods 下载完成 ({} 个)", download_list.len());
    Ok(())
}
