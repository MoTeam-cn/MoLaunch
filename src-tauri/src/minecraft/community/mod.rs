//! 社区资源模块
//!
//! 参考 PCL2 PageDownload/Resource + Modules/Resource
//! 对接 CurseForge 和 Modrinth 两大平台 API
//!
//! 模块结构：
//! - types.rs: 数据类型定义（ResourceProject / ResourceVersion 等）
//! - curseforge.rs: CurseForge API 客户端
//! - modrinth.rs: Modrinth API 客户端
//! - searcher.rs: 双平台搜索调度（并行请求 + 去重 + 排序）
//! - tags.rs: 分类标签映射表
//! - mcmod.rs: mcmod.cn 中文译名数据库

pub mod cache;
pub mod curseforge;
pub mod mcmod;
pub mod modrinth;
pub mod searcher;
pub mod secure_storage;
pub mod tags;
pub mod types;

pub use searcher::search;
pub use types::{Platform, ResourceProject, ResourceVersion, ResourceType, SearchParams, SearchResult};
