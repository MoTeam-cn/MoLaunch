//! 版本 Setup 模块
//!
//! 管理每个版本的 setup.ini 文件，记录版本元数据（加载器类型、版本号等）。
//! 参考 PCL2 的 setup.ini 机制。
//!
//! 按关注点拆分为子模块：
//! - `types`     PersonalizationUpdate + VersionSetup（含 4 个分组子 struct）+ new/empty 构造
//! - `helpers`   parse_ini / extract_maven_version / read_setup_* / detect_* + file_path/exists
//! - `save`      save / save_full / save_with_options / ensure_complete
//! - `load`      load / load_or_create / from_version_json
//! - `update`    update_personalization
//! - `tests`     单元测试

#[cfg(test)]
mod tests;

mod helpers;
mod load;
mod save;
mod types;
mod update;

pub use helpers::{detect_version_and_loader, read_mc_version_from_json, read_setup_version_and_loader};
pub use types::{PersonalizationUpdate, VersionSetup};
