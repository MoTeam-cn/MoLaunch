//! AppData 缓存工具（`%APPDATA%/.minecraft/runtime/`）
//!
//! 对外提供自由函数式 API，包装 `storage::cache_app::CacheApp` 单例。
//! 所有业务模块应通过本模块访问 AppData 下的 Java Runtime 目录，
//! 而非直接读取 `APPDATA` 环境变量拼接路径。
//!
//! ## 缓存目录结构
//!
//! ```text
//! %APPDATA%/.minecraft/
//!   runtime/
//!     {component}/  - Java Runtime（Mojang 官方位置，跨游戏目录共享）
//! ```
//!
//! ## 设计说明
//!
//! Java Runtime 存到官启默认 .minecraft 目录下：
//! - 跨游戏目录共享，不随 game_dir 删除而丢失
//! - 与官启兼容，可被官启和本启动器共同使用

use std::path::PathBuf;

use crate::storage::cache_app::CacheApp;

/// 基础目录（`%APPDATA%/.minecraft/`）
pub fn dir() -> PathBuf {
    CacheApp::instance().dir().clone()
}

/// runtime 根目录（`%APPDATA%/.minecraft/runtime/`）
pub fn runtime_base_dir() -> PathBuf {
    CacheApp::instance().runtime_base_dir()
}

/// 获取指定 component 的 Java Runtime 目录
///
/// 路径：`%APPDATA%/.minecraft/runtime/{component}/`
///
/// 返回错误表示 APPDATA 环境变量不可用。
pub fn runtime_dir(component: &str) -> Result<PathBuf, String> {
    CacheApp::instance().runtime_dir(component)
}

/// 确保指定 component 的 Java Runtime 目录存在，返回完整路径
pub fn ensure_runtime_dir(component: &str) -> Result<PathBuf, String> {
    CacheApp::instance().ensure_runtime_dir(component)
}
