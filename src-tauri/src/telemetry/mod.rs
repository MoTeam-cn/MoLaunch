//! 匿名错误上报（阿里云 SLS WebTracking）
//!
//! 只上报 ERROR 日志与 panic：脱敏 + 截断 + 同签名去重 + 每日限流，
//! 崩溃前写本地队列、下次启动补发；未配置 Project/Logstore 时仅入队不发送。

mod config;
mod reporter;
mod signature;
mod sink;
mod store;
mod types;

pub use reporter::{
    flush, is_enabled, push_log_line, record_error, record_panic, record_startup, set_enabled,
    spawn_background, status,
};

#[cfg(test)]
mod telemetry_test;
