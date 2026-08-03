//! 双平台搜索调度器
//!
//! 并行调用 CurseForge 和 Modrinth，合并结果、去重、排序；子模块：aggregate / sort。

mod aggregate;
mod sort;

pub use aggregate::PAGE_SIZE;
pub use aggregate::search;
