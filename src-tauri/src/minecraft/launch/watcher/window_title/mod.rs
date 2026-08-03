//! 游戏窗口标题修改（跨平台）
//!
//! 启动后轮询找到 MC 进程的窗口并改写标题，支持 `{date}` / `{time}` 实时替换。
//! 平台实现：Windows 用 Win32 `SetWindowTextW`，macOS 用 osascript，Linux 用 wmctrl/xdotool。

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod manager;
#[cfg(windows)]
mod windows;

pub use manager::apply_window_title;
