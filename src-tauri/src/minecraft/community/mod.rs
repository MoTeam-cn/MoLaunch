//! 社区资源模块：对接 CurseForge 和 Modrinth 两大平台 API

pub mod cache;
pub mod common;
pub mod curseforge;
pub mod fuzzy;
pub mod mcmod;
pub mod modrinth;
pub mod preload;
pub mod searcher;
pub mod secure_storage;
pub mod tags;
pub mod types;
pub mod version_extract;

pub use searcher::search;
pub use types::{
    Platform, ResourceProject, ResourceType, ResourceVersion, SearchParams, SearchResult,
};

/// 读取社区资源来源策略
///
/// 0=尽量镜像 / 1=缓慢时换镜像 / 2=尽量官方（默认）
///
/// 直接从 INI 读取（无内存缓存），因 INI 文件小且 `set_community_config` 命令
// 通过 `update_config` 写 INI，故配置变更后立即生效
pub fn get_source_pref() -> u8 {
    crate::storage::Storage::instance()
        .get_config("Community", "source")
        .and_then(|v| v.parse::<u8>().ok())
        .filter(|&v| v <= 2)
        .unwrap_or(2)
}

/// 读取是否忽略 Quilt 加载器
///
/// 默认 true
pub fn get_ignore_quilt() -> bool {
    crate::storage::Storage::instance()
        .get_config("Community", "ignore_quilt")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true)
}
