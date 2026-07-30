//! macOS 实现：AppleScript（通过 shell 模块统一调用 osascript）
//!
//! 通过 System Events 按 PID 找到进程窗口并修改其 name 属性。
//! 需用户在"系统设置 > 隐私与安全性 > 辅助功能"中授权启动器；
//! Java 应用窗口由 JVM 管理，可能无法修改（输出警告但不影响游戏运行）。

use crate::minecraft::system::shell::run_osascript;

/// 检查指定 PID 的进程是否有可见窗口
/// macOS 上用 osascript 通过 System Events 检查进程是否有窗口
pub async fn is_window_visible(pid: u32) -> bool {
    let script = format!(
        r#"tell application "System Events" to count (windows of (first process whose unix id is {}))"#,
        pid
    );
    match run_osascript(&script) {
        Ok(output) => {
            if !output.status.success() {
                return false;
            }
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            stdout.parse::<u32>().unwrap_or(0) > 0
        }
        Err(_) => false,
    }
}

/// 改写指定 PID 的窗口标题（通过 AppleScript System Events）
pub async fn set_window_title(pid: u32, title: &str) {
    // 转义标题中的双引号和反斜杠
    let escaped_title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "System Events" to set name of (first window of (first process whose unix id is {})) to "{}""#,
        pid, escaped_title
    );
    match run_osascript(&script) {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    // 常见错误：
                    // - "Not authorized to send Apple events" → 需要辅助功能权限
                    // - "Can't set name of window" → Java 应用不支持修改窗口标题
                    let err = stderr.trim();
                    if err.contains("Not authorized") || err.contains("Apple events") {
                        crate::log_warn!("[Watcher] macOS 修改窗口标题需要辅助功能权限，请在系统设置 > 隐私与安全性 > 辅助功能中授权");
                    } else if err.contains("Can't set name") || err.contains("can't be set") {
                        // Java 应用可能不支持，静默处理避免刷屏
                        // 只在第一次输出警告
                        static WARNED: std::sync::atomic::AtomicBool =
                            std::sync::atomic::AtomicBool::new(false);
                        if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                            crate::log_warn!("[Watcher] macOS Java 应用不支持修改窗口标题（System Events 无法设置 name 属性）");
                        }
                    } else {
                        crate::log_warn!("[Watcher] macOS 改写窗口标题失败: {}", err);
                    }
                }
            }
        }
        Err(e) => {
            crate::log_warn!("[Watcher] 无法执行 osascript: {}", e);
        }
    }
}
