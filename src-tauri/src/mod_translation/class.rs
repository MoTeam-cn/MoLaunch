//! class 文件处理：常量池解析、候选判定、跨文件聚合、UTF8 改写

mod classify;
mod discover;
mod pool;
mod rewrite;

pub use classify::classify_class_text;
pub use discover::{deterministic_class_exclusion_reason, discover_class_candidates};
pub use pool::{class_string_constants, parse_class_constant_pool};
pub use rewrite::replace_class_utf8;

#[cfg(test)]
mod class_test;
