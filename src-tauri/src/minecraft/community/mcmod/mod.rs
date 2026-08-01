//! MC 百科（mcmod.cn）数据库
//! 加载内置 moddata.txt，通过工程 Slug 查找中文译名和 MC 百科 class id。
//! 关键设计：moddata.txt 第 N 行 → mcmod.cn class id = N（空行也占行号）
//! 中文搜索：`search_by_chinese` 用本地模糊匹配把中文关键词映射到 CurseForge/Modrinth Slug，
//! 提取英文单词作为搜索关键词。

mod database;
mod lookup;
mod parsers;
mod search;

// 公开 API re-export（外部调用方通过 mcmod::xxx 访问）
pub use lookup::{lookup_cf, lookup_class_id, lookup_mr, translate};
pub use search::{search_by_chinese, RewriteResult};

// 测试用：内部符号引入父模块命名空间（测试 `use super::*;` 可访问）
#[cfg(test)]
use parsers::{parse_slug_part, process_wildcard};
#[cfg(test)]
use search::extract_words;

#[cfg(test)]
#[path = "../mcmod_tests.rs"]
mod tests;
