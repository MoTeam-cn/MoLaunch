//! 资源包管理：列出资源包并在 ZIP 与目录格式间转换。
//! 目录解析与路径处理由 helpers 统一提供。

mod convert;
mod explore;
mod helpers;
mod list;

pub use convert::convert;
pub use explore::{
    rp_export, rp_open, rp_pack_format_info, rp_read, rp_read_many, rp_version_pack_format,
    rp_write,
};
pub use list::list;

// 私有 use：保持子模块 `use super::{resolve_packs_dir, path_to_string};` 引用可用
use helpers::{path_to_string, resolve_packs_dir};
