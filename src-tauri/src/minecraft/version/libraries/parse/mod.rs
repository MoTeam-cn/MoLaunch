//! 库解析：从版本 JSON 解析库 + 规则推导 + 路径解析 + 去重

mod parser;
mod path;
mod rules;

pub use parser::parse_libraries;
pub use rules::{check_rules, is_native_matching_arch};