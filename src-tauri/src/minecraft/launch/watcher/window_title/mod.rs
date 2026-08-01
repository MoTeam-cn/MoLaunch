//! 游戏窗口标题修改（跨平台）
//!
//! 启动后轮询找到 MC 进程的窗口并改写标题，支持 `{date}` / `{time}` 实时替换。
//! 平台实现：Windows 用 Win32 `SetWindowTextW`，macOS 用 osascript，Linux 用 wmctrl/xdotool。

use std::time::Duration;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

// 平台分发：根据编译目标选择对应实现
#[cfg(target_os = "linux")]
use linux::{is_window_visible, set_window_title};
#[cfg(target_os = "macos")]
use macos::{is_window_visible, set_window_title};
#[cfg(windows)]
use windows::{is_window_visible, set_window_title};

/// 轮询找到 MC 进程的窗口并改写标题
///
/// - 等待窗口出现（最多 60 秒，每秒检查一次）
/// - 窗口出现后，每秒改写一次（支持 {date}/{time} 实时替换）
/// - 持续 5 分钟后停止（避免无限循环）
pub async fn apply_window_title(pid: u32, title_template: String) {
    // 阶段 1：等待窗口出现（最多 60 秒）
    let mut waited_secs = 0u32;
    loop {
        if is_window_visible(pid).await {
            crate::log_info!("[Watcher] 找到游戏窗口 (PID={})，开始改写标题", pid);
            break;
        }
        waited_secs += 1;
        if waited_secs >= 60 {
            crate::log_warn!("[Watcher] 60 秒内未找到游戏窗口，放弃改写标题");
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // 阶段 2：持续改写标题（每秒一次，持续 5 分钟）
    // 每次轮询都改写，支持 {date}/{time} 实时替换
    let mut elapsed_secs = 0u32;
    loop {
        let title = render_title(&title_template);
        set_window_title(pid, &title).await;

        elapsed_secs += 1;
        if elapsed_secs >= 300 {
            break; // 5 分钟后停止
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// 渲染标题模板：替换 {date} 和 {time}
fn render_title(template: &str) -> String {
    let now = chrono::Local::now();
    template
        .replace("{date}", &now.format("%Y/%-m/%-d").to_string())
        .replace("{time}", &now.format("%H:%M:%S").to_string())
}
