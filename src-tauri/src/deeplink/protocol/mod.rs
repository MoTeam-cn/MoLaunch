//! 深度链接协议注册 / 卸载 / 状态查询工具（跨平台，供便携版运行时注册协议）
//! 平台实现按 cfg 拆分为 windows.rs（HKCU 注册表）与 linux.rs（desktop 文件），
//! 平台无关编排逻辑在 router.rs。

#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

mod router;

#[cfg(any(windows, target_os = "linux"))]
use router::PROTOCOL;
use router::current_exe_path;
pub use router::{auto_register, register, status, unregister, DeeplinkStatus};