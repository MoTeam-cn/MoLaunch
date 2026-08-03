//! 社区资源模块：对接 CurseForge 和 Modrinth 两大平台 API

pub mod cache;
pub mod common;
mod config;
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
pub use config::{get_ignore_quilt, get_source_pref};
