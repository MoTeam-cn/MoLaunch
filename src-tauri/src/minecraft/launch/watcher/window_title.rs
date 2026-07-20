//! 游戏窗口标题修改（跨平台）
//!
//! 参考 PCL2 ModWatcher.vb 第 62-101 行：
//! 启动后轮询找到属于 MC 进程的窗口，改写标题。
//! 支持 `{date}` 和 `{time}` 实时替换。
//!
//! 平台实现：
//! - Windows：Win32 `SetWindowTextW`（直接 API 调用，最可靠）
//! - macOS：AppleScript `osascript`（通过 System Events 按进程名改标题）
//! - Linux：`wmctrl` 或 `xdotool` 外部命令（需用户安装，Wayland 下可能不支持）

use std::time::Duration;

/// 轮询找到 MC 进程的窗口并改写标题
///
/// 参考 PCL2 ModWatcher.vb：
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
    // 参考 PCL2 ModWatcher.vb：每次轮询都改写，支持 {date}/{time} 实时替换
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

// ============================================================================
// Windows 实现：Win32 API（最可靠，无需外部依赖）
// ============================================================================

#[cfg(windows)]
mod windows_impl {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::System::Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        SetWindowTextW,
    };

    /// 传递给 EnumWindows 回调的数据
    struct EnumData {
        /// Java 进程启动时间（用于匹配 MC 窗口，兼容子进程）
        java_start_time: Option<i64>,
        /// 找到的窗口句柄
        found_hwnd: Option<isize>,
    }

    /// 检查指定 PID 的进程是否有可见的 MC 窗口
    pub async fn is_window_visible(pid: u32) -> bool {
        let java_start_time = get_process_creation_time(pid);
        let mut data = EnumData {
            java_start_time,
            found_hwnd: None,
        };
        unsafe {
            let _ = EnumWindows(Some(enum_proc), LPARAM(&mut data as *mut _ as isize));
        }
        data.found_hwnd.is_some()
    }

    /// 改写 MC 窗口标题
    pub async fn set_window_title(pid: u32, title: &str) {
        let java_start_time = get_process_creation_time(pid);
        let mut data = EnumData {
            java_start_time,
            found_hwnd: None,
        };
        unsafe {
            let _ = EnumWindows(Some(enum_proc), LPARAM(&mut data as *mut _ as isize));
        }
        if let Some(hwnd_raw) = data.found_hwnd {
            let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                let hwnd = HWND(hwnd_raw as usize as *mut core::ffi::c_void);
                let _ = SetWindowTextW(hwnd, windows::core::PCWSTR(title_wide.as_ptr()));
            }
        }
    }

    /// 获取进程创建时间（FILETIME 转为 100ns 单位的 i64）
    fn get_process_creation_time(pid: u32) -> Option<i64> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut creation: i64 = 0;
            let mut exit: i64 = 0;
            let mut kernel: i64 = 0;
            let mut user: i64 = 0;
            let _ = GetProcessTimes(
                handle,
                &mut creation as *mut _ as *mut _,
                &mut exit as *mut _ as *mut _,
                &mut kernel as *mut _ as *mut _,
                &mut user as *mut _ as *mut _,
            );
            let _ = windows::Win32::Foundation::CloseHandle(handle);
            Some(creation)
        }
    }

    /// 获取窗口类名
    fn get_class_name(hwnd: HWND) -> String {
        unsafe {
            let mut buf = [0u16; 512];
            let len = GetClassNameW(hwnd, &mut buf);
            if len > 0 {
                String::from_utf16_lossy(&buf[..len as usize])
            } else {
                String::new()
            }
        }
    }

    /// 获取窗口标题
    fn get_window_text(hwnd: HWND) -> String {
        unsafe {
            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut buf);
            if len > 0 {
                String::from_utf16_lossy(&buf[..len as usize])
            } else {
                String::new()
            }
        }
    }

    /// EnumWindows 回调函数
    /// 参考 PCL2 ModWatcher.vb TryGetMinecraftWindow 第 304-338 行：
    /// 1. 检查类名：GLFW30 / LWJGL / SunAwtFrame
    /// 2. 检查标题：排除 PopupMessageWindow 和以 GLFW 开头的辅助窗口
    /// 3. 检查进程启动时间：窗口进程的启动时间 >= Java 进程启动时间
    extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let data = &mut *(lparam.0 as *mut EnumData);
            if data.found_hwnd.is_some() {
                return BOOL(0); // 已找到，停止枚举
            }

            // ① 检查窗口类名（必须是 MC 使用的三种之一）
            let class_name = get_class_name(hwnd);
            if !matches!(class_name.as_str(), "GLFW30" | "LWJGL" | "SunAwtFrame") {
                return BOOL(1); // 继续枚举
            }

            // ② 检查窗口标题（排除辅助窗口）
            let window_text = get_window_text(hwnd);
            // PCL2 逻辑：允许 FML 开头，排除 PopupMessageWindow 和 GLFW 开头
            if !window_text.starts_with("FML")
                && (window_text == "PopupMessageWindow" || window_text.starts_with("GLFW"))
            {
                return BOOL(1); // 继续枚举
            }

            // ③ 检查可见性
            if !IsWindowVisible(hwnd).as_bool() {
                return BOOL(1); // 继续枚举
            }

            // ④ 获取窗口所属进程 ID
            let mut window_pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

            // ⑤ 比较进程启动时间（兼容子进程，参考 PCL2 ModWatcher.vb 第 329 行）
            // 窗口进程的启动时间必须 >= Java 进程的启动时间
            if let Some(java_start) = data.java_start_time {
                if let Some(window_start) = get_process_creation_time(window_pid) {
                    // 窗口进程启动时间早于 Java 进程 → 不是本局启动的 MC
                    if window_start < java_start {
                        return BOOL(1); // 继续枚举
                    }
                }
            }

            // ⑥ 命中
            data.found_hwnd = Some(hwnd.0 as usize as isize);
            BOOL(0) // 停止枚举
        }
    }
}

