//! 自动更新平台分流的编排逻辑实现（原聚合入口 mod.rs 中的实现）
//!
//! `UpdateInfo` 类型与下载/安装/退出时替换三个平台分流函数收敛于此，
//! 具体平台实现在 `install_windows` / `install_unix`，检查逻辑在 `check`。

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[cfg(not(target_os = "windows"))]
use crate::log_info;
use crate::state::AppState;

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
/// - **Windows**：复用通用 DownloadManager 下载 + 启动 updater.exe 子进程
/// - **macOS / Linux**：转发到官方 plugin 的 download_and_install()
pub async fn download_and_install(
    app: &AppHandle,
    state: &AppState,
    info: UpdateInfo,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        super::install_windows::download_and_install_windows(app, state, info).await
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (state, info);
        super::install_unix::download_and_install_unix(app).await
    }
}

/// 后台静默下载新版本到 `%APPDATA%/.Molaunch/last.exe`
///
/// 前端定时检查发现新版本后调用此命令，将安装包下载到 appdata。
/// 下载完成后不立即替换，等用户退出程序时由 `apply_pending_update` 触发替换。
/// 每次定时检查命中都会重新下载覆盖 last.exe（无 size/hash 元数据，不做重复跳过）。
///
/// **平台差异**：Windows 便携版复用通用 DownloadManager 后台预下载（绕过文件锁、支持退出时延迟替换）；
/// macOS / Linux 由官方 `tauri-plugin-updater` 接管，无后台预下载流程，调用此命令
/// 仅记录 INFO 日志后返回 `Ok(false)`，前端应通过 `download_and_install_update` 触发更新。
pub async fn download_update_to_appdata(
    state: &AppState,
    info: UpdateInfo,
) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        super::install_windows::download_update_to_appdata_impl(state, info).await
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (state, info);
        log_info!(
            "[Updater] download_update_to_appdata 在 macOS/Linux 上由 tauri-plugin-updater 接管，无后台预下载"
        );
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
///
/// **平台差异**：仅 Windows 便携版支持退出时延迟替换；macOS / Linux 由官方 plugin
/// 在 `download_and_install_update` 内同步完成下载安装，无退出时替换流程，
/// 调用此命令仅记录 INFO 日志后返回 `Ok(false)`，前端应直接退出主程序。
pub async fn apply_pending_update(app: &AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        super::install_windows::apply_pending_update_impl(app).await
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        log_info!(
            "[Updater] apply_pending_update 在 macOS/Linux 上由 tauri-plugin-updater 接管，无退出时替换"
        );
        Ok(false)
    }
}
