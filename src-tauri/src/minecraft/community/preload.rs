//! 本地 Mod 详情预加载
//!
//! 参考 PCL2 `Modules/Resource/LocalResourceLoaders.vb` 的 `LocalResourceOnlineLoader`：
//! 在 `list_mods` 返回后（同步阶段只做文件枚举），后台异步完成两件事：
//!
//! 1. **读 JAR 元数据**（slug / 描述 / 版本 / logo / 译名）：参考 PCL2 `LoadMetadataFromJar`
//! 2. **批量 hash 查询 CF/MR 工程详情**：参考 PCL2 `LocalResourceOnlineLoader` 联网阶段
//!
//! 两件事都通过 `mods-preload-update` 事件推送给前端，前端按 `file_name` 匹配更新。
//! 元数据读完就先 emit（前端立即看到译名、logo），不必等联网查询完成。
//!
//! 核心设计（对齐 PCL2）：
//! - **两阶段加载**：`list_mods` 同步只做文件枚举（瞬间返回），本模块异步补全所有元数据
//! - **持久化缓存**：结果写入 `.Molaunch/cache/preload_mods/{version_id}.json`，6h TTL
//! - **事件驱动**：每读到一个 mod 的元数据/project 就 emit，前端逐个刷新

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use super::common::fmt_elapsed;
use super::types::{ResourceProject, ResourceType};

/// 预加载缓存版本号（结构变更时递增，使旧缓存失效）
const PRELOAD_CACHE_VERSION: u32 = 2;
/// 缓存有效期（6 小时，参考 PCL2）
const PRELOAD_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);

/// 单条预加载结果（推送给前端的事件 payload）
///
/// 前端按 `file_name` 匹配对应 mod，更新所有非 null 字段。
/// 元数据字段和 project 可能分两次 emit（元数据先、project 后）。
#[derive(Debug, Clone, Serialize)]
pub struct PreloadUpdate {
    /// 本地 mod 文件名（前端按此字段匹配更新对应 mod）
    pub file_name: String,
    /// JAR 内读到的 slug（空字符串表示未读到，前端不更新）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// JAR 内读到的描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JAR 内读到的版本号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// JAR 内提取的 logo（base64 data URL）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_data: Option<String>,
    /// mcmod 数据库查到的中文译名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_name: Option<String>,
    /// CF/MR 查到的平台工程（None 表示未查到或尚未查询）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ResourceProject>,
}

/// 持久化缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// 缓存版本号（与 `PRELOAD_CACHE_VERSION` 不匹配时弃用）
    version: u32,
    /// 缓存写入时间（Unix 时间戳，秒）
    cache_time: i64,
    /// 每个 mod 的完整元数据 + project（按 file_name 索引）
    mods: HashMap<String, CachedMod>,
}

/// 缓存中单个 mod 的完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedMod {
    slug: String,
    description: String,
    version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    logo_data: Option<String>,
    translated_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project: Option<ResourceProject>,
}

/// 读取持久化缓存（参考 PCL2 `Cache/LocalMod.json`）
///
/// 返回 `(cached_map, is_fresh)`：
/// - `cached_map`：file_name → CachedMod
/// - `is_fresh`：缓存是否存在且未过期（false 表示需重新联网）
fn load_file_cache(version_id: &str) -> (HashMap<String, CachedMod>, bool) {
    let rel = format!("cache/preload_mods/{}.json", sanitize_cache_key(version_id));
    let json = match crate::storage::Storage::instance().read_file(&rel) {
        Ok(s) => s,
        Err(_) => return (HashMap::new(), false),
    };
    let entry: CacheEntry = match serde_json::from_str(&json) {
        Ok(e) => e,
        Err(e) => {
            crate::log_warn!("[Preload] 缓存解析失败，弃用: {}", e);
            return (HashMap::new(), false);
        }
    };
    if entry.version != PRELOAD_CACHE_VERSION {
        crate::log_info!(
            "[Preload] 缓存版本过期 ({} != {})，重新联网",
            entry.version,
            PRELOAD_CACHE_VERSION
        );
        return (HashMap::new(), false);
    }
    let now = chrono::Utc::now().timestamp();
    let is_fresh = now - entry.cache_time < PRELOAD_CACHE_TTL.as_secs() as i64;
    (entry.mods, is_fresh)
}

