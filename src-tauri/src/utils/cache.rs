//! 运行路径缓存工具（`.Molaunch/cache/`）
//!
//! 对外提供自由函数式 API，包装 `storage::cache::Cache` 单例。
//! 所有业务模块应通过本模块访问运行路径缓存，而非直接使用 `storage::cache::Cache`。
//!
//! ## 缓存目录结构
//!
//! ```text
//! .Molaunch/cache/
//!   images/           - 图片缓存（皮肤、披风、头像）
//!   forge_installer/  - Forge 安装器注入资源
//!   preload_mods/     - 社区资源预加载缓存
//!   <embedded jars>   - 嵌入资源释放
//! ```

use std::path::PathBuf;

use crate::storage::cache::Cache;

/// 缓存根目录（`.Molaunch/cache`）
pub fn dir() -> PathBuf {
    Cache::instance().dir().clone()
}

/// 拼接缓存子路径（不创建，仅返回路径）
pub fn path(relative_path: &str) -> PathBuf {
    Cache::instance().path(relative_path)
}

/// 确保缓存子目录存在，返回完整路径
pub fn ensure_dir(relative_path: &str) -> anyhow::Result<PathBuf> {
    Cache::instance().ensure_dir(relative_path)
}

/// 判断缓存文件是否存在
pub fn exists(relative_path: &str) -> bool {
    Cache::instance().exists(relative_path)
}

/// 读取缓存文件（文本）
pub fn read(relative_path: &str) -> anyhow::Result<String> {
    Cache::instance().read(relative_path)
}

/// 读取缓存文件（二进制）
pub fn read_bytes(relative_path: &str) -> anyhow::Result<Vec<u8>> {
    Cache::instance().read_bytes(relative_path)
}

/// 写入缓存文件（文本），自动创建父目录
pub fn write(relative_path: &str, content: &str) -> anyhow::Result<()> {
    Cache::instance().write(relative_path, content)
}

/// 写入缓存文件（二进制），自动创建父目录
pub fn write_bytes(relative_path: &str, content: &[u8]) -> anyhow::Result<()> {
    Cache::instance().write_bytes(relative_path, content)
}

/// 删除缓存文件（不存在时静默成功）
pub fn remove(relative_path: &str) -> anyhow::Result<()> {
    Cache::instance().remove(relative_path)
}

/// 列出缓存子目录下的文件名
pub fn list(relative_path: &str) -> anyhow::Result<Vec<String>> {
    Cache::instance().list(relative_path)
}

/// 清空缓存子目录（删除目录下所有文件，保留目录本身）
pub fn clear_dir(relative_path: &str) -> anyhow::Result<()> {
    Cache::instance().clear_dir(relative_path)
}
