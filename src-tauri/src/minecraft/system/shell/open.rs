//! 文件管理器与 URL 打开
//!
//! 封装 `open_path`/`open_url`/`reveal_in_file_manager`，处理跨平台差异与安全校验
//! （拒绝路径遍历/UNC）。`validate_path` 为本模块私有辅助。

use crate::{log_error, log_info};

use super::shell_err;

/// 安全校验：拒绝路径遍历（..）和 UNC 路径（防止 SMB 认证泄露）
fn validate_path(path: &str) -> Result<(), String> {
    if path.contains("..") {
        return Err("路径不能包含 ..".to_string());
    }
    if path.starts_with("\\\\") || path.starts_with("//") {
        return Err("不支持 UNC 路径".to_string());
    }
    if !std::path::Path::new(path).exists() {
        return Err(format!("路径不存在: {}", path));
    }
    Ok(())
}

/// 用系统文件管理器打开文件夹
///
/// - Windows: `cmd /c start "" "<path>"`（不能用 `explorer <path>`：
///   Rust Command::arg 会给含空格的路径自动加引号，explorer.exe 对带引号的
///   裸路径解析失败会回退到打开"文档"库。start 命令正确处理带引号路径）
/// - macOS: `open <path>`
/// - Linux: `xdg-open <path>`
pub fn open_path(path: &str) -> Result<(), String> {
    validate_path(path)?;
    log_info!("[Shell] open_path: {}", path);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("cmd")
            .args(["/c", "start", "", path])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| shell_err("open_path", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| shell_err("open_path", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| shell_err("open_path", e))?;
    }

    Ok(())
}

/// 用系统默认浏览器打开 URL
///
/// 仅允许 http/https 协议，防止任意协议跳转（如 file://、javascript:）。
/// - Windows: `cmd /c start "" "<url>"`
/// - macOS: `open <url>`
/// - Linux: `xdg-open <url>`
///
/// 与 `open_path` 区别：不校验路径存在性（URL 不是文件系统路径），
/// 仅校验协议白名单。用于 Frp OAuth2 授权跳转、Device Code 验证链接等场景。
pub fn open_url(url: &str) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("仅允许打开 http/https URL".to_string());
    }
    log_info!("[Shell] open_url: {}", url);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| shell_err("open_url", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| shell_err("open_url", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| shell_err("open_url", e))?;
    }

    Ok(())
}

/// 在文件管理器中打开父目录并选中指定文件
///
/// - Windows: 用 Win32 API `ShellExecuteW` 直接调用 explorer.exe，传参 `/select,"<path>"`
///   （不能用 `Command::new("explorer").arg(...)`：Rust Command 在 Windows 上
///   会对含空格的参数加引号并转义内部引号，导致 explorer 收到错误参数；
///   也不能用 `cmd /c explorer /select,"<path>"`：cmd.exe 不识别 `\"` 转义，
///   Rust 转义后的引号会变成字面字符，explorer 解析失败回退到默认位置"此电脑"。
///   ShellExecuteW 直接构造 UTF-16 命令行，绕过 Rust 的转义，explorer 正确解析）
/// - macOS: `open -R <path>`
/// - Linux: 无统一选中接口，回退到打开父目录
pub fn reveal_in_file_manager(path: &str) -> Result<(), String> {
    validate_path(path)?;
    log_info!("[Shell] reveal_in_file_manager: {}", path);

    #[cfg(target_os = "windows")]
    {
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

        // 将 &str 转为以 null 结尾的 UTF-16 字符串（ShellExecuteW 要求宽字符）
        fn to_wide_null(s: &str) -> Vec<u16> {
            std::ffi::OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        }

        // explorer.exe 的 /select 语法：/select,"<path>"
        // explorer 用 CommandLineToArgvW 解析参数，识别引号并合并内容
        let params = format!("/select,\"{}\"", path);
        let file_wide = to_wide_null("explorer.exe");
        let params_wide = to_wide_null(&params);

        let hinst = unsafe {
            ShellExecuteW(
                0,
                std::ptr::null(),
                file_wide.as_ptr(),
                params_wide.as_ptr(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };

        // ShellExecuteW 返回值 <= 32 表示错误（详见 SE_ERR_* 常量）
        if hinst as isize <= 32 {
            let msg = format!("ShellExecuteW failed (code: {})", hinst);
            log_error!("[Shell] reveal_in_file_manager: {}", msg);
            return Err(msg);
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", path])
            .spawn()
            .map_err(|e| shell_err("reveal_in_file_manager", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let p = std::path::Path::new(path);
        let parent = p.parent().unwrap_or(std::path::Path::new("."));
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| shell_err("reveal_in_file_manager", e))?;
    }

    Ok(())
}
