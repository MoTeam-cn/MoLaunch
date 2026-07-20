//! 游戏进程启动与早期崩溃检测

use std::path::PathBuf;

use crate::log_info;

use super::super::watcher::{ExitInfo, GameWatcher, LogEntry};
use super::{LaunchError, LaunchPipeline, LaunchResult, LaunchStage};

impl LaunchPipeline {
    /// 启动游戏进程
    pub(super) async fn launch_process(
        &self,
        java_path: &PathBuf,
        args: &super::super::LaunchArguments,
    ) -> Result<LaunchResult, LaunchError> {
        use tokio::process::Command;

        let mut cmd = Command::new(java_path);

        // 添加JVM参数
        for arg in &args.jvm_args {
            cmd.arg(arg);
        }

        // 添加主类
        cmd.arg(&args.main_class);

        // 添加游戏参数
        for arg in &args.game_args {
            cmd.arg(arg);
        }

        // 设置工作目录（使用 effective_game_dir，即隔离目录）
        // args.game_dir 是 build_launch_arguments 内部通过 isolation::get_effective_game_dir 计算的有效目录
        cmd.current_dir(&args.game_dir);

        // 设置环境变量（APPDATA 也指向隔离目录，某些 Mod 会读取）
        cmd.env("appdata", &args.game_dir);

        // 重定向stdout和stderr以便监控
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Windows: 不显示控制台窗口
        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        // 打印启动命令（脱敏处理，避免 access_token 等敏感信息写入日志文件）
        // 修复：之前 {:?} 打印完整 game_args，其中含 --accessToken <真实token>，会被持久化到日志文件
        log_info!(
            "Launching: {} {} {} {}",
            java_path.display(),
            super::super::sanitize_args_for_log(&args.jvm_args).join(" "),
            args.main_class,
            super::super::sanitize_args_for_log(&args.game_args).join(" ")
        );

        let child = cmd.spawn().map_err(|e| LaunchError {
            stage: LaunchStage::LaunchProcess,
            message: format!("启动进程失败: {}", e),
            is_user_facing: true,
        })?;

        let pid = child.id().unwrap_or(0);
        log_info!("Game process started with PID: {}", pid);

        // 创建监控器（game_dir 使用隔离目录，确保崩溃分析在正确目录查找日志）
        // 传入 window_title：非空时启动后通过 Win32 SetWindowText 改写游戏窗口标题
        let watcher = GameWatcher::new(
            pid,
            std::path::PathBuf::from(&args.game_dir),
            self.config.version_id.clone(),
            self.config.window_title.clone(),
        );

        // 启动监控
        let child_handle = watcher.start_monitoring(child).await;

        // 保存监控器和子进程引用
        *self.watcher.lock().await = Some(watcher);
        *self.child_process.lock().await = Some(child_handle.clone());

        // 等待一段时间检查进程是否立即崩溃
        // Forge 启动较慢，等待 5 秒覆盖早期崩溃
        let exit_rx = {
            let watcher_guard = self.watcher.lock().await;
            if let Some(ref w) = *watcher_guard {
                w.exit_receiver()
            } else {
                return Err(LaunchError {
                    stage: LaunchStage::LaunchProcess,
                    message: "Watcher not available".to_string(),
                    is_user_facing: false,
                });
            }
        };

        // 早期崩溃检测：轮询最多 2 秒，每 200ms 检查一次
        let fatal_errors = [
            "A Java Exception has occurred",
            "Error: A JNI error has occurred",
            "Could not create the Java Virtual Machine",
            "Exception in thread",
            "java.lang.NoClassDefFoundError",
            "java.lang.ClassNotFoundException",
            "java.lang.UnsupportedClassVersionError",
        ];
        // 正常启动标志：出现这些说明游戏已开始正常加载，不再需要等待
        let healthy_signs = [
            "LWJGL",
            "Setting user",
            "GL info",
            "OpenAL",
            "lwjgl",
            "ModLauncher",
            "EARLYDISPLAY",
            "Launching target",
        ];

        let mut exit_info_caught: Option<ExitInfo> = None;
        let mut error_logs: Option<Vec<String>> = None;

        let poll_deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);
        loop {
            if tokio::time::Instant::now() >= poll_deadline {
                break;
            }

            // 先检查进程是否退出（非阻塞：借用 watch 的已接收值）
            {
                let mut rx = exit_rx.clone();
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    rx.changed(),
                )
                .await
                {
                    Ok(Ok(())) => {
                        if let Some(ref info) = *rx.borrow() {
                            exit_info_caught = Some(info.clone());
                            break; // 进程已退出，跳出处理
                        }
                    }
                    _ => {}
                }
            }

            // 检查日志
            let logs = {
                let watcher_guard = self.watcher.lock().await;
                if let Some(ref w) = *watcher_guard {
                    w.recent_logs(80).await
                } else {
                    Vec::new()
                }
            };

            let logs_chronological: Vec<&LogEntry> = logs.iter().rev().collect();

            // 先检查是否有 Java 异常
            for (idx, log) in logs_chronological.iter().enumerate() {
                for error in &fatal_errors {
                    if log.message.contains(error) {
                        let tail: Vec<String> = logs_chronological
                            .iter()
                            .skip(idx)
                            .take(30)
                            .map(|l| l.message.clone())
                            .collect();
                        error_logs = Some(tail);
                        break;
                    }
                }
                if error_logs.is_some() {
                    break;
                }
            }
            if let Some(tail) = error_logs.take() {
                return Err(LaunchError {
                    stage: LaunchStage::LaunchProcess,
                    message: format!("Java启动失败:\n{}", tail.join("\n")),
                    is_user_facing: true,
                });
            }

            // 检查是否有正常启动标志 → 立即返回
            let has_healthy = logs_chronological
                .iter()
                .any(|l| healthy_signs.iter().any(|s| l.message.contains(s)));
            if has_healthy {
                break;
            }
        }

        // 处理轮询期间捕获的进程退出
        if let Some(exit_info) = exit_info_caught {
            if exit_info.code != 0 {
                let logs = {
                    let watcher_guard = self.watcher.lock().await;
                    if let Some(ref w) = *watcher_guard {
                        w.recent_logs(40).await
                    } else {
                        Vec::new()
                    }
                };
                let tail: Vec<String> = logs.iter().take(40).map(|l| l.message.clone()).collect();
                return Err(LaunchError {
                    stage: LaunchStage::LaunchProcess,
                    message: format!(
                        "游戏进程退出（代码: {}）\n最近日志:\n{}",
                        exit_info.code,
                        tail.join("\n")
                    ),
                    is_user_facing: true,
                });
            }
        }

        Ok(LaunchResult {
            pid,
            java_path: java_path.clone(),
            game_dir: self.config.game_dir.clone(),
            args: args
                .jvm_args
                .iter()
                .chain(std::iter::once(&args.main_class))
                .chain(args.game_args.iter())
                .cloned()
                .collect(),
        })
    }
}
