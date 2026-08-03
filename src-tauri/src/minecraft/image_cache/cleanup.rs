//! 图片缓存清理

use super::store::IMAGE_CACHE_DIR;
use crate::utils::cache;

/// 清除指定 URL 的缓存（用于强制刷新）
pub fn invalidate(remote_url: &str) -> anyhow::Result<()> {
    let rel = super::store::cache_rel_path(remote_url);
    cache::remove(&rel)
}

/// 清空所有图片缓存
pub fn clear_all() -> anyhow::Result<()> {
    cache::clear_dir(IMAGE_CACHE_DIR)
}