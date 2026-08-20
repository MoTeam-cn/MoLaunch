//! DownloadManager 批次下载实现（并发调度 / 取消暂停 / URL 重排）

use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Duration;

use tokio::sync::Mutex;

use super::super::downloader;
use super::super::rate_limiter::RateLimiter;
use super::super::types::{DownloadProgress, DownloadStatus, DownloadTask, GlobalProgress};
use super::state::ProgressTracker;
use super::DownloadManager;
use crate::log_debug;

impl DownloadManager {
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

    /// 根据源模式重新排序 URLs
    pub(crate) fn reorder_urls(&self, urls: &[String]) -> Vec<String> {
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
            super::super::super::sources::DownloadSourceMode::Official => official_urls,
            super::super::super::sources::DownloadSourceMode::Mirror => mirror_urls,
            super::super::super::sources::DownloadSourceMode::Smart => {
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
        self.hold_panel();

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
            let urls = if self.preserve_order {
                task.urls.clone()
            } else {
                self.reorder_urls(&task.urls)
            };
            let source_mode = self.source_mode;
            let self_chunk_count = self.chunk_count;
            let chunked_ids = chunked_task_ids.clone();
            let cancel_flag = self.cancel_flag.clone();
            let pause_flag = self.pause_flag.clone();
            let content_validator = self.content_validator.clone();
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
                    content_validator,
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
        self.release_panel();

        let final_results = results.lock().await.clone();
        final_results
    }
}
