//! Windows 实现：Win32 API（最可靠，无需外部依赖）
//!
//! `EnumWindows` 枚举顶层窗口，按类名（GLFW30/LWJGL/SunAwtFrame）+ 标题过滤
//! + 进程启动时间（兼容子进程）三层匹配，命中后用 `SetWindowTextW` 改写标题。

use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};
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

/// EnumWindows 回调：按类名 → 标题 → 可见性 → 启动时间依次过滤
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
        // 允许 FML 开头，排除 PopupMessageWindow 和 GLFW 开头
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

        // ⑤ 比较进程启动时间（兼容子进程）
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
