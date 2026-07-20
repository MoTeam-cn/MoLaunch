//! 游戏窗口标题修改
//!
//! 参考 PCL2 ModWatcher.vb 第 62-101 行：
//! 启动后轮询找到属于 MC 进程的窗口句柄，用 Win32 SetWindowTextW 改写标题。
//! 支持 `{date}` 和 `{time}` 实时替换。

use std::time::Duration;

use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsWindowVisible, SetWindowTextW,
};

/// 传递给 EnumWindows 回调的数据
struct EnumData {
    target_pid: u32,
    found_hwnd: Option<isize>,
}

/// 轮询找到 MC 进程的窗口并改写标题
///
/// 参考 PCL2 ModWatcher.vb：
/// - 等待窗口出现（最多 60 秒，每秒检查一次）
/// - 窗口出现后，每秒 SetWindowText 一次（支持 {date}/{time} 实时替换）
/// - 持续 5 分钟后停止（避免无限循环）
pub async fn apply_window_title(pid: u32, title_template: String) {
    // 阶段 1：等待窗口出现（最多 60 秒）
    // HWND 不是 Send，用 isize 存储原始句柄值
    let mut waited_secs = 0u32;
    let hwnd_raw: isize = loop {
        if let Some(hwnd) = find_window_by_pid(pid) {
            crate::log_info!("[Watcher] 找到游戏窗口 (HWND={})，开始改写标题", hwnd);
            break hwnd;
        }
        waited_secs += 1;
        if waited_secs >= 60 {
            crate::log_warn!("[Watcher] 60 秒内未找到游戏窗口，放弃改写标题");
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    };

    // 阶段 2：持续改写标题（每秒一次，持续 5 分钟）
    // 参考 PCL2 ModWatcher.vb：每次轮询都 SetWindowText，支持 {date}/{time} 实时替换
    let mut elapsed_secs = 0u32;
    loop {
        let title = render_title(&title_template);
        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let hwnd = HWND(hwnd_raw as usize as *mut core::ffi::c_void);
            let _ = SetWindowTextW(hwnd, windows::core::PCWSTR(title_wide.as_ptr()));
        }

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

/// 枚举所有顶层窗口，找到属于指定 PID 的可见窗口
/// 返回窗口句柄的原始 isize 值（HWND 不是 Send，跨 await 需用 isize）
fn find_window_by_pid(pid: u32) -> Option<isize> {
    let mut data = EnumData {
        target_pid: pid,
        found_hwnd: None,
    };
    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut data as *mut _ as isize));
    }
    data.found_hwnd
}

/// EnumWindows 回调函数
extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let data = &mut *(lparam.0 as *mut EnumData);

        let mut window_pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

        if window_pid == data.target_pid && IsWindowVisible(hwnd).as_bool() {
            data.found_hwnd = Some(hwnd.0 as usize as isize); // 存储原始句柄值
            return BOOL(0); // 找到目标，停止枚举
        }
        BOOL(1) // 继续枚举
    }
}