// ============================================================================
// macOS 实现：AppleScript（通过 osascript 命令，无需额外 Rust 依赖）
// ============================================================================

#[cfg(target_os = "macos")]
mod macos_impl {
    use std::process::Command;

    /// 检查指定 PID 的进程是否有可见窗口
    /// macOS 上用 osascript 检查进程是否存在窗口
    pub async fn is_window_visible(pid: u32) -> bool {
        // 用 System Events 检查该 PID 进程是否有窗口
        let script = format!(
            r#"tell application "System Events" to count (windows of (first process whose unix id is {}))"#,
            pid
        );
        match Command::new("osascript").arg("-e").arg(&script).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                stdout.parse::<u32>().unwrap_or(0) > 0
            }
            Err(_) => false,
        }
    }

    /// 改写指定 PID 的窗口标题
    /// 通过 AppleScript 的 System Events 修改进程窗口标题
    pub async fn set_window_title(pid: u32, title: &str) {
        // 转义标题中的双引号和反斜杠
        let escaped_title = title.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            r#"tell application "System Events" to set name of (first window of (first process whose unix id is {})) to "{}""#,
            pid, escaped_title
        );
        match Command::new("osascript").arg("-e").arg(&script).output() {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // 静默处理错误（窗口可能已关闭或权限不足）
                    if !stderr.is_empty() {
                        crate::log_warn!("[Watcher] macOS 改写窗口标题失败: {}", stderr.trim());
                    }
                }
            }
            Err(e) => {
                crate::log_warn!("[Watcher] 无法执行 osascript: {}", e);
            }
        }
    }
}

// ============================================================================
// Linux 实现：wmctrl / xdotool 外部命令
// ============================================================================

#[cfg(target_os = "linux")]
mod linux_impl {
    use std::process::Command;

    /// 检查指定 PID 的进程是否有可见窗口
    /// 优先用 wmctrl，其次 xdotool
    pub async fn is_window_visible(pid: u32) -> bool {
        // 尝试用 xdotool 按 PID 搜索窗口
        match Command::new("xdotool")
            .args(["search", "--pid", &pid.to_string(), "--onlyvisible"])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                !stdout.is_empty()
            }
            Err(_) => {
                // xdotool 不存在，尝试 wmctrl（只能列出所有窗口，无法按 PID 过滤）
                // 退而求其次：检查进程是否在运行
                Command::new("ps")
                    .args(["-p", &pid.to_string()])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
        }
    }

    /// 改写指定 PID 的窗口标题
    /// 优先用 xdotool（支持按 PID 查找窗口），其次 wmctrl
    pub async fn set_window_title(pid: u32, title: &str) {
        // 方案 1：xdotool search --pid <pid> 然后 set_window --name
        if let Ok(output) = Command::new("xdotool")
            .args(["search", "--pid", &pid.to_string(), "--onlyvisible"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(window_id) = stdout.lines().next() {
                let _ = Command::new("xdotool")
                    .args(["set_window", "--name", title, window_id])
                    .output();
                return;
            }
        }

        // 方案 2：wmctrl -r "旧标题" -T "新标题"（需要知道旧标题，不太可靠）
        // wmctrl 无法按 PID 查找窗口，这里用窗口类名 "Minecraft" 作为 fallback
        let _ = Command::new("wmctrl")
            .args(["-r", "Minecraft", "-T", title])
            .output();
    }
}

// ============================================================================
// 平台分发
// ============================================================================

#[cfg(windows)]
use windows_impl::{is_window_visible, set_window_title};

#[cfg(target_os = "macos")]
use macos_impl::{is_window_visible, set_window_title};

#[cfg(target_os = "linux")]
use linux_impl::{is_window_visible, set_window_title};
