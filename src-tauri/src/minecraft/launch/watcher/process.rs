//! 游戏进程状态机
//!
//! 定义 GameWatcher 结构、状态访问与停止控制；事件分发见 scheduler.rs。

use super::types::{ExitInfo, GameState, LoadProgress, LogEntry};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{watch, Mutex, RwLock};

/// 游戏进程监控器
pub struct GameWatcher {
    /// 进程ID
    #[allow(dead_code)]
    pub(super) pid: u32,
    /// 游戏状态
    pub(super) state: Arc<RwLock<GameState>>,
    /// 加载进度
    pub(super) load_progress: Arc<RwLock<LoadProgress>>,
    /// 日志缓冲区
    pub(super) log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    /// 最大日志行数
    pub(super) max_log_lines: usize,
    /// 游戏目录
    #[allow(dead_code)]
    pub(super) game_dir: PathBuf,
    /// 版本ID
    pub(super) version_id: String,
    /// 退出通知通道
    pub(super) exit_tx: watch::Sender<Option<ExitInfo>>,
    /// 退出接收通道（供外部监听）
    pub(super) exit_rx: watch::Receiver<Option<ExitInfo>>,
    /// 自定义窗口标题（非空时启动后改写游戏窗口标题）
    pub(super) window_title: Option<String>,
    /// 手动停止标志（stop_game 调用时设为 true，watcher 检测到后跳过崩溃分析）
    /// 修复：之前 kill_process_tree 后游戏以非 0 退出码退出，watcher 误判为崩溃并触发分析
    pub(super) manual_stop: Arc<std::sync::atomic::AtomicBool>,
    /// Tauri AppHandle（用于 emit 联机端口检测事件）
    /// None 时跳过 emit（防御性兜底，实际场景下 build_launch_config 总是注入 Some）
    pub(super) app_handle: Option<tauri::AppHandle>,
}

impl GameWatcher {
    /// 创建新的监控器
    ///
    /// `window_title`：自定义窗口标题，非空时启动后通过 Win32 SetWindowText 改写游戏窗口标题；
    /// 空值/空字符串不改写（跟随全局设置）
    /// `app_handle`：Tauri AppHandle，用于在 stdout 检测到 LAN 端口时 emit 事件给联机模块
    pub fn new(
        pid: u32,
        game_dir: PathBuf,
        version_id: String,
        window_title: Option<String>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Self {
        let (exit_tx, exit_rx) = watch::channel(None);
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
            app_handle,
        }
    }

    /// 标记为手动停止（stop_game 调用）
    /// watcher 检测到此标志后，跳过崩溃分析，直接按正常退出处理
    pub fn mark_manual_stop(&self) {
        self.manual_stop
            .store(true, std::sync::atomic::Ordering::Relaxed);
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
    pub fn exit_receiver(&self) -> watch::Receiver<Option<ExitInfo>> {
        self.exit_rx.clone()
    }

    /// 停止监控
    pub async fn stop(&self, child: &Arc<Mutex<Option<tokio::process::Child>>>) {
        let mut guard = child.lock().await;
        if let Some(ref mut c) = *guard {
            let _ = c.kill().await;
        }
    }
}