/// 写入持久化缓存
fn save_file_cache(version_id: &str, mods: &HashMap<String, CachedMod>) {
    let entry = CacheEntry {
        version: PRELOAD_CACHE_VERSION,
        cache_time: chrono::Utc::now().timestamp(),
        mods: mods.clone(),
    };
    let json = match serde_json::to_string(&entry) {
        Ok(s) => s,
        Err(e) => {
            crate::log_warn!("[Preload] 缓存序列化失败: {}", e);
            return;
        }
    };
    let rel = format!("cache/preload_mods/{}.json", sanitize_cache_key(version_id));
    if let Err(e) = crate::storage::Storage::instance().write_file(&rel, &json) {
        crate::log_warn!("[Preload] 缓存写入失败: {}", e);
    }
}

/// 净化缓存文件名（防止 version_id 含特殊字符）
fn sanitize_cache_key(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// 计算 CurseForge 用的 MurmurHash2 指纹
///
/// 参考 PCL2 `LocalResourceFile.CurseForgeHash`：
/// 1. 读取文件所有字节
/// 2. **跳过空白字符**（0x09 制表符 / 0x0A 换行 / 0x0D 回车 / 0x20 空格）
/// 3. 对处理后的字节流做 MurmurHash2（seed=1，与 CF 官方一致）
pub fn compute_curseforge_fingerprint(path: &Path) -> Result<u32, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let filtered: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|&b| b != 0x09 && b != 0x0A && b != 0x0D && b != 0x20)
        .collect();
    Ok(murmur_hash2(&filtered, 1))
}

