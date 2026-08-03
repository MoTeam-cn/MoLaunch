//! 认证模块统一分发逻辑（auth 域 meta_manager 入口）

mod authlib;
mod dispatcher;
mod microsoft;
mod offline;

pub use dispatcher::dispatch;
