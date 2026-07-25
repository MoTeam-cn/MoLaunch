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
    /// 覆写目录名（默认空，表示 `overrides/`；可为 `"."` 或 `"./"` 表示根目录）
    #[serde(default)]
    pub(super) overrides: Option<String>,
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
pub(super) struct CfManifestFile {
    /// CurseForge project ID。
    /// CF 官方 manifest.json 用 `projectID`（大写 ID），部分第三方工具用 `projectId`（小写 id），
    /// 用 alias 兼容两者。None 时跳过 slug 查询与译名匹配。
    #[serde(default, alias = "projectID", alias = "projectId")]
    pub(super) project_id: Option<i64>,
    /// CurseForge file ID。
    /// CF 官方 manifest.json 用 `fileID`（大写 ID），部分第三方工具用 `fileId`（小写 id），
    /// 用 alias 兼容两者。None 时跳过该 mod。
    #[serde(default, alias = "fileID", alias = "fileId")]
    pub(super) file_id: Option<i64>,
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
    /// CF API /mods/files 返回的 file id 字段名是 `id`（不是 fileId）
    #[serde(rename = "id")]
    pub(super) file_id: i64,
    #[serde(default)]
    pub(super) file_name: String,
    #[serde(default)]
    pub(super) download_url: Option<String>,
    #[serde(default)]
    pub(super) file_length: u64,
    /// CF 文件 modules 数组（用于资源类型分流：mods / resourcepacks / shaderpacks）
    #[serde(default)]
    pub(super) modules: Vec<CfModule>,
}

/// CF 文件 module 条目
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CfModule {
    #[serde(default)]
    pub(super) name: String,
}

