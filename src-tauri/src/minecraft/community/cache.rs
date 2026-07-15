//! 社区资源内存缓存
//!
//! 参考 PCL2 ResourceProject.Cache / ResourceVersion.ProjectFilesCache
//! 缓存工程详情和版本列表，避免重复请求

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::types::{ResourceProject, ResourceVersion};

/// 缓存条目
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
}

/// 缓存 TTL
const PROJECT_TTL: Duration = Duration::from_secs(300); // 5 分钟
const VERSIONS_TTL: Duration = Duration::from_secs(300); // 5 分钟

/// 工程详情缓存: key = "platform:id"
static PROJECT_CACHE: once_cell::sync::Lazy<Mutex<HashMap<String, CacheEntry<ResourceProject>>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// 版本列表缓存: key = "platform:id"
static VERSIONS_CACHE: once_cell::sync::Lazy<Mutex<HashMap<String, CacheEntry<Vec<ResourceVersion>>>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

fn project_key(platform: &str, id: &str) -> String {
    format!("{}:{}", platform, id)
}

/// 获取缓存的工程详情
pub fn get_project(platform: &str, id: &str) -> Option<ResourceProject> {
    let key = project_key(platform, id);
    let cache = PROJECT_CACHE.lock().ok()?;
    let entry = cache.get(&key)?;
    if entry.created_at.elapsed() < PROJECT_TTL {
        Some(entry.data.clone())
    } else {
        None
    }
}

/// 写入工程详情缓存
pub fn set_project(platform: &str, id: &str, project: &ResourceProject) {
    let key = project_key(platform, id);
    if let Ok(mut cache) = PROJECT_CACHE.lock() {
        cache.insert(
            key,
            CacheEntry {
                data: project.clone(),
                created_at: Instant::now(),
            },
        );
    }
}

/// 获取缓存的版本列表
pub fn get_versions(platform: &str, id: &str) -> Option<Vec<ResourceVersion>> {
    let key = project_key(platform, id);
    let cache = VERSIONS_CACHE.lock().ok()?;
    let entry = cache.get(&key)?;
    if entry.created_at.elapsed() < VERSIONS_TTL {
        Some(entry.data.clone())
    } else {
        None
    }
}

/// 写入版本列表缓存
pub fn set_versions(platform: &str, id: &str, versions: &[ResourceVersion]) {
    let key = project_key(platform, id);
    if let Ok(mut cache) = VERSIONS_CACHE.lock() {
        cache.insert(
            key,
            CacheEntry {
                data: versions.to_vec(),
                created_at: Instant::now(),
            },
        );
    }
}

/// 清空所有缓存（切换页面/刷新时调用）
pub fn clear_all() {
    if let Ok(mut c) = PROJECT_CACHE.lock() {
        c.clear();
    }
    if let Ok(mut c) = VERSIONS_CACHE.lock() {
        c.clear();
    }
}
