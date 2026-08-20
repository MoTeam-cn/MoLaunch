//! 加载器损坏检测与自动重装
//!
//! `detect_loader_damage` 检查版本 JSON 中加载器库文件是否缺失/损坏；
//! `repair_version_loader` 检测到损坏后复用 `install_single_loader` 重装加载器，
//! 并将新生成的加载器库合并回当前版本 JSON。

mod detect;
mod merge;
mod repair;

pub use detect::detect_loader_damage;
pub use repair::repair_version_loader;

// 测试经 super::* 访问子模块实现
#[cfg(test)]
pub(crate) use crate::minecraft::loaders::LoaderType;
#[cfg(test)]
pub(crate) use crate::minecraft::version::state::VersionType;
#[cfg(test)]
pub(crate) use detect::{find_loader_lib, json_lib_local_path, loader_lib_pattern};
#[cfg(test)]
pub(crate) use merge::{
    merge_argument_arrays, merge_fields, merge_libraries_dedup, merge_loader_json_into,
    merge_minecraft_args,
};
#[cfg(test)]
pub(crate) use repair::fresh_loader_dir_name;
#[cfg(test)]
pub(crate) use std::path::Path;

#[cfg(test)]
#[path = "repair_loader_test.rs"]
mod tests;
