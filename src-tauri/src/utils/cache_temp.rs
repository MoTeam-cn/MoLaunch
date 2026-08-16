//! 系统临时目录缓存工具（`<temp>/MoLaunch/`）
//!
//! 自由函数式 API，包装 `storage::cache_temp::CacheTemp` 单例。
//! 业务模块应通过本模块访问系统临时目录缓存。

use std::path::PathBuf;

use crate::storage::cache_temp::CacheTemp;

/// 临时缓存根目录（`<temp>/MoLaunch/`）
pub fn dir() -> PathBuf {
    CacheTemp::instance().dir().clone()
}

/// TaskTemp 子目录路径（`<temp>/MoLaunch/TaskTemp/`）
///
/// 用于 Forge/NeoForge 安装包临时下载，安装完成后可清理。
pub fn task_temp_dir() -> PathBuf {
    CacheTemp::instance().task_temp_dir()
}

/// 确保 TaskTemp 子目录存在，返回完整路径
pub fn ensure_task_temp_dir() -> anyhow::Result<PathBuf> {
    CacheTemp::instance().ensure_task_temp_dir()
}

/// SDK 子目录路径（`<temp>/MoLaunch/sdk/`）
///
/// 用于 SDK 动态库释放，支持热更新和主程序更新自动覆盖。
pub fn sdk_dir() -> PathBuf {
    CacheTemp::instance().sdk_dir()
}

/// 确保 SDK 子目录存在，返回完整路径
pub fn ensure_sdk_dir() -> anyhow::Result<PathBuf> {
    CacheTemp::instance().ensure_sdk_dir()
}

/// SDK 动态库完整路径（`<temp>/MoLaunch/sdk/<filename>`）
///
/// 传入当前平台的 SDK 文件名（通过 `crate::sdk::get_sdk_filename()` 获取）。
pub fn sdk_library_path(sdk_filename: &str) -> PathBuf {
    CacheTemp::instance().sdk_library_path(sdk_filename)
}

/// hongshi 子目录路径（`<temp>/MoLaunch/hongshi/`）
///
/// 红石联机内核释放目录，随系统临时目录自动清理；tunnel.ini 状态文件
/// 与内核日志（logs/）均落于此目录。
pub fn hongshi_dir() -> PathBuf {
    CacheTemp::instance().hongshi_dir()
}

/// 确保 hongshi 子目录存在，返回完整路径
pub fn ensure_hongshi_dir() -> anyhow::Result<PathBuf> {
    CacheTemp::instance().ensure_hongshi_dir()
}
