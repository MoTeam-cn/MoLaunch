//! 窗口管理外部命令（macOS / Linux）
//!
//! 封装 `window_title` 模块用到的外部命令，避免业务代码直接调用 `std::process::Command`。
//! 所有命令均带 `[Shell]` 前缀日志。

#[cfg(unix)]
use crate::log_info;

#[cfg(unix)]
use super::exec::shell_err;

/// macOS：运行 AppleScript（osascript -e <script>）
///
/// 返回 stdout（已 trim）。脚本失败时返回错误字符串（含 stderr）。
/// 需要用户在"系统设置 > 隐私与安全性 > 辅助功能"中授权启动器。
#[cfg(target_os = "macos")]
pub fn run_osascript(script: &str) -> Result<std::process::Output, String> {
    log_info!("[Shell] osascript -e <script>");
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| shell_err("osascript", e))
}

/// Linux：xdotool search --pid <pid> [--onlyvisible]
///
/// 返回 stdout（窗口 ID 列表，每行一个）。命令不存在或执行失败返回错误。
#[cfg(target_os = "linux")]
pub fn xdotool_search_pid(pid: u32, only_visible: bool) -> Result<std::process::Output, String> {
    log_info!(
        "[Shell] xdotool search --pid {} (only_visible={})",
        pid,
        only_visible
    );
    let mut cmd = std::process::Command::new("xdotool");
    cmd.arg("search").arg("--pid").arg(pid.to_string());
    if only_visible {
        cmd.arg("--onlyvisible");
    }
    cmd.output().map_err(|e| shell_err("xdotool search", e))
}

/// Linux：xdotool set_window --name <title> <window_id>
#[cfg(target_os = "linux")]
pub fn xdotool_set_window_name(
    window_id: &str,
    title: &str,
) -> Result<std::process::Output, String> {
    log_info!(
        "[Shell] xdotool set_window --name '{}' {}",
        title,
        window_id
    );
    std::process::Command::new("xdotool")
        .args(["set_window", "--name", title, window_id])
        .output()
        .map_err(|e| shell_err("xdotool set_window", e))
}

/// Linux：wmctrl -l -p（列出所有窗口，含 PID）
#[cfg(target_os = "linux")]
pub fn wmctrl_list() -> Result<std::process::Output, String> {
    log_info!("[Shell] wmctrl -l -p");
    std::process::Command::new("wmctrl")
        .args(["-l", "-p"])
        .output()
        .map_err(|e| shell_err("wmctrl -l", e))
}

/// Linux：wmctrl -r <old_title> -T <new_title>
#[cfg(target_os = "linux")]
pub fn wmctrl_rename(old_title: &str, new_title: &str) -> Result<std::process::Output, String> {
    log_info!("[Shell] wmctrl -r '{}' -T '{}'", old_title, new_title);
    std::process::Command::new("wmctrl")
        .args(["-r", old_title, "-T", new_title])
        .output()
        .map_err(|e| shell_err("wmctrl -r", e))
}

/// Linux：ps -p <pid>（检查进程是否存在）
///
/// 返回 true 表示进程存在（exit code 0），false 表示进程不存在或 ps 不可用。
#[cfg(target_os = "linux")]
pub fn ps_pid_exists(pid: u32) -> bool {
    log_info!("[Shell] ps -p {}", pid);
    std::process::Command::new("ps")
        .args(["-p", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
