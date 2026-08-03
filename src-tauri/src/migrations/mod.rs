//! 启动时自动迁移模块：由 `Storage::init` 调用 `run_all()` 统一执行
//! 子模块：appdata_naming / portable_to_appdata / online_legacy / common（共享辅助）

mod common;

pub mod appdata_naming;
pub mod online_legacy;
pub mod portable_to_appdata;

pub use common::run_all;