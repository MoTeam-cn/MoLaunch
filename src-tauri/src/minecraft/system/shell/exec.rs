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
/// - Unix: 递归收集所有后代 PID 后批量 `kill -9`（原实现仅杀单进程，与函数名"树"语义不符）
pub fn kill_process_tree(pid: u32) -> Result<(), String> {
    log_info!("[Shell] kill_process_tree: pid={}", pid);

    #[cfg(target_os = "windows")]
    let output = std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output()
        .map_err(|e| shell_err("kill_process_tree", e))?;

    #[cfg(not(target_os = "windows"))]
    let output = {
        // 用 ps 一次性获取所有进程的 pid 与 ppid，构建 parent -> children 映射
        // `ps -A -o pid= -o ppid=` 是 POSIX 标准写法（`=` 去表头），Linux/macOS 通用
        let ps_output = std::process::Command::new("ps")
            .args(["-A", "-o", "pid=", "-o", "ppid="])
            .output()
            .map_err(|e| shell_err("kill_process_tree", e))?;

        if !ps_output.status.success() {
            let stderr = String::from_utf8_lossy(&ps_output.stderr);
            let msg = format!(
                "kill process tree {} failed: ps command failed: {}",
                pid,
                stderr.trim()
            );
            log_error!("[Shell] {}", msg);
            return Err(msg);
        }

        // 解析 ps 输出，构建 parent_pid -> [child_pid, ...] 映射
        let stdout = String::from_utf8_lossy(&ps_output.stdout);
        let mut parent_map: std::collections::HashMap<u32, Vec<u32>> =
            std::collections::HashMap::new();
        for line in stdout.lines() {
            let mut iter = line.split_whitespace();
            let (Some(p_str), Some(pp_str)) = (iter.next(), iter.next()) else {
                continue;
            };
            let (Ok(p), Ok(pp)) = (p_str.parse::<u32>(), pp_str.parse::<u32>()) else {
                continue;
            };
            parent_map.entry(pp).or_default().push(p);
        }

        // 递归收集所有后代 PID（含 pid 自身）
        let mut all_pids = vec![pid];
        let mut stack = vec![pid];
        while let Some(current) = stack.pop() {
            if let Some(children) = parent_map.get(&current) {
                for child in children {
                    all_pids.push(*child);
                    stack.push(*child);
                }
            }
        }

        log_debug!(
            "[Shell] kill_process_tree: pid={} 收集到 {} 个进程（含自身）",
            pid,
            all_pids.len()
        );

        // 批量 kill -9：先杀子进程，最后杀父进程，避免子进程被 reparent 到 init
        let mut errs = Vec::new();
        for p in &all_pids {
            let out = std::process::Command::new("kill")
                .args(["-9", &p.to_string()])
                .output();
            if let Ok(o) = out {
                if !o.status.success() && !o.stderr.is_empty() {
                    let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
                    if !err.is_empty() {
                        errs.push(format!("pid {}: {}", p, err));
                    }
                }
            }
        }

        if errs.is_empty() {
            return Ok(());
        }
        let msg = format!(
            "kill process tree {} partially failed: {}",
            pid,
            errs.join("; ")
        );
        log_error!("[Shell] {}", msg);
        return Err(msg);
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = format!("kill process {} failed: {}", pid, stderr.trim());
        log_error!("[Shell] {}", msg);
        return Err(msg);
    }
    Ok(())
}
