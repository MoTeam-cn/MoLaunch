//! CurseForge API 客户端（API 文档: https://docs.curseforge.com/）
//!
//! 支持镜像模式（默认走 MCIM 镜像源，无需 API Key）和官方模式（用户配置 API Key 后走官方 API）
//!
//! 模块结构：
//! - `http`：请求层（cf_get/cf_post + source 策略 + 官方/镜像回退）
//! - `types`：API 响应数据结构
//! - `convert`：CF 数据 → 项目通用 ResourceProject/ResourceVersion 转换
//! - `fingerprint`：MurmurHash2 指纹批量查询（整合包安装 / 导出）
//! - `search`：关键词搜索
//! - `project`：工程详情、版本列表、批量 slug 查询

mod convert;
mod fingerprint;
pub(crate) mod http;
mod project;
mod search;
mod types;

pub use fingerprint::{fingerprint_search, fingerprint_search_with_downloads};
pub use project::{batch_get_mod_slugs, get_project, get_versions};
pub use search::search;
