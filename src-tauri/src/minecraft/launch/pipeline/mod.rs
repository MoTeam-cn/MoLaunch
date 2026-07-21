//! Launch pipeline - 完整的Minecraft启动流程
//! 支持并行执行和进度追踪
//!
//! 结构：
//! - pipeline/mod.rs: 结构体定义 + 公共 API + 模块编排
//! - pipeline/types.rs: 数据类型（阶段/进度/配置/结果/错误）
//! - pipeline/execute.rs: execute 编排与进度更新
//! - pipeline/validate.rs: 文件校验补全与启动参数构建
//! - pipeline/java_check.rs: Java 检测与校验
//! - pipeline/natives.rs: Natives 原生库解压
//! - pipeline/pre_launch.rs: 启动前命令执行
//! - pipeline/process_spawn.rs: 游戏进程启动与早期崩溃检测

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use super::watcher::{GameState, GameWatcher, LoadProgress, LogEntry};

mod execute;
mod java_check;
mod natives;
mod pre_launch;
mod process_spawn;
mod types;
mod validate;

pub use self::types::{LaunchConfig, LaunchError, LaunchProgress, LaunchResult, LaunchStage};

/// 启动流水线
pub struct LaunchPipeline {
    pub(super) config: LaunchConfig,
    pub(super) progress: Arc<RwLock<LaunchProgress>>,
    #[allow(dead_code)]
    pub(super) current_stage: Arc<Mutex<LaunchStage>>,
    pub(super) cancel_flag: Arc<Mutex<bool>>,
    pub(super) watcher: Arc<Mutex<Option<GameWatcher>>>,
    pub(super) child_process:
        Arc<Mutex<Option<Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>>>>,
}

impl LaunchPipeline {
    /// 创建新的启动流水线
    pub fn new(config: LaunchConfig) -> Self {
        Self {
            config,
            progress: Arc::new(RwLock::new(LaunchProgress {
                stage: LaunchStage::Init,
                stage_progress: 0.0,
                overall_progress: 0.0,
                message: "初始化中...".to_string(),
            })),
            current_stage: Arc::new(Mutex::new(LaunchStage::Init)),
            cancel_flag: Arc::new(Mutex::new(false)),
            watcher: Arc::new(Mutex::new(None)),
            child_process: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取游戏状态
    pub async fn game_state(&self) -> Option<GameState> {
        let watcher = self.watcher.lock().await;
        if let Some(ref w) = *watcher {
            Some(w.state().await)
        } else {
            None
        }
    }

    /// 获取加载进度
    pub async fn load_progress(&self) -> Option<LoadProgress> {
        let watcher = self.watcher.lock().await;
        if let Some(ref w) = *watcher {
            Some(w.load_progress().await)
        } else {
            None
        }
    }

    /// 获取最近日志
    pub async fn recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let watcher_guard = self.watcher.lock().await;
        if let Some(ref w) = *watcher_guard {
            w.recent_logs(count).await
        } else {
            Vec::new()
        }
    }

    /// 获取退出通知接收器
    pub async fn exit_receiver(
        &self,
    ) -> Option<tokio::sync::watch::Receiver<Option<super::watcher::ExitInfo>>> {
        let watcher_guard = self.watcher.lock().await;
        watcher_guard.as_ref().map(|w| w.exit_receiver())
    }

    /// 标记 watcher 为手动停止（跳过崩溃分析）
    pub async fn mark_manual_stop(&self) {
        let watcher = self.watcher.lock().await;
        if let Some(ref w) = *watcher {
            w.mark_manual_stop();
        }
    }

    /// 停止游戏
    pub async fn stop_game(&self) {
        // 先标记手动停止，让 watcher 跳过崩溃分析
        {
            let watcher = self.watcher.lock().await;
            if let Some(ref w) = *watcher {
                w.mark_manual_stop();
            }
        }
        let child = self.child_process.lock().await;
        if let Some(ref child) = *child {
            let watcher = self.watcher.lock().await;
            if let Some(ref w) = *watcher {
                w.stop(child).await;
            }
        }
    }

    /// 获取当前进度
    pub async fn progress(&self) -> LaunchProgress {
        self.progress.read().await.clone()
    }

    /// 取消启动
    pub async fn cancel(&self) {
        *self.cancel_flag.lock().await = true;
    }
}
