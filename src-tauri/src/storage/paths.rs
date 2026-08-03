//! 存储路径解析：base_dir 及各子目录路径

use super::Storage;
use std::path::PathBuf;

impl Storage {
    pub(super) fn resolve_base_dir() -> PathBuf {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join(".Molaunch");
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join(".Molaunch");
        }
        PathBuf::from(".Molaunch")
    }

    pub fn config_path(&self) -> PathBuf {
        self.base_dir.join("config.ini")
    }

    pub fn instance_path(&self) -> PathBuf {
        self.base_dir.join("instance.ini")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.base_dir.join("logs")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.base_dir.join("cache")
    }

    pub fn temp_dir(&self) -> PathBuf {
        self.base_dir.join("temp")
    }

    /// 外部下载工具的默认保存目录（.Molaunch/Download/）
    pub fn download_dir(&self) -> PathBuf {
        self.base_dir.join("Download")
    }
}
