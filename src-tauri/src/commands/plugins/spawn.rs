//! 插件子进程执行（带权限校验 + 命令白名单 + 超时控制）
//!
//! - `plugin_spawn_process`：执行子进程命令
//!
//! 安全限制：
//! - command 必须在 manifest.processPermissions.allowedCommands 白名单内
//! - 命令名会被 canonicalize 后与白名单匹配，支持绝对路径或 PATH 查找结果
//! - 非 shell 执行（`tokio::process::Command`），防止注入
//! - 超时默认 30 秒，最大 5 分钟（`tokio::time::timeout` 包裹 `child.wait()`，超时 `child.kill()`）
//! - stdout/stderr 管道异步读取各截断到 1MB（在 UTF-8 字符边界切割）
//!
//! 共享类型在 `super::` 中（`ProcessPermissions` / `read_plugin_manifest`）。
//!
//! 注：原 Tauri 命令已聚合为 `plugins_manager` 一个 IPC 入口，子模块函数已去掉
//! `#[tauri::command]` 标注，由 `utils::plugins_manager::dispatch` 调用。

use super::read_plugin_manifest;
use crate::error_util::log_err;
use crate::log_info;
use serde::Serialize;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// stdout/stderr 最大字节数（1MB）
pub(crate) const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
/// 超时上限（5 分钟）
pub(crate) const MAX_TIMEOUT_MS: u64 = 300_000;

/// 子进程执行结果
#[derive(Debug, Serialize)]
pub struct ProcessResult {
    /// 退出码（None 表示超时或被信号终止）
    pub exit_code: Option<i32>,
    /// 标准输出（截断到 1MB）
    pub stdout: String,
    /// 标准误差（截断到 1MB）
    pub stderr: String,
    /// 是否超时
    pub timed_out: bool,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
}

/// 执行插件子进程命令
///
/// 流程：权限校验 → 命令白名单匹配 → 非 shell 执行 → 超时控制 → 管道异步读取 + 截断
pub async fn plugin_spawn_process(
    plugin_id: String,
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
) -> Result<ProcessResult, String> {
    // 1. 读取 manifest
    let manifest = read_plugin_manifest(&plugin_id)?;

    // 2. 校验 spawnProcess 权限
    if !manifest.permissions.iter().any(|p| p == "spawnProcess") {
        return Err(format!(
            "Plugin {} does not have spawnProcess permission",
            plugin_id
        ));
    }

    // 3. 校验 process_permissions 配置存在
    let proc_perms = manifest
        .process_permissions
        .as_ref()
        .ok_or_else(|| format!("Plugin {} missing processPermissions config", plugin_id))?;

    // 4. 校验命令在白名单内
    if !is_command_allowed(&command, &proc_perms.allowed_commands)? {
        return Err(format!("Command not allowed: {}", command));
    }

    // 5. 校验超时上限
    let timeout_ms = proc_perms.timeout_ms.min(MAX_TIMEOUT_MS);
    let timeout = Duration::from_millis(timeout_ms);

    // 6. 构建 Command（非 shell 执行，防注入）
    let mut cmd = Command::new(&command);
    cmd.args(&args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    // 阻止继承 stdin / 父环境
    cmd.stdin(std::process::Stdio::null());

    let start = Instant::now();

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {}", command, e))?;

    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();

    // 7. 并行读取 stdout/stderr（独立 task）
    let stdout_task = tokio::spawn(async move {
        if let Some(mut stdout) = stdout_handle {
            let mut buf = Vec::new();
            stdout.read_to_end(&mut buf).await.ok();
            buf
        } else {
            Vec::new()
        }
    });

    let stderr_task = tokio::spawn(async move {
        if let Some(mut stderr) = stderr_handle {
            let mut buf = Vec::new();
            stderr.read_to_end(&mut buf).await.ok();
            buf
        } else {
            Vec::new()
        }
    });

    // 8. 超时包裹 child.wait()
    let (exit_code, timed_out) = match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(status)) => (status.code(), false),
        Ok(Err(e)) => {
            return Err(format!("Wait failed: {}", e));
        }
        Err(_) => {
            // 超时：kill 进程
            let _ = child.kill().await;
            (None, true)
        }
    };

    let stdout_bytes = stdout_task.await.unwrap_or_default();
    let stderr_bytes = stderr_task.await.unwrap_or_default();

    let stdout = truncate_output(&stdout_bytes);
    let stderr = truncate_output(&stderr_bytes);

    let duration_ms = start.elapsed().as_millis() as u64;

    log_info!(
        "插件 {} 执行命令 '{}' 退出码={:?} 耗时={}ms 超时={}",
        plugin_id,
        command,
        exit_code,
        duration_ms,
        timed_out
    );

    Ok(ProcessResult {
        exit_code,
        stdout,
        stderr,
        timed_out,
        duration_ms,
    })
}

