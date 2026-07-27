//! 通用工具模块
//!
//! 包含跨业务的纯函数工具（如 markdown 表格解析、缓存访问）。
//!
//! ## 缓存工具
//!
//! 三个缓存工具模块分别对应三种不同的缓存位置：
//!
//! | 模块 | 位置 | 用途 |
//! |------|------|------|
//! | `cache` | `.Molaunch/cache/` | 运行路径缓存（图片、安装器、预加载、嵌入资源） |
//! | `cache_temp` | `<temp>/MoLaunch/` | 系统临时目录（TaskTemp 安装包、SDK 动态库） |
//! | `cache_app` | `%APPDATA%/.minecraft/runtime/` | AppData 缓存（Java Runtime，Mojang 官方位置） |
//!
//! 所有业务模块应通过 `utils::cache*` 访问缓存，而非直接使用 `storage::cache*`
//! 或拼接 `std::env::temp_dir()` / `std::env::var("APPDATA")`。

pub mod cache;
pub mod cache_app;
pub mod cache_cleanup;
pub mod cache_stats;
pub mod cache_temp;
pub mod community_manager;
pub mod config_manager;
pub mod datetime;
pub mod dispatcher;
pub mod format;
pub mod fs;
pub mod image_cache_manager;
pub mod java_manager;
pub mod markdown_table;
pub mod meta_manager;
pub mod online_manager;
pub mod path;
pub mod plugins_manager;
pub mod signaling_manager;
pub mod sdk_manager;
pub mod skin_manager;
pub mod system_manager;
pub mod tun_manager;
pub mod version;
pub mod version_export_manager;
pub mod version_install_manager;
pub mod version_launch_manager;
pub mod version_list_manager;
pub mod version_mods_manager;
pub mod version_progress_manager;
