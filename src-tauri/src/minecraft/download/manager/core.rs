//! DownloadManager 主实现：批量下载编排（限速 / URL 重排 / 进度跟踪）

use crate::log_debug;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;
use tokio::sync::Mutex;

use super::super::super::sources::DownloadSourceMode;
use super::super::config::DownloadManagerConfig;
use super::super::downloader;
use super::super::rate_limiter::RateLimiter;
use super::super::types::{DownloadProgress, DownloadStatus, DownloadTask, GlobalProgress};
use super::state::ProgressTracker;
use crate::state::AppState;

/// 下载面板显隐事件（前端监听此事件控制浮动下载面板显示/隐藏）
pub const PANEL_STATE_EVENT: &str = "download-panel-state";

/// 下载管理器
pub struct DownloadManager {
    client: reqwest::Client,
    max_threads: usize,
    chunk_count: usize,
    speed_limit: u64,
    source_mode: DownloadSourceMode,
    progress: Arc<StdMutex<GlobalProgress>>,
    /// 取消信号（可选，由外部传入）
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
    /// 暂停信号（可选，由外部传入）
    pause_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
    /// 应用句柄（非 None 且非 silent 时下载开始/结束 emit 面板事件）
    app_handle: Option<AppHandle>,
    /// 静默下载（不 emit 面板事件，供 Java 下载 / 程序更新 / 启动补全等后台任务）
    silent: bool,
    /// 共享批次计数（来自 AppState，协调并发批次的面板显隐）
    active_batches: Option<Arc<std::sync::atomic::AtomicUsize>>,
}

impl DownloadManager {
    pub fn new(
        max_threads: usize,
        chunk_count: usize,
        speed_limit: u64,
        source_mode: DownloadSourceMode,
    ) -> Self {
        let client = crate::http::get_client();

        Self {
            client,
            max_threads,
            chunk_count,
            speed_limit,
            source_mode,
            progress: Arc::new(StdMutex::new(GlobalProgress::default())),
            cancel_flag: None,
            pause_flag: None,
            app_handle: None,
            silent: false,
            active_batches: None,
        }
    }

    /// 从 DownloadManagerConfig 构造（统一参数来源，避免硬编码）
    pub fn from_config(config: &DownloadManagerConfig) -> Self {
        let client = match config.user_agent.as_deref() {
            Some(ua) if !ua.is_empty() => crate::http::build_client_with_user_agent(ua, None),
            _ => crate::http::get_client(),
        };
        let mut manager = Self::new(
            config.max_threads,
            config.chunk_count,
            config.speed_limit,
            config.source_mode,
        );
        manager.client = client;
        manager.app_handle = config.app_handle.clone();
        manager.silent = config.silent;
        manager.active_batches = config.panel_counter.clone();
        manager
    }

    /// 从 AppState 提取下载配置并构造（统一收敛 3 处重复的 lock/extract/drop）
    ///
    /// `app_handle` / `panel_counter` 由 `DownloadManagerConfig::from_state` 自动填充
    pub async fn from_state(state: &AppState) -> Self {
        Self::from_config(&DownloadManagerConfig::from_state(state).await)
    }

    /// 设置取消信号（用于支持前端取消下载）
    pub fn with_cancel_flag(mut self, flag: Arc<std::sync::atomic::AtomicBool>) -> Self {
        self.cancel_flag = Some(flag);
        self
    }

    /// 设置暂停信号（用于支持前端暂停/恢复下载）
    pub fn with_pause_flag(mut self, flag: Arc<std::sync::atomic::AtomicBool>) -> Self {
        self.pause_flag = Some(flag);
        self
    }

    /// 设置静默模式（不 emit 面板显隐事件）
    ///
    /// 供后台任务使用：Java 下载 / 程序更新 / 启动时文件补全等场景
    /// 不应打扰用户弹出下载面板。
    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// 接入共享批次计数（协调并发批次的面板显隐）
    ///
    /// 多个 DownloadManager 实例共享同一个 `AppState.panel_active_count`，
    /// 首个批次开始 emit 显示、最后批次结束 emit 隐藏，避免并发下载时面板提前消失。
    pub fn with_panel_counter(mut self, counter: Arc<std::sync::atomic::AtomicUsize>) -> Self {
        self.active_batches = Some(counter);
        self
    }

    /// 通知前端下载面板显隐（silent 或缺少 AppHandle 时静默跳过）
    fn notify_panel(&self, visible: bool) {
        if self.silent {
            return;
        }
        let Some(app) = &self.app_handle else {
            return;
        };
        let _ = app.emit(PANEL_STATE_EVENT, serde_json::json!({ "visible": visible }));
    }

    /// 检查是否已取消
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancel_flag
            .as_ref()
            .map(|f| f.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(false)
    }

