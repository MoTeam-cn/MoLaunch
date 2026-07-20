//! 游戏进程监控器
//!
//! 监控游戏状态和崩溃检测。
//!
//! 按关注点拆分为 3 个子模块：
//! - `types`       GameState / ExitInfo / CrashInfo / CrashCategory / LogLevel / LogEntry / LoadProgress
//! - `log_parser`  parse_log_line / extract_log_level / detect_load_progress 纯函数
//! - `analyzer`    analyze_crash / analyze_stack_for_mod / analyze_crash_report / parse_crash_report
//! - `mod.rs`      GameWatcher 结构体 + start_monitoring / stop 核心流程

mod analyzer;
mod log_parser;
mod types;
mod window_title;

pub use types::{
    CrashCategory, CrashInfo, ExitInfo, GameState, LoadProgress, LogEntry, LogLevel,
};

use crate::log_info;
use log_parser::{detect_load_progress, parse_log_line};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, RwLock};

/// 游戏进程监控器
pub struct GameWatcher {
    /// 进程ID
    #[allow(dead_code)]
    pid: u32,
    /// 游戏状态
    state: Arc<RwLock<GameState>>,
    /// 加载进度
    load_progress: Arc<RwLock<LoadProgress>>,
    /// 日志缓冲区
    log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    /// 最大日志行数
    max_log_lines: usize,
    /// 游戏目录
    #[allow(dead_code)]
    game_dir: PathBuf,
    /// 版本ID
    version_id: String,
    /// 退出通知通道
    exit_tx: tokio::sync::watch::Sender<Option<ExitInfo>>,
    /// 退出接收通道（供外部监听）
    exit_rx: tokio::sync::watch::Receiver<Option<ExitInfo>>,
    /// 自定义窗口标题（非空时启动后改写游戏窗口标题）
    window_title: Option<String>,
    /// 手动停止标志（stop_game 调用时设为 true，watcher 检测到后跳过崩溃分析）
    /// 修复：之前 kill_process_tree 后游戏以非 0 退出码退出，watcher 误判为崩溃并触发分析
    manual_stop: Arc<std::sync::atomic::AtomicBool>,
}

impl GameWatcher {
    /// 创建新的监控器
    ///
    /// `window_title`：自定义窗口标题，非空时启动后通过 Win32 SetWindowText 改写游戏窗口标题
    pub fn new(pid: u32, game_dir: PathBuf, version_id: String, window_title: Option<String>) -> Self {
        let (exit_tx, exit_rx) = tokio::sync::watch::channel(None);
        Self {
            pid,
            state: Arc::new(RwLock::new(GameState::Starting)),
            load_progress: Arc::new(RwLock::new(LoadProgress::None)),
            log_buffer: Arc::new(Mutex::new(VecDeque::new())),
            exit_tx,
            exit_rx,
            max_log_lines: 10000,
            game_dir,
            version_id,
            window_title,
            manual_stop: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 标记为手动停止（stop_game 调用）
    /// watcher 检测到此标志后，跳过崩溃分析，直接按正常退出处理
    pub fn mark_manual_stop(&self) {
        self.manual_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取当前状态
    pub async fn state(&self) -> GameState {
        self.state.read().await.clone()
    }

    /// 获取加载进度
    pub async fn load_progress(&self) -> LoadProgress {
        *self.load_progress.read().await
    }

    /// 获取最近的日志
    pub async fn recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let buffer = self.log_buffer.lock().await;
        buffer.iter().rev().take(count).cloned().collect()
    }

    /// 获取退出通知接收器
    pub fn exit_receiver(&self) -> tokio::sync::watch::Receiver<Option<ExitInfo>> {
        self.exit_rx.clone()
    }

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

                            // 添加到缓冲区
                            let mut buffer = log_buffer.lock().await;
                            buffer.push_back(entry);
                            if buffer.len() > max_lines {
                                buffer.pop_front();
                            }
                        }
                        Err(e) => {
                            crate::log_warn!("[Watcher] stdout 读取异常: {}", e);
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
                            crate::log_warn!("[Watcher] stderr 读取异常: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // 启动窗口标题修改
        // 如果设置了自定义窗口标题，启动后轮询找到 MC 窗口，改写标题
        // 支持 {date} 和 {time} 实时替换
        // 跨平台：Windows 用 Win32 API，macOS 用 osascript，Linux 用 wmctrl/xdotool
        if let Some(ref title) = self.window_title {
            let title = title.clone();
            let pid = self.pid;
            tokio::spawn(async move {
                window_title::apply_window_title(pid, title).await;
            });
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
                analyzer::analyze_crash(exit_code, &logs, &game_dir).await
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
                log_info!("[Watcher] 崩溃分析完成: {}（类别: {:?}）", info.reason, info.category);
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

    /// 停止监控
    pub async fn stop(&self, child: &Arc<Mutex<Option<tokio::process::Child>>>) {
        let mut guard = child.lock().await;
        if let Some(ref mut c) = *guard {
            let _ = c.kill().await;
        }
    }
}
