//! 自动更新命令模块（统一入口，平台内部分流）
//! Windows 便携版：自实现下载 + 启动 updater.exe 子进程替换 exe（绕过 Windows 文件锁，
//! 无需 NSIS installer）；macOS/Linux：转发到 `tauri-plugin-updater` 官方 plugin（复用其
//! 下载/验签/替换/重启全流程）。前端通过 `system_manager` 统一调用 `check_update`/
//! `download_and_install_update`。See: docs/updater/design.md §4 Windows 便携版 updater

mod check;
#[cfg(not(target_os = "windows"))]
mod install_unix;
#[cfg(target_os = "windows")]
mod install_windows;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

pub use check::check_update;

/// 更新信息（check_update 返回，download_and_install_update 接收）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct UpdateInfo {
    /// 是否有可用更新
    pub available: bool,
    /// 新版本号
    pub version: String,
    /// 更新日志
    pub notes: String,
    /// 是否强制更新（来自 manifest 扩展字段 force_update）
    pub force_update: bool,
    /// 下载 URL（presigned URL，Windows 自实现下载用）
    #[serde(default)]
    pub download_url: String,
    /// 签名（base64，Windows 预留验签用）
    #[serde(default)]
    pub signature: String,
}

/// 下载并安装更新（平台内部分流）
///
/// - **Windows**：自实现下载 + 启动 updater.exe 子进程
/// - **macOS / Linux**：转发到官方 plugin 的 download_and_install()
pub async fn download_and_install(app: &AppHandle, info: UpdateInfo) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        install_windows::download_and_install_windows(app, info).await
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = info;
        install_unix::download_and_install_unix(app).await
    }
}

/// 后台静默下载新版本到 `%APPDATA%/.Molaunch/last.exe`
///
/// 前端定时检查发现新版本后调用此命令，将安装包下载到 appdata。
/// 下载完成后不立即替换，等用户退出程序时由 `apply_pending_update` 触发替换。
///
/// 若 last.exe 已存在且版本相同（通过文件大小判断），跳过重复下载。
pub async fn download_update_to_appdata(info: UpdateInfo) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        install_windows::download_update_to_appdata_impl(info).await
    }

    #[cfg(not(target_os = "windows"))]
    {
        // macOS/Linux 使用官方 plugin，不需要后台下载
        let _ = info;
        Ok(false)
    }
}

/// 退出时检查并应用待安装更新
///
/// 检查 `%APPDATA%/.Molaunch/last.exe` 是否存在：
/// - 存在：释放 updater.exe，启动替换子进程，返回 true（调用方应随后退出主程序）
/// - 不存在：无待安装更新，返回 false（正常退出）
///
/// 前端在窗口 close 事件中调用此命令，返回 true 则让主程序退出由 updater.exe 接管。
pub async fn apply_pending_update(app: &AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        install_windows::apply_pending_update_impl(app).await
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Ok(false)
    }
}