    /// 检查是否已暂停
    fn is_paused(&self) -> bool {
        self.pause_flag
            .as_ref()
            .map(|f| f.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(false)
    }

    /// 获取当前源模式（用于构造 URL）
    pub fn source_mode(&self) -> DownloadSourceMode {
        self.source_mode
    }

    /// 根据源模式重新排序 URLs
    fn reorder_urls(&self, urls: &[String]) -> Vec<String> {
        if urls.len() <= 1 {
            return urls.to_vec();
        }

        let mut official_urls = Vec::new();
        let mut mirror_urls = Vec::new();

        for url in urls {
            if super::super::super::sources::is_mirror_url(url) {
                mirror_urls.push(url.clone());
            } else {
                official_urls.push(url.clone());
            }
        }

        match self.source_mode {
            DownloadSourceMode::Official => official_urls,
            DownloadSourceMode::Mirror => mirror_urls,
            DownloadSourceMode::Smart => {
                let mut result = Vec::new();
                let max_len = official_urls.len().max(mirror_urls.len());
                for i in 0..max_len {
                    if i < official_urls.len() {
                        result.push(official_urls[i].clone());
                    }
                    if i < mirror_urls.len() {
                        result.push(mirror_urls[i].clone());
                    }
                }
                result
            }
        }
    }

    /// 批量下载文件
    pub async fn download_batch(
        &self,
        tasks: Vec<DownloadTask>,
        progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
    ) -> Vec<DownloadProgress> {
        // 通知面板显示：首个非静默批次开始时 emit 显示（并发批次由共享计数器协调）
        let panel_enabled = !self.silent && self.app_handle.is_some();
        if panel_enabled {
            let first = match &self.active_batches {
                Some(c) => c.fetch_add(1, Ordering::SeqCst) == 0,
                None => true,
            };
            if first {
                self.notify_panel(true);
            }
        }

        let total_bytes: u64 = tasks.iter().map(|t| t.expected_size.max(0) as u64).sum();

        let tracker = Arc::new(ProgressTracker::new(
            tasks.len(),
            total_bytes,
            progress_callback,
        ));
        let timer_handle = tracker.start_timer();

        let results = Arc::new(Mutex::new(Vec::new()));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_threads));
        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(self.speed_limit)));
        let chunked_task_ids: Arc<StdMutex<std::collections::HashSet<String>>> =
            Arc::new(StdMutex::new(std::collections::HashSet::new()));

        let mut handles = Vec::new();
        let total_task_count = tasks.len();

        for (task_index, task) in tasks.into_iter().enumerate() {
            // 检查取消信号
            if self.is_cancelled() {
                let remaining = total_task_count - task_index;
                log_debug!("[Download] 检测到取消信号，跳过剩余 {} 个任务", remaining);
                break;
            }

            // 检查暂停信号：暂停时等待恢复或取消（只打印一次暂停日志）
            if self.is_paused() && !self.is_cancelled() {
                log_debug!("[Download] 下载已暂停，等待恢复...");
            }
            while self.is_paused() && !self.is_cancelled() {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            if self.is_cancelled() {
                let remaining = total_task_count - task_index;
                log_debug!(
                    "[Download] 暂停期间检测到取消信号，跳过剩余 {} 个任务",
                    remaining
                );
                break;
            }

            let sem = semaphore.clone();
            let results = results.clone();
            let client = self.client.clone();
            let limiter = rate_limiter.clone();
            let urls = self.reorder_urls(&task.urls);
            let source_mode = self.source_mode;
            let self_chunk_count = self.chunk_count;
            let chunked_ids = chunked_task_ids.clone();
            let cancel_flag = self.cancel_flag.clone();
            let pause_flag = self.pause_flag.clone();
            let batch_tracker = tracker.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                // 获取许可后再次检查取消信号
                if let Some(ref flag) = cancel_flag {
                    if flag.load(std::sync::atomic::Ordering::Relaxed) {
                        let result = DownloadProgress {
                            task_id: task.id.clone(),
                            status: DownloadStatus::Failed,
                            downloaded: 0,
                            total: task.expected_size as u64,
                            speed: 0,
                            error: Some("下载已取消".to_string()),
                        };
                        results.lock().await.push(result);
                        return;
                    }
                }

                let result = downloader::download_single(
                    &client,
                    &task,
                    &urls,
                    self_chunk_count,
                    Some(limiter),
                    source_mode,
                    Some(batch_tracker.handle()),
                    Some(chunked_ids.clone()),
                    pause_flag.clone(),
                    cancel_flag.clone(),
                )
                .await;

                match &result.status {
                    DownloadStatus::Completed => batch_tracker.mark_completed(),
                    DownloadStatus::Failed => batch_tracker.mark_failed(),
                    DownloadStatus::Skipped => {
                        batch_tracker.mark_skipped(result.total);
                    }
                    _ => {}
                }
                batch_tracker.notify();

                results.lock().await.push(result);
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        tracker.finish();

        let _ = timer_handle.await;

        // 通知面板隐藏：最后批次结束时 emit 隐藏
        if panel_enabled {
            let last = match &self.active_batches {
                Some(c) => c.fetch_sub(1, Ordering::SeqCst) == 1,
                None => true,
            };
            if last {
                self.notify_panel(false);
            }
        }

        let final_results = results.lock().await.clone();
        final_results
    }

    /// 获取当前进度
    pub async fn get_progress(&self) -> GlobalProgress {
        self.progress.lock().unwrap().clone()
    }
}

#[cfg(test)]
#[path = "../manager_tests.rs"]
mod tests;
