//! 事件分发与日志监控调度
//!
//! 负责 stdout/stderr 读取、加载进度检测、联机端口事件与退出/崩溃分析调度。

use super::log_parser::{detect_load_progress, parse_log_line};
use super::process::GameWatcher;
use super::types::{ExitInfo, GameState};
use crate::{log_info, log_warn};
use regex::Regex;
use std::sync::{Arc, OnceLock};
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

/// 联机模块 MC 局域网端口检测事件名
///
/// GameWatcher 在 stdout 检测到 "Local game hosted on port XXXXX" 或
/// "Started LAN game on port XXXXX" 时，通过此事件通知前端联机模块。
/// payload 为 u16 端口号。
pub const ONLINE_MC_PORT_DETECTED_EVENT: &str = "online://mc-port-detected";

/// MC 开放局域网时 stdout 输出的端口正则
///
/// 匹配 MC 标准日志格式：
/// - "Local game hosted on port 49152"（单人开放 LAN）
/// - "Started LAN game on port 49152"（部分版本）
static LAN_PORT_RE: OnceLock<Regex> = OnceLock::new();

fn lan_port_regex() -> &'static Regex {
    LAN_PORT_RE.get_or_init(|| {
        Regex::new(r"(?:Local game hosted|Started LAN game) on port (\d{1,5})")
            .expect("LAN port regex 编译失败")
    })
}

