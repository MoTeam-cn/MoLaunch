//! AppData 全局共享目录辅助模块
//!
//! 集中管理跨启动器实例共享的全局存储路径：
//! - Windows: `%APPDATA%/.Molaunch/`
//! - macOS/Linux: `~/.config/Molaunch/`
//!
//! 与 `<exe_dir>/.Molaunch/`（便携式、每实例独立）相对，本模块目录下的资源
//! 在同一用户的所有 MoLaunch 启动器实例间共享（如 TLS 证书、frpc 二进制、设备凭证等）。
//!
//! 命名历史：早期 `personalization.rs` 与 `online/storage.rs` 误用 `.MolaLaunch`（多了一个 La），
//! 后续 `auth/storage` 跟随。现统一为 `.Molaunch`（与便携式目录、updater last.exe 一致）。
//! 旧路径 `%APPDATA%/.MolaLaunch/` 由 `crate::migrations::appdata_naming` 启动时一次性迁移。

use std::path::PathBuf;

use crate::log_info;

/// AppData 全局共享根目录
///
/// - Windows: `%APPDATA%/.Molaunch/`
/// - macOS/Linux: `~/.config/Molaunch/`
///
/// 环境变量缺失时返回 Err（调用方决定降级策略）。父目录不自动创建，
/// 由 [`ensure_appdata_subdir`] 或调用方按需 `create_dir_all`。
pub fn appdata_root() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "APPDATA environment variable not set".to_string())?;
        Ok(PathBuf::from(appdata).join(".Molaunch"))
    }

    #[cfg(not(windows))]
    {
        let home =
            std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        Ok(PathBuf::from(home).join(".config").join("Molaunch"))
    }
}

/// 解析 AppData 下指定子目录的完整路径（不自动创建）
///
/// 例如 `appdata_subdir("certs")` 返回 `%APPDATA%/.Molaunch/certs/`。
pub fn appdata_subdir(subdir: &str) -> Result<PathBuf, String> {
    Ok(appdata_root()?.join(subdir))
}

/// 确保 AppData 下指定子目录存在，返回其完整路径
///
/// 目录已存在则直接返回；不存在则递归创建。创建失败时返回 Err。
pub fn ensure_appdata_subdir(subdir: &str) -> Result<PathBuf, String> {
    let dir = appdata_subdir(subdir)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建 AppData 子目录失败: {}", e))?;
        log_info!("[AppData] Created subdir: {}", dir.display());
    }
    Ok(dir)
}
