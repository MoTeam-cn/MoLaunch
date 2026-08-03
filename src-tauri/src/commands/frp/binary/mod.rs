//! frpc 二进制下载与管理（从原 `binary.rs` 拆分）
//! 系统默认厂商 frpc：从 apiServer `/v1/frp/manifest` 获取最新版本下载 URL；
//! 外部厂商 frpc：按 `manifest.binary.distribution` 处理（bundled 仅校验，url 下载）。
//! 子模块：system_default / external / archive / manager（入口编排）。

mod archive;
mod external;
mod manager;
mod system_default;

pub(crate) use external::host_matches;
pub use manager::ensure_frpc;
pub use system_default::fetch_latest_frpc_version;
