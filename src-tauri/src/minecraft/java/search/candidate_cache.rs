//! Java 候选路径缓存
//!
//! 全盘扫描是 Java 搜索里最慢的一步（秒级到几十秒），而候选路径本身很少变。
//! 缓存一份、下次只做验证，比重新扫一遍划算得多。

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 缓存有效期
const TTL: Duration = Duration::from_secs(30 * 60);

const STORE_PATH: &str = "java/candidate_cache.json";
const VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Store {
    version: u32,
    key: String,
    created_ms: u64,
    candidates: Vec<String>,
}

/// 额外搜索路径集合的指纹：与顺序无关，重复项忽略
pub(super) fn key_of(extra_paths: &[PathBuf]) -> String {
    let mut parts: Vec<String> = extra_paths
        .iter()
        .map(|path| path.to_string_lossy().to_lowercase())
        .collect();
    parts.sort();
    parts.dedup();
    parts.join("|")
}

/// 读取缓存：三条同时成立才算命中——同一批额外路径、未过期、候选都还在
pub(super) fn load(extra_paths: &[PathBuf]) -> Option<Vec<PathBuf>> {
    let text = std::fs::read_to_string(store_path()?).ok()?;
    let store: Store = serde_json::from_str(&text).ok()?;
    if store.version != VERSION || store.key != key_of(extra_paths) {
        return None;
    }
    if !is_fresh(store.created_ms, unix_millis()) {
        return None;
    }
    if !candidates_alive(&store.candidates) {
        // 用户卸载了某个 Java：缓存里有死路径，必须重扫
        crate::log_debug!("[Java] 候选缓存含已消失的路径，作废重扫");
        return None;
    }
    Some(store.candidates.into_iter().map(PathBuf::from).collect())
}

/// 写缓存；空结果不写——「一个 Java 都没找到」多半是环境问题，不该被记住半小时
pub(super) fn save(extra_paths: &[PathBuf], candidates: &[PathBuf]) {
    if candidates.is_empty() {
        return;
    }
    let Some(path) = store_path() else {
        return;
    };
    let store = Store {
        version: VERSION,
        key: key_of(extra_paths),
        created_ms: unix_millis(),
        candidates: candidates
            .iter()
            .map(|item| item.to_string_lossy().to_string())
            .collect(),
    };
    let Ok(text) = serde_json::to_string(&store) else {
        return;
    };
    if let Err(e) = crate::utils::fs::write_atomic(&path, text.as_bytes()) {
        crate::log_debug!("[Java] 候选缓存落盘失败: {}", e);
    }
}

fn is_fresh(created_ms: u64, now_ms: u64) -> bool {
    now_ms.saturating_sub(created_ms) <= TTL.as_millis() as u64
}

fn candidates_alive(candidates: &[String]) -> bool {
    candidates.iter().all(|item| Path::new(item).exists())
}

fn store_path() -> Option<PathBuf> {
    Some(crate::storage::cache::Cache::instance().path(STORE_PATH))
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "candidate_cache_test.rs"]
mod candidate_cache_test;
