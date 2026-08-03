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

pub use api::{UpdateInfo, apply_pending_update, download_and_install, download_update_to_appdata};
pub use check::check_update;