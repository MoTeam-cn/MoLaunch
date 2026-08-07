//! FRP 安全沙箱模块：校验隧道参数，并隔离执行认证适配器脚本。

mod adapter;
mod validate;

pub use adapter::run_auth_adapter;
pub use validate::{validate_tunnel, validate_tunnel_update};

#[cfg(test)]
#[path = "../sandbox_tests.rs"]
mod tests;
