//! 导出选项定义 + 动态子选项扫描
//!
//! 提供 ~20 个静态选项 + 资源包/存档/光影包的动态子选项扫描。
//! 选项可见性根据实例目录实际文件决定。
//! 公共构建逻辑（build_all_options / GLOBAL_EXCLUDES / 目录探测）位于 `common`。

mod basic;
mod common;
mod misc;
mod mods;
mod packs;
mod world;

pub use common::build_all_options;
pub use common::GLOBAL_EXCLUDES;
pub(super) use common::{has_file_or_dir, has_licence_file, scan_sub_options};