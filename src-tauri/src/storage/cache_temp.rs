//! 系统临时目录缓存模块 - 管理 `<temp>/MoLaunch/` 目录
//!
//! 存放 `TaskTemp/`（安装包临时下载）和 `sdk/`（SDK 动态库释放）。
//! 设计与 `Storage` / `Cache` 一致：全局单例 + OnceLock 懒加载。

use std::path::PathBuf;
use std::sync::OnceLock;

/// TaskTemp 子目录名（安装包临时下载）
const SUBDIR_TASK_TEMP: &str = "TaskTemp";
/// SDK 子目录名（动态库释放）
const SUBDIR_SDK: &str = "sdk";

static CACHE_TEMP: OnceLock<CacheTemp> = OnceLock::new();

/// 系统临时目录缓存组件
pub struct CacheTemp {
    base_dir: PathBuf,
}

impl CacheTemp {
    /// 获取全局单例
    pub fn instance() -> &'static CacheTemp {
        CACHE_TEMP.get_or_init(|| CacheTemp {
            base_dir: std::env::temp_dir().join("MoLaunch"),
        })
    }

    /// 临时缓存根目录（`<temp>/MoLaunch/`）
    pub fn dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// TaskTemp 子目录路径（`<temp>/MoLaunch/TaskTemp/`）
    ///
    /// 用于 Forge/NeoForge 安装包临时下载，安装完成后可清理。
    pub fn task_temp_dir(&self) -> PathBuf {
        self.base_dir.join(SUBDIR_TASK_TEMP)
    }

    /// 确保 TaskTemp 子目录存在，返回完整路径
    pub fn ensure_task_temp_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.task_temp_dir();
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    /// SDK 子目录路径（`<temp>/MoLaunch/sdk/`）
    ///
    /// 用于 SDK 动态库释放，支持热更新（临时目录文件替换）和主程序更新自动覆盖。
    pub fn sdk_dir(&self) -> PathBuf {
        self.base_dir.join(SUBDIR_SDK)
    }

    /// 确保 SDK 子目录存在，返回完整路径
    pub fn ensure_sdk_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.sdk_dir();
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    /// SDK 动态库完整路径（`<temp>/MoLaunch/sdk/<filename>`）
    ///
    /// 传入当前平台的 SDK 文件名（通过 `sdk::get_sdk_filename()` 获取）。
    pub fn sdk_library_path(&self, sdk_filename: &str) -> PathBuf {
        self.sdk_dir().join(sdk_filename)
    }
}