/// 安装 CF 整合包依赖 mods
///
/// POST /v1/mods/files 批量查询所有 file_id 的下载信息，然后并发下载到 mods 目录。
/// 文件名按用户设置的 `community_filename_format` 重命名（查询 mod slug → mcmod 译名 → 应用格式）。
///
/// 资源类型分流：
/// - modules 含 META-INF 或 mcmod.info → mods/（默认 mod）
/// - modules 含 pack.mcmeta → resourcepacks/
/// - 其他 → shaderpacks/
/// 无 modules 字段或判断失败时默认放 mods/。
pub(super) async fn install_cf_mods(
    state: &AppState,
    manifest_files: &[CfManifestFile],
    mods_dir: &std::path::Path,
    max_threads: usize,
    instance_dir: &std::path::Path,
    stage_index: usize,
    include_optional: bool,
) -> Result<(), String> {
    if manifest_files.is_empty() {
        log_info!("[Community] CF manifest 无依赖 mods");
        return Ok(());
    }

    // 按 required 字段过滤：required=false 为可选 Mod，由 include_optional 决定是否下载
    let effective_files: Vec<&CfManifestFile> = manifest_files
        .iter()
        .filter(|f| f.required || include_optional)
        .collect();
    let skipped_optional = manifest_files.len() - effective_files.len();
    if skipped_optional > 0 {
        log_info!(
            "[Community] CF 跳过 {} 个可选 Mod（required=false，用户未选择下载）",
            skipped_optional
        );
    }

    // 1. 批量查询下载信息
    // file_id 为 None 的项跳过
    let file_ids: Vec<i64> = effective_files.iter().filter_map(|f| f.file_id).collect();
    if file_ids.is_empty() {
        log_info!("[Community] CF manifest 无有效 file_id，跳过依赖下载");
        return Ok(());
    }
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

    // 镜像源可能不支持 /mods/files 批量查询，返回 200 但 data 为空。
    // 此时不能静默成功（否则整合包"安装完成"但 mods 目录为空），必须报错让用户知道。
    if batch.data.is_empty() {
        return Err(format!(
            "CF 批量查询返回 0 个文件（请求 {} 个）。可能是当前下载源（镜像）不支持 /mods/files 批量查询，请在「设置 → 下载」中将下载源切换为「缓慢时换镜像」或「尽量官方」后重试。",
            file_ids.len()
        ));
    }

    // 部分缺失检测：batch.data.len() < file_ids.len() 说明部分 file 已被作者删除
    // 构造缺失列表，反查 project_id 显示中文名
    if (batch.data.len() as usize) < file_ids.len() {
        let returned_ids: std::collections::HashSet<i64> =
            batch.data.iter().map(|e| e.file_id).collect();
        let missing: Vec<(i64, Option<i64>)> = file_ids
            .iter()
            .filter(|id| !returned_ids.contains(id))
            .map(|fid| {
                let pid = effective_files
                    .iter()
                    .find(|f| f.file_id == Some(*fid))
                    .and_then(|f| f.project_id);
                (*fid, pid)
            })
            .collect();

        crate::log_warn!(
            "[Community] CF 批量查询缺失 {} 个文件（请求 {}，返回 {}），可能已被作者删除：",
            missing.len(),
            file_ids.len(),
            batch.data.len()
        );
        for (fid, pid) in &missing {
            crate::log_warn!(
                "[Community]   缺失: file_id={} project_id={:?}",
                fid,
                pid
            );
        }

        return Err(format!(
            "CF 整合包缺失 {} 个文件（共请求 {} 个），它们可能已被原作者删除。请向整合包作者反馈此问题。\n缺失的 file_id：{}",
            missing.len(),
            file_ids.len(),
            missing
                .iter()
                .map(|(fid, _)| fid.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // 2. 批量查询 mod info 拿 slug（用于查 mcmod 译名 + 应用 community_filename_format）
    //    部分 file 可能缺失 project_id（Option），过滤 None 后仅查询有 project_id 的项
    let project_ids: Vec<i64> = effective_files.iter().filter_map(|f| f.project_id).collect();
    let mod_slug_map =
        crate::minecraft::community::curseforge::batch_get_mod_slugs(&project_ids).await;

    // 3. 构造 file_id → 译名 映射（通过 manifest 关联 file_id 与 project_id）
    //    project_id 为 None 的项跳过 slug 查询，译名留空（仍正常下载，仅文件名不翻译）
    //    file_id 为 None 的项在此跳过（已在 file_ids 构造时过滤）
    let mut file_translated: std::collections::HashMap<i64, Option<String>> =
        std::collections::HashMap::new();
    for mf in &effective_files {
        let Some(fid) = mf.file_id else { continue };
        let translated = mf
            .project_id
            .and_then(|pid| {
                let slug = mod_slug_map.get(&pid)?;
                crate::minecraft::community::mcmod::lookup_cf(slug).map(|n| n.to_string())
            });
        file_translated.insert(fid, translated);
    }

    // 4. 读取用户设置的文件名格式
    let filename_format = state.config.lock().await.community.filename_format;

    // 5. 构造下载列表（CF 通常只有一个 download_url，包装为单元素数组）
    //    根据 modules 字段分流到 mods / resourcepacks / shaderpacks 目录
    let mut download_list: Vec<(Vec<String>, String, u64)> = Vec::with_capacity(batch.data.len());
    let mut total_bytes: u64 = 0;
    let resourcepacks_dir = instance_dir.join("resourcepacks");
    let shaderpacks_dir = instance_dir.join("shaderpacks");
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
        // 按 modules 分流目标目录
        let target_dir = classify_cf_target_dir(&entry.modules, mods_dir, &resourcepacks_dir, &shaderpacks_dir);
        if !target_dir.exists() {
            std::fs::create_dir_all(&target_dir)
                .map_err(|e| format!("创建 {} 目录失败: {}", target_dir.display(), e))?;
        }
        let target = target_dir.join(&final_name);
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
        crate::utils::format::bytes(total_bytes)
    );

    // 6. 并发下载
    super::concurrent::download_files_concurrent(
        state,
        stage_index,
        &download_list,
        max_threads,
        total_bytes,
    )
    .await?;

    log_info!("[Community] CF mods 下载完成 ({} 个)", download_list.len());
    Ok(())
}

/// 根据 CF modules 字段判断文件类型并返回目标目录
///
/// - modules 含 META-INF 或 mcmod.info → mod（mods/）
/// - modules 含 pack.mcmeta → resourcepack（resourcepacks/）
/// - 其他 → shaderpack（shaderpacks/）
/// 无 modules 字段或 modules 为空时默认 mods/
fn classify_cf_target_dir(
    modules: &[CfModule],
    mods_dir: &std::path::Path,
    resourcepacks_dir: &std::path::Path,
    shaderpacks_dir: &std::path::Path,
) -> std::path::PathBuf {
    if modules.is_empty() {
        return mods_dir.to_path_buf();
    }
    let has_meta_inf = modules.iter().any(|m| m.name == "META-INF");
    let has_mcmod_info = modules.iter().any(|m| m.name == "mcmod.info");
    let has_pack_mcmeta = modules.iter().any(|m| m.name == "pack.mcmeta");
    if has_pack_mcmeta {
        resourcepacks_dir.to_path_buf()
    } else if has_meta_inf || has_mcmod_info {
        mods_dir.to_path_buf()
    } else {
        shaderpacks_dir.to_path_buf()
    }
}
