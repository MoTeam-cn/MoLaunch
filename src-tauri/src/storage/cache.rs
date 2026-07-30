//! Cache storage module - manages `.Molaunch/cache` directory
//!
//! 所有缓存文件读写都应通过 `Cache::instance()` 进行。
//! 设计与 `Storage` 一致：全局单例 + OnceLock 懒加载。

use std::path::PathBuf;
use std::sync::OnceLock;

use super::Storage;

static CACHE: OnceLock<Cache> = OnceLock::new();

/// 缓存存储组件
pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    /// 获取全局单例
    pub fn instance() -> &'static Cache {
        CACHE.get_or_init(|| Cache {
            cache_dir: Storage::instance().cache_dir(),
        })
    }

    /// 缓存根目录（`.Molaunch/cache`）
    pub fn dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// 拼接缓存子路径（不创建，仅返回路径）
    pub fn path(&self, relative_path: &str) -> PathBuf {
        self.cache_dir.join(relative_path)
    }

    /// 确保缓存子目录存在，返回完整路径
    pub fn ensure_dir(&self, relative_path: &str) -> anyhow::Result<PathBuf> {
        let path = self.cache_dir.join(relative_path);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    /// 判断缓存文件是否存在
    pub fn exists(&self, relative_path: &str) -> bool {
        self.cache_dir.join(relative_path).exists()
    }

    /// 读取缓存文件（文本）
    pub fn read(&self, relative_path: &str) -> anyhow::Result<String> {
        let path = self.cache_dir.join(relative_path);
        Ok(std::fs::read_to_string(&path)?)
    }

    /// 读取缓存文件（二进制）
    pub fn read_bytes(&self, relative_path: &str) -> anyhow::Result<Vec<u8>> {
        let path = self.cache_dir.join(relative_path);
        Ok(std::fs::read(&path)?)
    }

    /// 写入缓存文件（文本），自动创建父目录
    pub fn write(&self, relative_path: &str, content: &str) -> anyhow::Result<()> {
        let path = self.cache_dir.join(relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// 写入缓存文件（二进制），自动创建父目录
    pub fn write_bytes(&self, relative_path: &str, content: &[u8]) -> anyhow::Result<()> {
        let path = self.cache_dir.join(relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// 删除缓存文件（不存在时静默成功）
    pub fn remove(&self, relative_path: &str) -> anyhow::Result<()> {
        let path = self.cache_dir.join(relative_path);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// 列出缓存子目录下的文件名
    pub fn list(&self, relative_path: &str) -> anyhow::Result<Vec<String>> {
        let path = self.cache_dir.join(relative_path);
        let mut entries = Vec::new();
        if path.exists() && path.is_dir() {
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
        Ok(entries)
    }

    /// 清空缓存子目录（删除目录下所有文件，保留目录本身）
    pub fn clear_dir(&self, relative_path: &str) -> anyhow::Result<()> {
        let path = self.cache_dir.join(relative_path);
        if !path.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                std::fs::remove_dir_all(&entry_path)?;
            } else {
                std::fs::remove_file(&entry_path)?;
            }
        }
        Ok(())
    }
}
