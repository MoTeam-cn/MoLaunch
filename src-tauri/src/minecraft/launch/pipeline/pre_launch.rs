//! 启动前命令执行（PreLaunch）

use crate::{log_info, log_warn};

use super::{LaunchError, LaunchPipeline};

impl LaunchPipeline {
    /// 执行启动前命令（语法同 Windows cmd，不等待退出，失败仅记录日志）
    pub(super) async fn run_pre_launch(&self) -> Result<(), LaunchError> {
        let cmd_str = match self.config.pre_launch_cmd.as_ref() {
            Some(s) if !s.is_empty() => s.clone(),
            _ => return Ok(()),
        };

        // 安全检测：检查命令字符串中的危险字符/关键词（仅警告，不阻止执行）
        // 保留底层执行方式（cmd /C 或 sh -c）以维持向后兼容
        match validate_pre_launch_cmd(&cmd_str) {
            Err(reason) => log_warn!(
                "PreLaunch executing command: {} (warning: contains potentially dangerous characters: {})",
                cmd_str,
                reason
            ),
            Ok(()) => log_warn!("PreLaunch executing command: {}", cmd_str),
        }

        #[cfg(target_os = "windows")]
        let (program, args) = ("cmd", vec!["/C".to_string(), cmd_str.clone()]);
        #[cfg(not(target_os = "windows"))]
        let (program, args) = ("sh", vec!["-c".to_string(), cmd_str.clone()]);

        let game_dir = self.config.game_dir.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = std::process::Command::new(program);
            cmd.args(&args).current_dir(&game_dir);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            cmd.output()
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                if !output.status.success() {
                    log_info!(
                        "[PreLaunch] Command exited with status: {}",
                        output.status
                    );
                }
                Ok(())
            }
            Ok(Err(e)) => {
                log_info!("[PreLaunch] Failed to execute command: {}", e);
                // 启动前命令失败不中断启动流程（与 PCL2 行为一致）
                Ok(())
            }
            Err(e) => {
                log_info!("[PreLaunch] Task spawn failed: {}", e);
                Ok(())
            }
        }
    }
}

/// 检测 PreLaunch 命令字符串中的危险字符/关键词。
/// 返回 `Err(reason)` 表示检测到危险模式（reason 为具体原因），`Ok(())` 表示未检测到。
/// 注意：仅用于日志警告，不阻止命令执行（保持向后兼容，用户可能确实需要这些命令）。
fn validate_pre_launch_cmd(cmd: &str) -> Result<(), String> {
    // 命令分隔符：&、&&、|
    if cmd.contains('&') || cmd.contains('|') {
        return Err("command separator (& or |)".to_string());
    }
    // 重定向：>、<
    if cmd.contains('>') || cmd.contains('<') {
        return Err("redirection (> or <)".to_string());
    }
    // 命令替换：反引号、$(
    if cmd.contains('`') || cmd.contains("$(") {
        return Err("command substitution (` or $()".to_string());
    }
    // 常见攻击载荷关键词（不区分大小写）
    let lower = cmd.to_lowercase();
    for keyword in ["powershell", "curl", "wget", "iex", "invoke-"] {
        if lower.contains(keyword) {
            return Err(format!("suspicious keyword: {}", keyword));
        }
    }
    Ok(())
}
