//! 模组翻译：保序 JSON 解析/渲染/指针写入（结构化资源写回用）

mod parser;
mod value;

pub use value::JsonValue;

#[cfg(test)]
#[path = "json_value_test.rs"]
mod tests;
