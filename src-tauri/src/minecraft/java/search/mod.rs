//! Java 搜索模块
//!
//! 入口：平台扫描收集候选路径（platform）→ 验证并排序（version）。

mod entry;
mod platform;
pub(crate) mod priority;
mod version;

pub use entry::{search_java, search_java_with_paths};
