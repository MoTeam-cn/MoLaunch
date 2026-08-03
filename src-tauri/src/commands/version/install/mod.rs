//! install_merged：整合安装入口（MC 本体 + 加载器 + Fabric API + 后处理）
//! 编排流程由 `flow.rs` 承载，各阶段详细实现拆分到对应子模块，本文件仅聚合入口。

pub mod cleanup;
mod download;
mod fabric;
mod fabric_api;
pub mod flow;
mod forge;
pub(crate) mod loader_helpers;
mod neoforge;
mod post_install;
mod setup_persist;
mod stages;
pub mod version_naming;

pub use flow::install_merged;
