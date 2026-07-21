//! CF/MR 在线批量查询 + 结果合并 + 事件推送
//!
//! 对应 `preload_mods_detail` 的第 3-4 步：
//! 1. `tokio::join!` 并发调 CF `/fingerprints` + MR `/version_files`
//! 2. 合并结果：CF 优先（CF 收录更全），CF 没有再用 MR
//! 3. 每查到一个 project 就 emit `mods-preload-update` 事件
//! 4. 返回完整 `cache_map`（含元数据 + project）供持久化缓存写入

use std::collections::HashMap;

use tauri::{AppHandle, Emitter};

use super::cache::CachedMod;
use super::types::{HashedMod, PreloadUpdate};
use crate::minecraft::community::curseforge::fingerprint_search;
use crate::minecraft::community::modrinth::version_files_search;
use crate::minecraft::community::types::ResourceType;
use crate::minecraft::image_cache;

/// 在线批量查询的统计结果（供 mod.rs 写最终日志）
pub(crate) struct QueryStats {
    /// file_name → CachedMod（含元数据 + project，用于写持久化缓存）
    pub cache_map: HashMap<String, CachedMod>,
    /// CF 命中数
    pub cf_count: usize,
    /// MR 命中数
    pub mr_count: usize,
}

/// 并发批量查询 CF + MR，合并结果并 emit project 事件
///
/// CF 优先（CF 收录更全），CF 没有再用 MR。返回写入缓存用的 `cache_map`。
pub(crate) async fn query_and_merge(app: &AppHandle, hashed_mods: &[HashedMod]) -> QueryStats {
    let cf_fingerprints: Vec<u32> = hashed_mods
        .iter()
        .filter_map(|m| m.cf_fingerprint)
        .collect();
    let mr_sha1s: Vec<String> = hashed_mods
        .iter()
        .filter_map(|m| m.mr_sha1.clone())
        .collect();

    let (cf_result, mr_result) = tokio::join!(
        fingerprint_search(cf_fingerprints, ResourceType::Mod),
        version_files_search(mr_sha1s, ResourceType::Mod),
    );

    let mut cache_map: HashMap<String, CachedMod> = HashMap::new();
    let mut cf_count = 0;
    let mut mr_count = 0;

    // 先把所有 mod 的元数据写入缓存（project 暂为 None，后面填）
    for m in hashed_mods {
        cache_map.insert(
            m.file_name.clone(),
            CachedMod {
                slug: m.metadata.slug.clone(),
                description: m.metadata.description.clone(),
                version: m.metadata.version.clone(),
                cached_logo_url: None,
                translated_name: m.metadata.translated_name.clone(),
                project: None,
            },
        );
    }

    // CF 结果
    if let Ok(cf_map) = &cf_result {
        for m in hashed_mods {
            if let Some(fp) = m.cf_fingerprint {
                if let Some(project) = cf_map.get(&fp) {
                    if let Some(cm) = cache_map.get_mut(&m.file_name) {
                        cm.project = Some(project.clone());
                    }
                    cf_count += 1;
                    // 预填充 logo 缓存 URL（参考皮肤/披风 cached_url 机制）
                    let cached_logo_url = match &project.logo_url {
                        Some(url) if !url.is_empty() => {
                            let img = image_cache::get_image_url(url, Some(app.clone())).await;
                            Some(img.url)
                        }
                        _ => None,
                    };
                    let _ = app.emit(
                        "mods-preload-update",
                        PreloadUpdate {
                            file_name: m.file_name.clone(),
                            slug: None,
                            description: None,
                            version: None,
                            cached_logo_url,
                            translated_name: None,
                            project: Some(project.clone()),
                        },
                    );
                }
            }
        }
    }

    // MR 结果（CF 已查到的跳过）
    if let Ok(mr_map) = &mr_result {
        for m in hashed_mods {
            let already = cache_map
                .get(&m.file_name)
                .and_then(|cm| cm.project.as_ref())
                .is_some();
            if already {
                continue;
            }
            if let Some(ref sha1) = m.mr_sha1 {
                if let Some(project) = mr_map.get(sha1) {
                    if let Some(cm) = cache_map.get_mut(&m.file_name) {
                        cm.project = Some(project.clone());
                    }
                    mr_count += 1;
                    // 预填充 logo 缓存 URL（参考皮肤/披风 cached_url 机制）
                    let cached_logo_url = match &project.logo_url {
                        Some(url) if !url.is_empty() => {
                            let img = image_cache::get_image_url(url, Some(app.clone())).await;
                            Some(img.url)
                        }
                        _ => None,
                    };
                    let _ = app.emit(
                        "mods-preload-update",
                        PreloadUpdate {
                            file_name: m.file_name.clone(),
                            slug: None,
                            description: None,
                            version: None,
                            cached_logo_url,
                            translated_name: None,
                            project: Some(project.clone()),
                        },
                    );
                }
            }
        }
    }

    QueryStats {
        cache_map,
        cf_count,
        mr_count,
    }
}
