//! NBT 数据查看与编辑（level.dat / playerdata / region .mca 等）
//! 用 `fastnbt` crate 解析/序列化，`flate2` 处理压缩。
//! 子模块：api（公共命令）/ mca / convert / scan / compress。

mod api;
mod compress;
mod convert;
mod mca;
mod scan;

pub use api::{list_save_files, parse, save};

#[cfg(test)]
#[path = "nbt_test.rs"]
mod nbt_test;
