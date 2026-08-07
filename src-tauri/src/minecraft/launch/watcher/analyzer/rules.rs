//! 声明式崩溃检测规则
//!
//! 规则类型与静态规则表分离，保持检测器通过本模块访问既有公开项。

mod table;
mod types;

pub use table::KEYWORD_RULES;
#[allow(unused_imports)]
pub use types::{KeywordRule, SourceKind};
