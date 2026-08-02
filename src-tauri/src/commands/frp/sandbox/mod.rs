//! 安全沙箱：隧道配置校验 + 认证适配器脚本沙箱
//!
//! - `validate_tunnel`：校验用户输入的隧道参数，防止注入和非法值（§7.2）
//! - `run_auth_adapter`：沙箱化执行厂商认证适配器脚本（§7.5）
//!
//! 子模块：validate（隧道参数校验）/ adapter（脚本执行/超时/权限）。

mod adapter;
mod validate;

pub use adapter::run_auth_adapter;
pub use validate::{validate_tunnel, validate_tunnel_update};

#[cfg(test)]
#[path = "../sandbox_tests.rs"]
mod tests;
