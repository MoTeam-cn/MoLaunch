//! CurseForge API 模块：提供项目、版本、搜索与指纹查询，并统一导出公共接口。

mod convert;
mod fingerprint;
pub(crate) mod http;
mod project;
mod search;
mod types;

pub use fingerprint::{fingerprint_search, fingerprint_search_with_downloads};
pub use project::{batch_get_mod_slugs, get_project, get_versions};
pub use search::search;
