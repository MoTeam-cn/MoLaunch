//! install_modpack 阶段辅助：下载原始包、解析 manifest、提取可选 Mod、复制 Logo
//!
//! 子模块：parsers（各格式解析）/ migrate（配置迁移）/ stages（阶段实现）

mod migrate;
mod parsers;
mod stages;

pub(super) use migrate::migrate_modpack_config;
pub(super) use stages::{
    copy_external_logo, download_modpack_archive, extract_optional_mods, parse_modpack_info,
};
