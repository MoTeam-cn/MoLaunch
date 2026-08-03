//! 通用图片缓存组件（皮肤/披风/头像等远程 PNG）
//!
//! 混合缓存：首次返回远程 URL，后端异步下载到本地；二次返回自定义 URI scheme；
//! 子模块：store / download / cleanup。

mod cleanup;
mod download;
mod store;

pub use cleanup::{clear_all, invalidate};
pub use download::get_image_url;
pub use store::{
    cache_abs_path, cache_path_by_url, find_cache_by_hash, is_cache_url, parse_hash_from_request,
    read_cache_by_url, register_uri_scheme, CachedImage, CACHE_IMAGE_SCHEME,
};