/// 校验命令是否在白名单内
///
/// canonicalize 后比对，Windows 忽略大小写与 `.exe` 后缀。
pub(crate) fn is_command_allowed(command: &str, allowed: &[String]) -> Result<bool, String> {
    // 先尝试 canonicalize 输入命令
    let canonical = which_canonical(command).ok();

    for allowed_cmd in allowed {
        // 直接字符串匹配
        if allowed_cmd == command {
            return Ok(true);
        }

        // canonicalize 后匹配
        if let Some(ref canonical) = canonical {
            if let Ok(allowed_canonical) = which_canonical(allowed_cmd) {
                if paths_equal(canonical, &allowed_canonical) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// 简单的 which 实现：通过 PATH 查找命令的完整路径并 canonicalize
pub(crate) fn which_canonical(command: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(command);
    if path.is_absolute() {
        return path.canonicalize().map_err(log_err("Failed to canonicalize command path"));
    }

    let path_env = std::env::var("PATH").map_err(|_| "PATH not set".to_string())?;

    let separator = if cfg!(windows) { ';' } else { ':' };

    for dir in path_env.split(separator) {
        if dir.is_empty() {
            continue;
        }
        let dir_path = PathBuf::from(dir);

        #[cfg(windows)]
        {
            for ext in &[".exe", ".bat", ".cmd"] {
                let candidate = dir_path.join(format!("{}{}", command, ext));
                if candidate.exists() {
                    return candidate.canonicalize().map_err(log_err("Failed to canonicalize command path"));
                }
            }
        }

        let candidate = dir_path.join(command);
        if candidate.exists() {
            return candidate.canonicalize().map_err(log_err("Failed to canonicalize command path"));
        }
    }

    Err(format!("Command not found in PATH: {}", command))
}

/// 路径相等比较（Windows 忽略大小写与 `.exe` 后缀）
pub(crate) fn paths_equal(a: &std::path::Path, b: &std::path::Path) -> bool {
    #[cfg(windows)]
    {
        let a_str = a.to_string_lossy().to_lowercase();
        let b_str = b.to_string_lossy().to_lowercase();
        let a_clean = a_str.trim_end_matches(".exe");
        let b_clean = b_str.trim_end_matches(".exe");
        a_clean == b_clean
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

/// 截断输出到 MAX_OUTPUT_BYTES（在 UTF-8 字符边界切割）
///
/// 超过上限时从上限位置向前回退到 UTF-8 字符边界（非 continuation byte），
/// 避免 String::from_utf8_lossy 出现替换字符。
pub(crate) fn truncate_output(bytes: &[u8]) -> String {
    if bytes.len() <= MAX_OUTPUT_BYTES {
        return String::from_utf8_lossy(bytes).to_string();
    }

    // 找到 UTF-8 字符边界
    let mut end = MAX_OUTPUT_BYTES;
    while end > 0 && (bytes[end] & 0xC0) == 0x80 {
        end -= 1;
    }

    String::from_utf8_lossy(&bytes[..end]).to_string()
}
