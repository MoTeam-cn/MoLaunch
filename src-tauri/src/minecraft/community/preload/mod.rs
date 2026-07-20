//! 本地 Mod 详情预加载
//!
//! 在 `list_mods` 返回后（同步阶段只做文件枚举），后台异步完成两件事：
//! 1. **读 JAR 元数据**（slug / 描述 / 版本 / logo / 译名）
//! 2. **批量 hash 查询 CF/MR 工程详情**
//!
//! 两件事都通过 `mods-preload-update` 事件推送给前端，前端按 `file_name` 匹配更新。
//! 元数据读完就先 emit（前端立即看到译名、logo），不必等联网查询完成。
//!
//! 核心设计：
//! - **两阶段加载**：`list_mods` 同步只做文件枚举（瞬间返回），本模块异步补全所有元数据
//! - **持久化缓存**：结果写入 `.Molaunch/cache/preload_mods/{version_id}.json`，6h TTL
//! - **事件驱动**：每读到一个 mod 的元数据/project 就 emit，前端逐个刷新
//!
//! 模块结构：
//! - types.rs: PreloadUpdate / PreloadModInput / HashedMod
//! - cache.rs: 持久化缓存读写
//! - hash.rs: CurseForge MurmurHash2 + Modrinth SHA1 指纹
//! - jar_metadata.rs: JAR 元数据读取 + hash 计算（spawn_blocking）
//! - online_query.rs: CF/MR 在线查询 + 结果合并 + 事件推送

mod cache;
mod hash;
mod jar_metadata;
mod online_query;
mod types;

use std::time::Instant;

use tauri::{AppHandle, Emitter};

use super::common::fmt_elapsed;
use cache::{load_file_cache, save_file_cache};
use jar_metadata::read_jar_metadata_and_hash;
use online_query::query_and_merge;
use crate::minecraft::image_cache;

pub use hash::{compute_curseforge_fingerprint, compute_modrinth_sha1};
// pub use 同时把项带入当前作用域供本文件内使用，无需重复 use（参考 mods/mod.rs 模式）
pub use types::{PreloadModInput, PreloadUpdate};

/// 预加载主入口
///
/// 流程：
/// 1. 读取持久化缓存，命中的 mod 直接 emit（不联网、不读 jar）
/// 2. 未命中的 mod 并发读 JAR 元数据 + 计算 hash，每读完一个就 emit 元数据
/// 3. 并发调 CF `/fingerprints` + MR `/version_files` 批量查询工程详情
/// 4. 每查到一个 project 就 emit
/// 5. 全部完成后写持久化缓存
pub async fn preload_mods_detail(
    app: AppHandle,
    version_id: String,
    mods: Vec<PreloadModInput>,
) {
    if mods.is_empty() {
        return;
    }

    let start = Instant::now();
    let total = mods.len();
    crate::log_info!(
        "[Preload] 开始预加载 {} 个 mod（version={}）",
        total,
        version_id
    );

    // 1. 读取持久化缓存，命中则直接推送（不联网、不读 jar）
    let (cached, is_fresh) = load_file_cache(&version_id);
    if is_fresh && !cached.is_empty() {
        crate::log_info!(
            "[Preload] 缓存命中 {} 条，直接推送（不联网）",
            cached.len()
        );
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
                "mods-preload-update",
                PreloadUpdate {
                    file_name: file_name.clone(),
                    slug: Some(cm.slug.clone()),
                    description: Some(cm.description.clone()),
                    version: Some(cm.version.clone()),
                    cached_logo_url,
                    translated_name: Some(cm.translated_name.clone()),
                    project: cm.project.clone(),
                },
            );
        }
        // 通知前端预加载已全部完成（前端据此跳过 handleShowInfo 的等待循环）
        let _ = app.emit("mods-preload-done", ());
        return;
    }

    // 2. 并发读 JAR 元数据 + 计算 hash（每读完一个就 emit 元数据）
    let hashed_mods = read_jar_metadata_and_hash(&app, mods).await;

    crate::log_info!(
        "[Preload] JAR 元数据读取完成：{} / {} 个（耗时 {}）",
        hashed_mods.len(),
        total,
        fmt_elapsed(start)
    );

    if hashed_mods.is_empty() {
        crate::log_warn!("[Preload] 没有可计算 hash 的 mod，跳过联网查询");
        return;
    }

    // 3. 并发批量查询 CF + MR，合并结果并 emit project 事件
    let stats = query_and_merge(&app, &hashed_mods).await;

    // 4. 写持久化缓存
    save_file_cache(&version_id, &stats.cache_map);

    crate::log_info!(
        "[Preload] 预加载完成：CF 命中 {}，MR 命中 {}，共 {} / {} 个 mod 有 project（总耗时 {}）",
        stats.cf_count,
        stats.mr_count,
        stats.cache_map.values().filter(|c| c.project.is_some()).count(),
        hashed_mods.len(),
        fmt_elapsed(start)
    );

    // 通知前端预加载已全部完成（前端据此跳过 handleShowInfo 的等待循环）
    let _ = app.emit("mods-preload-done", ());
}
