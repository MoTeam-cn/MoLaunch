//! AppData 缓存模块 - 管理 `%APPDATA%/.minecraft/runtime/` 目录
//!
//! Java Runtime 存到官启默认 .minecraft 目录下，跨游戏目录共享、与官启兼容。
//! 设计与 `Storage` / `Cache` 一致：全局单例 + OnceLock 懒加载。

use std::path::PathBuf;
use std::sync::OnceLock;

/// runtime 子目录名（Java Runtime 存放处）
const SUBDIR_RUNTIME: &str = "runtime";
/// .minecraft 根目录名
const DIR_MINECRAFT: &str = ".minecraft";

static CACHE_APP: OnceLock<CacheApp> = OnceLock::new();

/// AppData 缓存组件
pub struct CacheApp {
    base_dir: PathBuf,
}

impl CacheApp {
    /// 获取全局单例
    ///
    /// Windows 上使用 `%APPDATA%/.minecraft/` 作为基础目录。
    /// 若 APPDATA 环境变量不可用，初始化仍会成功（延迟到调用时报错），
    /// 以保证 `instance()` 永不失败。
    pub fn instance() -> &'static CacheApp {
        CACHE_APP.get_or_init(|| {
            let base_dir = std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::new())
                .join(DIR_MINECRAFT);
            CacheApp { base_dir }
        })
    }

    /// 基础目录（`%APPDATA%/.minecraft/`）
    pub fn dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// runtime 根目录（`%APPDATA%/.minecraft/runtime/`）
    pub fn runtime_base_dir(&self) -> PathBuf {
        self.base_dir.join(SUBDIR_RUNTIME)
    }

    /// 获取指定 component 的 Java Runtime 目录
    ///
    /// 路径：`%APPDATA%/.minecraft/runtime/{component}/`
    pub fn runtime_dir(&self, component: &str) -> Result<PathBuf, String> {
        if self.base_dir.as_os_str().is_empty() {
            return Err("无法获取 APPDATA 环境变量".to_string());
        }
        Ok(self.runtime_base_dir().join(component))
    }

    /// 确保指定 component 的 Java Runtime 目录存在，返回完整路径
    pub fn ensure_runtime_dir(&self, component: &str) -> Result<PathBuf, String> {
        let path = self.runtime_dir(component)?;
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|e| format!("创建 runtime 目录失败: {}: {}", path.display(), e))?;
        }
        Ok(path)
    }
}
