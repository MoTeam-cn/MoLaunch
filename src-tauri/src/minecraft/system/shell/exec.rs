//! 通用可执行文件执行与进程管理
//!
//! `run_executable_output` 同步执行外部可执行文件并返回输出；
//! `kill_process_tree` 杀掉进程树（含子进程）。

use crate::{log_debug, log_error, log_info};

use super::shell_err;

/// 执行指定可执行文件并返回完整输出（同步阻塞）
///
/// 统一封装 `std::process::Command`：`[Shell]` 前缀日志、Windows CREATE_NO_WINDOW、统一错误格式。
/// 适用于 Java 探测（`java -version`）、PreLaunch（`cmd /C`/`sh -c`）等直接调外部可执行文件。
/// 不适用：异步子进程（用 `tokio::process::Command`）、权限校验沙箱（见 `commands::plugins::spawn`）。
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
