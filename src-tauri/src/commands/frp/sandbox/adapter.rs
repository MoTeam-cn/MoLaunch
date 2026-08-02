//! 认证适配器脚本沙箱（§7.5）：白名单命令校验 + 超时 + 环境清理 + 输出截断

use super::super::provider::{read_provider_manifest, SYSTEM_DEFAULT_ID};
use super::super::{validate_provider_id, ProcessPermissions};
use crate::commands::frp::providers_root;
use crate::commands::plugins::spawn::{
    is_command_allowed, truncate_output, ProcessResult, MAX_TIMEOUT_MS,
};
use crate::log_debug;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// 沙箱化执行厂商认证适配器脚本（§7.5）
///
/// 流程：读取厂商 `process_permissions` → 校验命令在白名单（`spawn::is_command_allowed`）
/// → 工作目录锁定厂商目录 → 超时控制（默认 30s，最大 5min）→ 非 shell 执行防注入
/// → `env_clear()` 仅保留 PATH → stdout/stderr 各截断到 1MB。
/// 系统默认厂商不提供自定义脚本，直接拒绝。
pub async fn run_auth_adapter(
    provider_id: &str,
    command: String,
    args: Vec<String>,
) -> Result<ProcessResult, String> {
    // 1. 系统默认厂商不提供自定义脚本
    if provider_id == SYSTEM_DEFAULT_ID {
        return Err("系统默认厂商不支持自定义认证适配器脚本".to_string());
    }

    // 2. 校验厂商 ID 合法性
    validate_provider_id(provider_id)?;

    // 3. 读取厂商 manifest 的 process_permissions
    let manifest = read_provider_manifest(provider_id)?;
    let proc_perms: &ProcessPermissions =
        manifest.process_permissions.as_ref().ok_or_else(|| {
            format!(
                "厂商 {} 未配置 processPermissions，禁止执行认证适配器脚本",
                provider_id
            )
        })?;

    if proc_perms.allowed_commands.is_empty() {
        return Err(format!(
            "厂商 {} 的 allowedCommands 白名单为空",
            provider_id
        ));
    }

    // 4. 校验命令在白名单内（复用 spawn::is_command_allowed）
    if !is_command_allowed(&command, &proc_perms.allowed_commands)? {
        return Err(format!(
            "命令不在厂商 {} 的白名单内: {}",
            provider_id, command
        ));
    }

    // 5. 校验超时上限（默认 30s，最大 5min）
    let timeout_ms = proc_perms.timeout_ms.min(MAX_TIMEOUT_MS);
    let timeout = Duration::from_millis(timeout_ms);

    // 6. 工作目录强制设为厂商目录
    let provider_dir = providers_root().join(provider_id);
    if !provider_dir.exists() {
        return Err(format!("厂商目录不存在: {}", provider_dir.display()));
    }

    log_debug!(
        "[Frp Sandbox] 执行认证适配器: provider={}, command={}, args={:?}, timeout={}ms",
        provider_id,
        command,
        args,
        timeout_ms
    );

    // 7. 构建 Command（非 shell 执行，防注入）
    let mut cmd = Command::new(&command);
    cmd.args(&args);
    cmd.current_dir(&provider_dir);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::null());

    // 8. 清空环境变量，仅保留 PATH（防止敏感环境变量泄露）
    cmd.env_clear();
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }

    let start = Instant::now();

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动认证适配器失败 '{}': {}", command, e))?;

    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();

    // 9. 并行读取 stdout/stderr
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

    // 10. 超时包裹 child.wait()
    let (exit_code, timed_out) = match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(status)) => (status.code(), false),
        Ok(Err(e)) => {
            return Err(format!("等待认证适配器退出失败: {}", e));
        }
        Err(_) => {
            // 超时：kill 进程
            let _ = child.kill().await;
            (None, true)
        }
    };

    let stdout_bytes = stdout_task.await.unwrap_or_default();
    let stderr_bytes = stderr_task.await.unwrap_or_default();

    // 11. 截断输出到 1MB（复用 spawn::truncate_output）
    let stdout = truncate_output(&stdout_bytes);
    let stderr = truncate_output(&stderr_bytes);

    let duration_ms = start.elapsed().as_millis() as u64;

    log_debug!(
        "[Frp Sandbox] 认证适配器完成: provider={}, exit_code={:?}, 耗时={}ms, 超时={}",
        provider_id,
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
