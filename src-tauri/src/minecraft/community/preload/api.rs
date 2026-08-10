//! 预加载主流程：preload_mods_detail / preload_packs_detail

use std::time::Instant;

use tauri::{AppHandle, Emitter};

use super::super::common::fmt_elapsed;
use super::cache::{load_file_cache, save_file_cache};
use super::jar_metadata::read_jar_metadata_and_hash;
use super::online_query::query_and_merge;
use super::types::{PreloadModInput, PreloadScope, PreloadUpdate};
use crate::minecraft::community::types::ResourceType;
use crate::minecraft::image_cache;

/// 预加载主入口
///
/// 流程：
/// 1. 读取持久化缓存，命中的条目直接 emit（不联网、不读文件）
/// 2. 未命中的条目并发读元数据（mods 读 JAR）+ 计算 hash，每完成一个就 emit 元数据
/// 3. 并发调 CF `/fingerprints` + MR `/version_files` 批量查询工程详情
/// 4. 每查到一个 project 就 emit
/// 5. 全部完成后写持久化缓存
pub async fn preload_mods_detail(app: AppHandle, version_id: String, mods: Vec<PreloadModInput>) {
    preload_detail(
        app,
        PreloadScope {
            event_prefix: "mods",
            resource_type: ResourceType::Mod,
            cache_dir: "preload_mods",
            read_jar_metadata: true,
        },
        version_id,
        mods,
    )
    .await;
}

/// 资源包/光影详情预加载
///
/// zip 包无 JAR 元数据，仅 hash 匹配 CF/MR 工程（logo / project 供详情与更新使用）。
pub async fn preload_packs_detail(
    app: AppHandle,
    version_id: String,
    resource_type: ResourceType,
    packs: Vec<PreloadModInput>,
) {
    let scope = match resource_type {
        ResourceType::ResourcePack => PreloadScope {
            event_prefix: "packs",
            resource_type,
            cache_dir: "preload_resourcepack",
            read_jar_metadata: false,
        },
        ResourceType::Shader => PreloadScope {
            event_prefix: "packs",
            resource_type,
            cache_dir: "preload_shader",
            read_jar_metadata: false,
        },
        _ => return,
    };
    preload_detail(app, scope, version_id, packs).await;
}

async fn preload_detail(
    app: AppHandle,
    scope: PreloadScope,
    version_id: String,
    inputs: Vec<PreloadModInput>,
) {
    if inputs.is_empty() {
        return;
    }

    let start = Instant::now();
    let total = inputs.len();
    crate::log_info!(
        "[Preload] 开始预加载 {} 个条目（type={:?}, version={}）",
        total,
        scope.resource_type,
        version_id
    );

    // 1. 读取持久化缓存，命中则直接推送（不联网、不读文件）
    let (cached, is_fresh) = load_file_cache(scope.cache_dir, &version_id);
    if is_fresh && !cached.is_empty() {
        crate::log_info!("[Preload] 缓存命中 {} 条，直接推送（不联网）", cached.len());
        for (file_name, cm) in &cached {
            // 从 project.logo_url 重新计算 cached_logo_url
            // （image_cache 状态可能已变化：首次未命中缓存的图片现在可能已下载完成）
            let cached_logo_url = match &cm.project {
                Some(project) => match &project.logo_url {
                    Some(url) if !url.is_empty() => {
                        let img = image_cache::get_image_url(url, Some(app.clone())).await;
                        Some(img.url)
                    }
                    _ => None,
                },
                None => None,
            };
            let _ = app.emit(
                format!("{}-preload-update", scope.event_prefix).as_str(),
                PreloadUpdate {
                    file_name: file_name.clone(),
                    // packs 无元数据字段，保持 None 防止前端覆盖
                    slug: scope.read_jar_metadata.then(|| cm.slug.clone()),
                    description: scope.read_jar_metadata.then(|| cm.description.clone()),
                    version: scope.read_jar_metadata.then(|| cm.version.clone()),
                    cached_logo_url,
                    translated_name: scope.read_jar_metadata.then(|| cm.translated_name.clone()),
                    project: cm.project.clone(),
                },
            );
        }
        // 通知前端预加载已全部完成（前端据此跳过 handleShowInfo 的等待循环）
        let _ = app.emit(format!("{}-preload-done", scope.event_prefix).as_str(), ());
        return;
    }

    // 2. 并发读元数据 + 计算 hash（每完成一个就 emit 元数据）
    let hashed_mods = read_jar_metadata_and_hash(&app, &scope, inputs).await;

    crate::log_info!(
        "[Preload] 元数据读取完成：{} / {} 个（耗时 {}）",
        hashed_mods.len(),
        total,
        fmt_elapsed(start)
    );

    if hashed_mods.is_empty() {
        crate::log_warn!("[Preload] 没有可计算 hash 的条目，跳过联网查询");
        return;
    }

    // 3. 并发批量查询 CF + MR，合并结果并 emit project 事件
    let stats = query_and_merge(&app, &scope, &hashed_mods).await;

    // 4. 写持久化缓存
    save_file_cache(scope.cache_dir, &version_id, &stats.cache_map);

    crate::log_info!(
        "[Preload] 预加载完成：CF 命中 {}，MR 命中 {}，共 {} / {} 个有 project（总耗时 {}）",
        stats.cf_count,
        stats.mr_count,
        stats
            .cache_map
            .values()
            .filter(|c| c.project.is_some())
            .count(),
        hashed_mods.len(),
        fmt_elapsed(start)
    );

    // 通知前端预加载已全部完成（前端据此跳过 handleShowInfo 的等待循环）
    let _ = app.emit(format!("{}-preload-done", scope.event_prefix).as_str(), ());
}