/// Modrinth 用的 SHA1 哈希（hex 字符串）
pub fn compute_modrinth_sha1(path: &Path) -> Result<String, String> {
    use sha1::{Digest, Sha1};
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// MurmurHash2 算法（参考 PCL2 `LocalResourceFile.CurseForgeHash` 第 434-459 行）
fn murmur_hash2(data: &[u8], seed: u32) -> u32 {
    let m: u32 = 0x5bd1_e995;
    let r: u32 = 24;
    let len = data.len();

    let mut h: u32 = seed ^ (len as u32);

    let mut i = 0;
    while i + 4 <= len {
        let mut k = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        k = k.wrapping_mul(m);
        k ^= k >> r;
        k = k.wrapping_mul(m);

        h = h.wrapping_mul(m);
        h ^= k;

        i += 4;
    }

    let remaining = len - i;
    if remaining >= 3 {
        h ^= (data[i + 2] as u32) << 16;
    }
    if remaining >= 2 {
        h ^= (data[i + 1] as u32) << 8;
    }
    if remaining >= 1 {
        h ^= data[i] as u32;
        h = h.wrapping_mul(m);
    }

    h ^= h >> 13;
    h = h.wrapping_mul(m);
    h ^= h >> 15;

    h
}

/// 预加载主入口
///
/// 流程（参考 PCL2 `LocalResourceOnlineLoader`）：
/// 1. 读取持久化缓存，命中的 mod 直接 emit（不联网、不读 jar）
/// 2. 未命中的 mod 并发读 JAR 元数据 + 计算 hash
/// 3. 每读完一个 mod 的元数据就 emit（前端立即看到译名、logo、版本等）
/// 4. 并发调 CF `/fingerprints` + MR `/version_files` 批量查询工程详情
/// 5. 每查到一个 project 就 emit
/// 6. 全部完成后写持久化缓存
pub async fn preload_mods_detail(
    app: AppHandle,
    version_id: String,
    mods: Vec<PreloadModInput>,
) {
    if mods.is_empty() {
        return;
    }

    let start = Instant::now();
    crate::log_info!(
        "[Preload] 开始预加载 {} 个 mod（version={}）",
        mods.len(),
        version_id
    );

    // 1. 读取持久化缓存
    let (cached, is_fresh) = load_file_cache(&version_id);
    if is_fresh && !cached.is_empty() {
        crate::log_info!(
            "[Preload] 缓存命中 {} 条，直接推送（不联网）",
            cached.len()
        );
        for (file_name, cm) in &cached {
            let _ = app.emit(
                "mods-preload-update",
                PreloadUpdate {
                    file_name: file_name.clone(),
                    slug: Some(cm.slug.clone()),
                    description: Some(cm.description.clone()),
                    version: Some(cm.version.clone()),
                    logo_data: cm.logo_data.clone(),
                    translated_name: Some(cm.translated_name.clone()),
                    project: cm.project.clone(),
                },
            );
        }
        // 通知前端预加载已全部完成（前端据此跳过 handleShowInfo 的等待循环）
        let _ = app.emit("mods-preload-done", ());
        return;
    }

    // 2. 并发读 JAR 元数据 + 计算 hash
    //    用 tokio::task::spawn_blocking 在阻塞线程池中执行（zip 读取是同步 IO）
    //    每完成一个就 emit 元数据事件
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    #[derive(Clone)]
    struct HashedMod {
        file_name: String,
        metadata: crate::commands::version::mods::ModMetadata,
        cf_fingerprint: Option<u32>,
        mr_sha1: Option<String>,
    }

    let semaphore = Arc::new(Semaphore::new(8)); // 限制并发度 8
    let total = mods.len();
    let mut handles = Vec::with_capacity(total);

    for m in mods {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let app = app.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let _permit = permit; // 持有 permit 直到读完
            let path = std::path::PathBuf::from(&m.path);

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
                    logo_data: metadata.logo_data.clone(),
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

    // 3. 并发批量查询 CF + MR
    let cf_fingerprints: Vec<u32> = hashed_mods
        .iter()
        .filter_map(|m| m.cf_fingerprint)
        .collect();
    let mr_sha1s: Vec<String> = hashed_mods
        .iter()
        .filter_map(|m| m.mr_sha1.clone())
        .collect();

    let (cf_result, mr_result) = tokio::join!(
        super::curseforge::fingerprint_search(cf_fingerprints, ResourceType::Mod),
        super::modrinth::version_files_search(mr_sha1s, ResourceType::Mod),
    );

    // 4. 合并结果：CF 优先（CF 收录更全），CF 没有再用 MR
    //    同时构建缓存并 emit project 事件
    let mut cache_map: HashMap<String, CachedMod> = HashMap::new();
    let mut cf_count = 0;
    let mut mr_count = 0;

    // 先把所有 mod 的元数据写入缓存
    for m in &hashed_mods {
        cache_map.insert(
            m.file_name.clone(),
            CachedMod {
                slug: m.metadata.slug.clone(),
                description: m.metadata.description.clone(),
                version: m.metadata.version.clone(),
                logo_data: m.metadata.logo_data.clone(),
                translated_name: m.metadata.translated_name.clone(),
                project: None,
            },
        );
    }

    // CF 结果
    if let Ok(cf_map) = &cf_result {
        for m in &hashed_mods {
            if let Some(fp) = m.cf_fingerprint {
                if let Some(project) = cf_map.get(&fp) {
                    if let Some(cm) = cache_map.get_mut(&m.file_name) {
                        cm.project = Some(project.clone());
                    }
                    cf_count += 1;
                    let _ = app.emit(
                        "mods-preload-update",
                        PreloadUpdate {
                            file_name: m.file_name.clone(),
                            slug: None,
                            description: None,
                            version: None,
                            logo_data: None,
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
        for m in &hashed_mods {
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
                    let _ = app.emit(
                        "mods-preload-update",
                        PreloadUpdate {
                            file_name: m.file_name.clone(),
                            slug: None,
                            description: None,
                            version: None,
                            logo_data: None,
                            translated_name: None,
                            project: Some(project.clone()),
                        },
                    );
                }
            }
        }
    }

    // 5. 写持久化缓存
    save_file_cache(&version_id, &cache_map);

    crate::log_info!(
        "[Preload] 预加载完成：CF 命中 {}，MR 命中 {}，共 {} / {} 个 mod 有 project（总耗时 {}）",
        cf_count,
        mr_count,
        cache_map.values().filter(|c| c.project.is_some()).count(),
        hashed_mods.len(),
        fmt_elapsed(start)
    );

    // 通知前端预加载已全部完成（前端据此跳过 handleShowInfo 的等待循环）
    let _ = app.emit("mods-preload-done", ());
}

/// 预加载输入：每个 mod 的文件名和绝对路径
#[derive(Debug, Clone)]
pub struct PreloadModInput {
    pub file_name: String,
    pub path: String,
}
