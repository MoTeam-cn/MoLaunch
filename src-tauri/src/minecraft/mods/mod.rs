//! Mod 目录索引
//!
//! 只读 `mods/` 下的文件名建立 modid → 显示名索引，不打开 jar（启动与崩溃分析都要求快）。

mod mod_index;

pub use mod_index::{load_mod_index, split_jar_name};

#[cfg(test)]
#[path = "mod_index_test.rs"]
mod mod_index_test;
