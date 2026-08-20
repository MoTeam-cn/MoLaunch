//! 模组翻译：语言文件读写（JSON / key-value / 结构化 JSON / 自由文本）

mod free_text;
mod json;
mod keyvalue;
mod structured;

pub use free_text::{
    align_free_text, read_localized_target, render_localized_text, snapshot_free_text,
    FreeTextSnapshot,
};
pub use json::{read_json_lang, write_json_lang};
pub use keyvalue::{parse_keyvalue, write_keyvalue};
pub use structured::{apply_structured_strings, collect_structured_strings};

#[cfg(test)]
#[path = "lang_test.rs"]
mod tests;
