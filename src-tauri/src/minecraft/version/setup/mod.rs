//! 版本 Setup 模块
//!
//! 管理每个版本的 setup.ini 文件；按关注点拆分为 types/helpers/save/load/update 子模块。

#[cfg(test)]
mod tests;

mod helpers;
mod load;
mod save;
mod types;
mod update;

pub use helpers::{
    detect_version_and_loader, read_mc_version_from_json, read_setup_version_and_loader,
};
pub use types::{PersonalizationUpdate, VersionSetup};
