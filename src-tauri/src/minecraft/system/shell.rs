//! 跨平台 shell 命令封装
//!
//! 统一管理系统级外部命令调用（文件管理器、进程管理、文件权限），
//! 避免业务代码直接操作 std::process::Command 造成的行为不一致。
//!
//! 整合范围：
//! - 文件管理器交互：open_path / reveal_in_file_manager（原 game_dir.rs 的 explorer/cmd / open / xdg-open）
//! - 进程管理：kill_process_tree（原 launch.rs 的 taskkill / kill）
//! - 文件权限：restrict_file_permissions（原 script_export.rs 的 icacls / chmod）
//!
//! 所有函数：
//! - 处理跨平台差异（Windows/macOS/Linux）
//! - 安全校验（拒绝路径遍历、UNC 路径）
//! - 统一日志（[Shell] 前缀，调用前后都记录）
//! - 错误转换为 String（Tauri 命令可直接返回）

use crate::{log_debug, log_error, log_info};

// ============ 路径校验 ============

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

// ============ 文件管理器交互 ============

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

        type HWND = isize;
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

// ============ 进程管理 ============

/// 杀掉进程树（含子进程）
///
/// - Windows: `taskkill /PID <pid> /T /F`（/T 杀子进程，/F 强制结束）
/// - Unix: `kill -9 <pid>`
pub fn kill_process_tree(pid: u32) -> Result<(), String> {
    log_info!("[Shell] kill_process_tree: pid={}", pid);

    #[cfg(target_os = "windows")]
    let output = std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output()
        .map_err(|e| shell_err("kill_process_tree", e))?;

    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .map_err(|e| shell_err("kill_process_tree", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = format!("kill process {} failed: {}", pid, stderr.trim());
        log_error!("[Shell] {}", msg);
        return Err(msg);
    }
    Ok(())
}

// ============ 文件权限 ============

/// 尽力限制文件权限为当前用户（防止敏感信息被其他用户读取）
///
/// 仅尽力执行，失败只记日志不返回错误（调用方不关心失败）：
/// - Windows: `icacls <path> /inheritance:r /grant:r "<user>:F"`
///   移除继承权限并仅保留当前用户完全控制
/// - Unix: `chmod 600`（仅当前用户可读写）
pub fn restrict_file_permissions(path: &std::path::Path) {
    log_info!("[Shell] restrict_file_permissions: {}", path.display());

    #[cfg(target_os = "windows")]
    {
        let username = std::env::var("USERNAME").unwrap_or_default();
        if username.is_empty() {
            log_error!("[Shell] icacls skipped: USERNAME env empty");
            return;
        }
        let grant = format!("{}:F", username);
        match std::process::Command::new("icacls")
            .arg(path)
            .arg("/inheritance:r")
            .arg("/grant:r")
            .arg(&grant)
            .output()
        {
            Ok(out) if !out.status.success() => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                log_error!("[Shell] icacls failed: {}", stderr.trim());
            }
            Err(e) => log_error!("[Shell] icacls failed: {}", e),
            _ => {}
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
            log_error!("[Shell] chmod 600 failed: {}", e);
        }
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        let _ = path;
    }
}

// ============ 通用可执行文件执行 ============

/// 执行指定可执行文件并返回完整输出（同步阻塞）
///
/// 统一封装 `std::process::Command` 调用，提供：
/// - `[Shell]` 前缀日志（记录 program + args + cwd）
/// - Windows 下 CREATE_NO_WINDOW 标志（避免弹出控制台窗口）
/// - 统一错误格式（含 [Shell] 前缀 + 日志）
///
/// 适用于：Java 二进制探测（`java -version`）、PreLaunch 命令执行（`cmd /C` / `sh -c`）
/// 等需要直接调用外部可执行文件的场景。
///
/// **不适用**：异步子进程执行（请用 `tokio::process::Command`）、需要权限校验的
/// 沙箱执行（请参考 `commands::plugins::spawn`）。
pub fn run_executable_output(
    program: &str,
    args: &[String],
    cwd: Option<&std::path::Path>,
) -> Result<std::process::Output, String> {
    log_debug!(
        "[Shell] run_executable: {} {} (cwd={})",
        program,
        args.join(" "),
        cwd.map(|p| p.display().to_string())
            .unwrap_or_else(|| "<inherit>".to_string())
    );

    let mut cmd = std::process::Command::new(program);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd.output().map_err(|e| shell_err(program, e))
}

// ============ 管理员权限 ============

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
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取当前 exe 路径失败: {}", e))?;
    log_info!("[Shell] relaunch_as_admin: {} {:?}", exe_path.display(), args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;

        type HWND = isize;
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

// ============ 辅助 ============

/// 统一格式化 shell 命令错误（含 [Shell] 前缀 + 日志）
fn shell_err(op: &str, e: std::io::Error) -> String {
    let msg = format!("{} failed: {}", op, e);
    log_error!("[Shell] {}", msg);
    msg
}

// ============ 窗口管理（macOS / Linux） ============
//
// 以下函数封装 window_title 模块用到的外部命令，避免业务代码直接
// 调用 std::process::Command。所有命令均带 [Shell] 前缀日志。

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
    log_info!(
        "[Shell] wmctrl -r '{}' -T '{}'",
        old_title,
        new_title
    );
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
