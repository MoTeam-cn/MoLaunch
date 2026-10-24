//! 崩溃规则引擎模块
//!
//! - `schema`：规则数据结构
//! - `engine`：匹配与产出

mod engine;
pub mod schema;

pub use engine::{analyze, rules_count, rules_version};
