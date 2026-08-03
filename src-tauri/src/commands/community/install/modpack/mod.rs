//! 整合包安装命令入口（install_modpack / install_local_modpack / preview_local_modpack）
//!
//! 子模块：online（在线下载安装）/ local（拖拽安装）/ shared（共用辅助逻辑）/ api（入口逻辑）

mod api;
mod local;
mod online;
mod shared;

pub use api::preview_local_modpack;
pub use local::install_local_modpack;
pub use online::install_modpack;

pub(super) use api::InstallGuard;