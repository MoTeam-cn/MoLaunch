//! 管理员权限检测与提权重启
//!
//! `is_admin` 检测当前进程是否以管理员权限运行；
//! `relaunch_as_admin` 以管理员权限重新启动当前程序。

use crate::log_info;

#[cfg(unix)]
use super::shell_err;

/// 检查当前进程是否以管理员权限运行
///
/// - Windows: 通过 `OpenProcessToken` + `GetTokenInformation(TokenElevation)` 检测 UAC 提权
/// - Unix: 检查 `id -u` 是否为 0（root）
///
/// 用于 TUN 虚拟网卡创建等需要管理员权限的场景：失败时据此判断是否需要提权重启。
pub fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token: HANDLE = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut ret_len = 0u32;
            let result = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut ret_len,
            );

            let _ = CloseHandle(token);
            result.is_ok() && elevation.TokenIsElevated != 0
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("id")
            .arg("-u")
            .output()
            .map(|o| {
                std::str::from_utf8(&o.stdout)
                    .map(|s| s.trim() == "0")
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

/// 以管理员权限重新启动当前程序
///
/// - Windows: `ShellExecuteW` with verb `"runas"` 触发 UAC 提权对话框
/// - macOS: `osascript` 请求管理员权限执行
/// - Linux: `pkexec` 请求策略Kit 授权
///
/// 调用后当前进程应自行退出，新进程以管理员权限运行。
pub fn relaunch_as_admin(args: &[String]) -> Result<(), String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("获取当前 exe 路径失败: {}", e))?;
    log_info!(
        "[Shell] relaunch_as_admin: {} {:?}",
        exe_path.display(),
        args
    );

    #[cfg(target_os = "windows")]
    {
        use crate::log_error;
        use std::os::windows::ffi::OsStrExt;

        #[allow(clippy::upper_case_acronyms)]
        type HWND = isize;
        #[allow(clippy::upper_case_acronyms)]
        type HINSTANCE = isize;
        const SW_SHOWNORMAL: i32 = 1;

        #[link(name = "shell32")]
        extern "system" {
            fn ShellExecuteW(
                hwnd: HWND,
                lp_operation: *const u16,
                lp_file: *const u16,
                lp_parameters: *const u16,
                lp_directory: *const u16,
                n_show_cmd: i32,
            ) -> HINSTANCE;
        }

        fn to_wide_null(s: &str) -> Vec<u16> {
            std::ffi::OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        }

        let exe_str = exe_path.to_string_lossy().into_owned();
        let verb_wide = to_wide_null("runas");
        let file_wide = to_wide_null(&exe_str);
        let params = args.join(" ");
        let params_wide = if params.is_empty() {
            vec![0u16]
        } else {
            to_wide_null(&params)
        };

        let hinst = unsafe {
            ShellExecuteW(
                0,
                verb_wide.as_ptr(),
                file_wide.as_ptr(),
                params_wide.as_ptr(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };

        // ShellExecuteW 返回值 <= 32 表示错误（用户拒绝 UAC 也会返回错误码）
        if hinst as isize <= 32 {
            let msg = format!("ShellExecuteW runas failed (code: {})", hinst);
            log_error!("[Shell] relaunch_as_admin: {}", msg);
            return Err(msg);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "do shell script \"{} {}\" with administrator privileges",
            exe_path.display(),
            args.join(" ")
        );
        std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn()
            .map_err(|e| shell_err("relaunch_as_admin", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let mut cmd = std::process::Command::new("pkexec");
        cmd.arg(&exe_path);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.spawn().map_err(|e| shell_err("relaunch_as_admin", e))?;
    }

    Ok(())
}
