//! 预加载持久化缓存
//!
//! 参考 PCL2 `Cache/LocalMod.json`：结果写入
//! `.Molaunch/cache/preload_mods/{version_id}.json`，6h TTL。
//! 结构变更时递增 `PRELOAD_CACHE_VERSION` 使旧缓存失效。

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::minecraft::community::types::ResourceProject;

/// 预加载缓存版本号（结构变更时递增，使旧缓存失效）
pub(crate) const PRELOAD_CACHE_VERSION: u32 = 2;
/// 缓存有效期（6 小时，参考 PCL2）
pub(crate) const PRELOAD_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);

/// 持久化缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CacheEntry {
    /// 缓存版本号（与 `PRELOAD_CACHE_VERSION` 不匹配时弃用）
    pub version: u32,
    /// 缓存写入时间（Unix 时间戳，秒）
    pub cache_time: i64,
    /// 每个 mod 的完整元数据 + project（按 file_name 索引）
    pub mods: HashMap<String, CachedMod>,
}

/// 缓存中单个 mod 的完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CachedMod {
    pub slug: String,
    pub description: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_data: Option<String>,
    pub translated_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ResourceProject>,
}

/// 读取持久化缓存（参考 PCL2 `Cache/LocalMod.json`）
///
/// 返回 `(cached_map, is_fresh)`：
/// - `cached_map`：file_name → CachedMod
/// - `is_fresh`：缓存是否存在且未过期（false 表示需重新联网）
pub(crate) fn load_file_cache(version_id: &str) -> (HashMap<String, CachedMod>, bool) {
    let rel = format!("preload_mods/{}.json", sanitize_cache_key(version_id));
    let json = match crate::storage::cache::Cache::instance().read(&rel) {
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
pub(crate) fn save_file_cache(version_id: &str, mods: &HashMap<String, CachedMod>) {
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
    let rel = format!("preload_mods/{}.json", sanitize_cache_key(version_id));
    if let Err(e) = crate::storage::cache::Cache::instance().write(&rel, &json) {
        crate::log_warn!("[Preload] 缓存写入失败: {}", e);
    }
}

/// 净化缓存文件名（防止 version_id 含特殊字符）
fn sanitize_cache_key(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
