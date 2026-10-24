//! 崩溃规则引擎模块
//!
//! - `schema`：规则数据结构
//! - `engine`：匹配与产出
//!
//! 规则**随版本发布**：规则库编译进二进制，不做云端下发、不做运行时热更新。
//! 改规则 = 改 resources/crash_rules.json + 发版，避免线上规则被随手改掉后
//! 出现「同一个崩溃昨天说是内存、今天说是驱动」这类无法复现的问题。

mod engine;
pub mod evidence;
pub mod schema;
mod special;

pub use engine::{analyze, analyze_with_pending, rules_count, rules_version};
