//! 日志系统模块
//! 将日志写入 storage/logs 目录，同时可选输出到控制台。
//! 子模块：core（LOGGER 静态实例 + LogLevel + 公开 API）/ viewer（文件查看）/ sanitize（脱敏）

mod core;
mod sanitize;
mod viewer;

pub use core::{init, init_from_config, log, separator, set_level, LogLevel};
pub use sanitize::sanitize_sensitive_info;
pub use viewer::{get_log_path, list_log_files, read_log_file};