impl GameWatcher {
    /// 开始监控
    pub async fn start_monitoring(
        &self,
        child: tokio::process::Child,
    ) -> Arc<Mutex<Option<tokio::process::Child>>> {
        let child_handle = Arc::new(Mutex::new(Some(child)));
        let child_clone = child_handle.clone();

        // 获取stdout和stderr
        let (stdout, stderr) = {
            let mut guard = child_clone.lock().await;
            if let Some(ref mut c) = *guard {
                (c.stdout.take(), c.stderr.take())
            } else {
                (None, None)
            }
        };

        // 启动日志读取
        if let Some(stdout) = stdout {
            let log_buffer = self.log_buffer.clone();
            let state = self.state.clone();
            let load_progress = self.load_progress.clone();
            let max_lines = self.max_log_lines;
            let app_handle = self.app_handle.clone();

            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout);
                let mut buf: Vec<u8> = Vec::with_capacity(1024);

                loop {
                    buf.clear();
                    match reader.read_until(b'\n', &mut buf).await {
                        Ok(0) => break, // 流正常关闭
                        Ok(_) => {
                            // 去掉末尾换行符
                            if buf.last() == Some(&b'\n') {
                                buf.pop();
                                if buf.last() == Some(&b'\r') {
                                    buf.pop();
                                }
                            }
                            // Java 在 Windows 上默认按 GBK 输出，可能不是合法 UTF-8
                            // 用 lossy 转换避免读取中断
                            let line = String::from_utf8_lossy(&buf).to_string();
                            let entry = parse_log_line(&line, "stdout");

                            // 检测加载进度
                            let new_progress = detect_load_progress(&line);
                            {
                                let mut current = load_progress.write().await;
                                if new_progress > *current {
                                    *current = new_progress;
                                }
                            }

                            // 检测是否开始加载
                            {
                                let mut state_guard = state.write().await;
                                if *state_guard == GameState::Starting {
                                    *state_guard = GameState::Loading;
                                }
                            }

                            // 联机模块：检测 MC 开放局域网端口，命中后 emit 事件给前端
                            if let Some(caps) = lan_port_regex().captures(&line) {
                                if let Some(port_str) = caps.get(1) {
                                    if let Ok(port) = port_str.as_str().parse::<u16>() {
                                        log_info!(
                                            "[Watcher] 检测到 MC 局域网端口: {}（联机模块自动捕获）",
                                            port
                                        );
                                        if let Some(ref handle) = app_handle {
                                            let _ =
                                                handle.emit(ONLINE_MC_PORT_DETECTED_EVENT, port);
                                        }
                                    }
                                }
                            }

                            // 添加到缓冲区
                            let mut buffer = log_buffer.lock().await;
                            buffer.push_back(entry);
                            if buffer.len() > max_lines {
                                buffer.pop_front();
                            }
                        }
                        Err(e) => {
                            log_warn!("[Watcher] stdout 读取异常: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // 读取stderr
        if let Some(stderr) = stderr {
            let log_buffer = self.log_buffer.clone();
            let max_lines = self.max_log_lines;

            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut buf: Vec<u8> = Vec::with_capacity(1024);

                loop {
                    buf.clear();
                    match reader.read_until(b'\n', &mut buf).await {
                        Ok(0) => break, // 流正常关闭
                        Ok(_) => {
                            if buf.last() == Some(&b'\n') {
                                buf.pop();
                                if buf.last() == Some(&b'\r') {
                                    buf.pop();
                                }
                            }
                            let line = String::from_utf8_lossy(&buf).to_string();
                            let entry = parse_log_line(&line, "stderr");
                            let mut buffer = log_buffer.lock().await;
                            buffer.push_back(entry);
                            if buffer.len() > max_lines {
                                buffer.pop_front();
                            }
                        }
                        Err(e) => {
                            log_warn!("[Watcher] stderr 读取异常: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // 启动窗口标题修改：轮询找到 MC 窗口并改写标题，支持 {date}/{time} 实时替换
        // 空标题（用户留空"跟随全局设置"）不触发改写，避免把游戏窗口标题改成空白
        if let Some(ref title) = self.window_title {
            if !title.trim().is_empty() {
                let title = title.clone();
                let pid = self.pid;
                tokio::spawn(async move {
                    super::window_title::apply_window_title(pid, title).await;
                });
            }
        }

        // 启动状态检测
        let state = self.state.clone();
        let _load_progress = self.load_progress.clone();
        let log_buffer = self.log_buffer.clone();
        let _pid = self.pid;
        let exit_tx = self.exit_tx.clone();
        let version_id = self.version_id.clone();
        let game_dir = self.game_dir.clone();
        let manual_stop = self.manual_stop.clone();

        tokio::spawn(async move {
            // 等待进程结束
            let exit_code = {
                let mut guard = child_clone.lock().await;
                if let Some(ref mut c) = *guard {
                    c.wait().await.ok().map(|s| s.code().unwrap_or(-1))
                } else {
                    None
                }
            };

            let exit_code = exit_code.unwrap_or(-1);
            let logs = {
                let buffer = log_buffer.lock().await;
                buffer.iter().cloned().collect::<Vec<_>>()
            };

            // 分析是否崩溃
            // 延迟 2 秒让文件系统刷新崩溃报告
            // 修复：手动停止（stop_game）时跳过崩溃分析，直接按正常退出处理
            let is_manual_stop = manual_stop.load(std::sync::atomic::Ordering::Relaxed);
            let crash_info = if exit_code != 0 && !is_manual_stop {
                log_info!(
                    "[Watcher] 游戏异常退出（code={}），2 秒后开始崩溃分析...",
                    exit_code
                );
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                super::analyzer::analyze_crash(exit_code, &logs, &game_dir).await
            } else if is_manual_stop {
                log_info!(
                    "[Watcher] 游戏被手动停止（code={}），跳过崩溃分析",
                    exit_code
                );
                None
            } else {
                None
            };

            let exit_info = if let Some(info) = crash_info {
                log_info!(
                    "[Watcher] 崩溃分析完成: {}（类别: {:?}）",
                    info.reason,
                    info.category
                );
                let mut state_guard = state.write().await;
                *state_guard = GameState::Crashed(info.clone());
                ExitInfo {
                    code: exit_code,
                    is_normal: false,
                    crash_info: Some(info),
                }
            } else {
                let exit_info = ExitInfo {
                    code: exit_code,
                    // 手动停止或退出码为 0 都算正常退出
                    is_normal: exit_code == 0 || is_manual_stop,
                    crash_info: None,
                };
                let mut state_guard = state.write().await;
                *state_guard = GameState::Exited(exit_info.clone());
                exit_info
            };

            // 发送退出通知
            let _ = exit_tx.send(Some(exit_info));

            log_info!(
                "[Watcher] Game process exited (PID: {}, code: {}, version: {})",
                _pid,
                exit_code,
                version_id
            );
        });

        child_handle
    }
}