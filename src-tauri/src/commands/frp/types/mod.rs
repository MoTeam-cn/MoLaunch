//! Frp 共享数据类型：隧道、厂商清单、认证配置、日志文件信息、Open API 规范
//!
//! 这些类型在 `frp` 模块及其子模块（provider/install/binary/process/auth/...）间共享，
//! 集中在此处避免循环依赖。`serde` 默认值函数与类型定义放在一起便于维护。

pub use api_spec::*;
pub use auth::*;
pub use provider::*;
pub use tunnel::*;

mod api_spec;
mod auth;
mod provider;
mod tunnel;

#[cfg(test)]
#[path = "../types_tests.rs"]
mod tests;
