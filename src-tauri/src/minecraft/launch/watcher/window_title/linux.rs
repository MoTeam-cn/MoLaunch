//! Linux 实现：wmctrl / xdotool 外部命令（通过 shell 模块统一调用）
//!
//! 优先 xdotool（按 PID 查找），fallback 到 wmctrl（按标题匹配）。
//! Wayland 下多数合成器禁止修改其他窗口标题，这是 Wayland 设计限制。

use crate::minecraft::system::shell::{
    ps_pid_exists, wmctrl_list, wmctrl_rename, xdotool_search_pid, xdotool_set_window_name,
};

/// 检查指定 PID 的进程是否有可见窗口
/// 优先用 xdotool（支持按 PID 查找），fallback 到 wmctrl + 进程检查
pub async fn is_window_visible(pid: u32) -> bool {
    // 方案 1：xdotool search --pid
    if let Ok(output) = xdotool_search_pid(pid, true) {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !stdout.is_empty() {
            return true;
        }
    }

    // 方案 2：wmctrl -l 列出所有窗口，检查是否有窗口标题包含 "Minecraft"
    if let Ok(output) = wmctrl_list() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            // wmctrl -l -p 输出格式：窗口ID 主机名 PID 标题
            let parts: Vec<&str> = line.splitn(4, char::is_whitespace).collect();
            if parts.len() >= 3 {
                if let Ok(window_pid) = parts[2].parse::<u32>() {
                    if window_pid == pid {
                        return true;
                    }
                }
            }
        }
    }

    // 方案 3：检查进程是否在运行（最后的 fallback）
    ps_pid_exists(pid)
}

/// 改写指定 PID 的窗口标题（xdotool 优先，wmctrl 兜底）
pub async fn set_window_title(pid: u32, title: &str) {
    // 方案 1：xdotool search --pid <pid> 然后 set_window --name
    if let Ok(output) = xdotool_search_pid(pid, true) {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Some(window_id) = stdout.lines().next() {
            match xdotool_set_window_name(window_id, title) {
                Ok(o) if o.status.success() => return,
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    if !stderr.is_empty() {
                        crate::log_warn!("[Watcher] xdotool set_window 失败: {}", stderr.trim());
                    }
                }
                Err(_) => {}
            }
        }
    }

    // 方案 2：wmctrl -r "旧标题" -T "新标题"
    // wmctrl 无法按 PID 查找窗口，需要知道旧标题
    // 先用 wmctrl -l 找到该 PID 的窗口标题，再改
    if let Ok(output) = wmctrl_list() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(4, char::is_whitespace).collect();
            if parts.len() >= 4 {
                if let Ok(window_pid) = parts[2].parse::<u32>() {
                    if window_pid == pid {
                        let old_title = parts[3];
                        let _ = wmctrl_rename(old_title, title);
                        return;
                    }
                }
            }
        }
    }

    // 两个工具都不可用（可能是 Wayland 环境）
    static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        crate::log_warn!(
            "[Watcher] Linux 无法修改窗口标题（可能缺少 xdotool/wmctrl，或运行在 Wayland 环境下）"
        );
    }
}
