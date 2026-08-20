//! 模组翻译：任务编排入口（分析 → 翻译 → 重打包）
//!
//! 单任务模型：同一时间仅允许一个翻译任务；分析结果在工作区缓存，
//! 启动翻译时若路径一致则复用，否则重新解包分析。

pub mod analyze;
pub mod class;
pub mod controller;
pub mod error;
pub mod jar;
pub mod json_value;
pub mod lang;
pub mod ledger;
pub mod memory;
pub mod mod_name;
pub mod package;
pub mod progress;
pub mod prompt;
pub mod quality;
pub mod repair;
pub mod resume;
pub mod status;
pub mod task;
pub mod translate_class;
pub mod translate_lang;
pub mod types;

pub use self::controller::{analyze_jar, cancel_task, start_task};
pub(crate) use self::progress::current_stage_progress;
pub use self::status::current_status;
pub use self::types::*;
