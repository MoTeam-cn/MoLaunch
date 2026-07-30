//! JAR 元数据读取 + hash 计算
//!
//! 用 `tokio::task::spawn_blocking` 并发读 JAR 元数据并计算 CF/MR 双平台 hash（zip 读取是同步 IO）。
//! 每完成一个 mod 就立即 emit 元数据事件（前端马上能看到译名、logo、版本等）

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;

use super::hash::{compute_curseforge_fingerprint, compute_modrinth_sha1};
use super::types::{HashedMod, PreloadModInput, PreloadUpdate};

/// 并发读 JAR 元数据 + 计算 hash
///
/// 限制并发度 8（与原实现一致），每个 mod 在独立的 spawn_blocking 任务中：
/// 1. 读 JAR 元数据（slug / 描述 / 版本 / logo / 译名）
/// 2. 计算 CF MurmurHash2 + MR SHA1
/// 3. 立即 emit `mods-preload-update` 元数据事件（project 字段为 None）
///
/// 返回所有成功读取的 `HashedMod`（任务 panic 的会被跳过并记日志）。
pub(crate) async fn read_jar_metadata_and_hash(
    app: &AppHandle,
    mods: Vec<PreloadModInput>,
) -> Vec<HashedMod> {
    let semaphore = Arc::new(Semaphore::new(8)); // 限制并发度 8
    let total = mods.len();
    let mut handles = Vec::with_capacity(total);

    for m in mods {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let app = app.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let _permit = permit; // 持有 permit 直到读完
            let path = PathBuf::from(&m.path);

            // 读 JAR 元数据
            let metadata = crate::commands::version::mods::read_mod_metadata(&path);

            // 计算 hash（CF MurmurHash2 + MR SHA1）
            let cf = compute_curseforge_fingerprint(&path).ok();
            let mr = compute_modrinth_sha1(&path).ok();

            // 立即 emit 元数据事件（前端马上能看到译名、logo、版本等）
            let _ = app.emit(
                "mods-preload-update",
                PreloadUpdate {
                    file_name: m.file_name.clone(),
                    slug: Some(metadata.slug.clone()),
                    description: Some(metadata.description.clone()),
                    version: Some(metadata.version.clone()),
                    cached_logo_url: None,
                    translated_name: Some(metadata.translated_name.clone()),
                    project: None,
                },
            );

            HashedMod {
                file_name: m.file_name,
                metadata,
                cf_fingerprint: cf,
                mr_sha1: mr,
            }
        });
        handles.push(handle);
    }

    // 等待所有 JAR 读取完成
    let mut hashed_mods: Vec<HashedMod> = Vec::with_capacity(total);
    for h in handles {
        match h.await {
            Ok(hm) => hashed_mods.push(hm),
            Err(e) => crate::log_warn!("[Preload] JAR 读取任务失败: {}", e),
        }
    }
    hashed_mods
}
