//! Modrinth API 客户端（API 文档: https://docs.modrinth.com/）
//!
//! 子模块：http / search / version_files / convert / types。

mod convert;
mod http;
mod search;
mod types;
mod version_files;

pub use search::{
    batch_get_project_slugs, get_project, get_projects_by_slugs, get_versions, search,
};
pub use version_files::{version_files_search, version_files_search_with_downloads};
