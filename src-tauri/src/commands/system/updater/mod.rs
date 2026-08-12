//! 自动更新命令模块（统一入口，平台内部分流）
//! 平台分流逻辑与 `UpdateInfo` 类型位于 `api`（实现），Windows 便携版自实现下载 +
//! 启动 updater.exe 子进程（见 `install_windows`）；macOS/Linux 转发到
//! `tauri-plugin-updater` 官方 plugin（见 `install_unix`）。检查逻辑见 `check`。

mod api;
mod check;
#[cfg(not(target_os = "windows"))]
mod install_unix;
#[cfg(target_os = "windows")]
mod install_windows;

/// 更新下载进度事件名（前端 UpdateDialog 监听，写入 updateState.downloaded/total）
pub(super) const PROGRESS_EVENT: &str = "update-download-progress";

/// 进度事件推送节流阈值（每累计下载 256KB 推送一次，避免高频 IPC 事件压垮前端）
/// 仅官方 plugin 路径（macOS/Linux）使用；Windows 复用 DownloadManager 的 300ms 进度回调
#[cfg(not(target_os = "windows"))]
pub(super) const PROGRESS_THROTTLE_BYTES: u64 = 256 * 1024;

pub use api::{apply_pending_update, download_and_install, download_update_to_appdata, UpdateInfo};
pub use check::check_update;
