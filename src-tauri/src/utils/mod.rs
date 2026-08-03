//! 通用工具模块
//!
//! 包含跨业务的纯函数工具（markdown 表格解析、缓存访问等）。
//! 业务模块应通过 `utils::cache*` 访问缓存，而非直接使用 `storage::cache*`。

pub mod cache;
pub mod cache_app;
pub mod cache_cleanup;
pub mod cache_stats;
pub mod cache_temp;
pub mod client_type;
pub mod datetime;
pub mod dispatcher;
pub mod format;
pub mod fs;
pub mod hash;
pub mod markdown_table;
pub mod path;
pub mod sdk_crypto;
pub mod signaling_manager;
pub mod tun_manager;
pub mod version;