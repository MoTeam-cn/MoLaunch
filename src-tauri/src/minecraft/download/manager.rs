//! Download manager - batch download with progress tracking

use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use super::super::sources::DownloadSourceMode;
use super::downloader;
use super::rate_limiter::RateLimiter;
use super::types::{DownloadProgress, DownloadStatus, DownloadTask, GlobalProgress};

/// 下载管理器
pub struct DownloadManager {
    client: reqwest::Client,
    max_threads: usize,
    chunk_count: usize,
    speed_limit: u64,
    source_mode: DownloadSourceMode,
    progress: Arc<StdMutex<GlobalProgress>>,
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
        }
    }

    /// 根据源模式重新排序 URLs
    fn reorder_urls(&self, urls: &[String]) -> Vec<String> {
        if urls.len() <= 1 {
            return urls.to_vec();
        }

        let mut official_urls = Vec::new();
        let mut mirror_urls = Vec::new();

        for url in urls {
            if url.contains("bmclapi") || url.contains("mirror") {
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
        let total_bytes: u64 = tasks.iter().map(|t| t.expected_size.max(0) as u64).sum();

        let progress = Arc::new(StdMutex::new(GlobalProgress {
            total_files: tasks.len(),
            total_bytes,
            is_active: true,
            ..Default::default()
        }));

        let results = Arc::new(Mutex::new(Vec::new()));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_threads));
        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(self.speed_limit)));
        let chunked_task_ids: Arc<StdMutex<std::collections::HashSet<String>>> =
            Arc::new(StdMutex::new(std::collections::HashSet::new()));

        // 滑动窗口速度计算
        let speed_window: Arc<Mutex<std::collections::VecDeque<(u64, Instant)>>> =
            Arc::new(Mutex::new(std::collections::VecDeque::new()));

        // 启动定期回调任务
        let prog_for_timer = progress.clone();
        let sw_for_timer = speed_window.clone();
        let callback_for_timer = progress_callback.clone();
        let timer_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(300));
            loop {
                interval.tick().await;
                {
                    let p = prog_for_timer.lock().unwrap();
                    if !p.is_active {
                        break;
                    }
                }
                // 在 timer 中 push 速度窗口，让下载过程中也能计算速度
                // 注意：StdMutex 的 MutexGuard 不是 Send，必须在 await 前释放
                let downloaded_snapshot = {
                    let p = prog_for_timer.lock().unwrap();
                    p.downloaded_bytes
                };
                {
                    let mut window = sw_for_timer.lock().await;
                    window.push_back((downloaded_snapshot, Instant::now()));
                    if window.len() > 10 {
                        window.pop_front();
                    }
                }
                let speed = {
                    let window = sw_for_timer.lock().await;
                    if window.len() >= 2 {
                        let (first_bytes, first_time) = window.front().unwrap();
                        let (last_bytes, last_time) = window.back().unwrap();
                        let bytes_diff = last_bytes.saturating_sub(*first_bytes);
                        let time_diff = last_time.duration_since(*first_time).as_secs_f64();
                        if time_diff > 0.0 {
                            (bytes_diff as f64 / time_diff) as u64
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                };
                let p_snapshot = {
                    let mut p = prog_for_timer.lock().unwrap();
                    p.current_speed = speed;
                    p.clone()
                };
                if let Some(ref cb) = callback_for_timer {
                    cb(p_snapshot);
                }
            }
        });

        let mut handles = Vec::new();

        for task in tasks {
            let sem = semaphore.clone();
            let prog = progress.clone();
            let results = results.clone();
            let client = self.client.clone();
            let callback = progress_callback.clone();
            let limiter = rate_limiter.clone();
            let urls = self.reorder_urls(&task.urls);
            let source_mode = self.source_mode;
            let self_chunk_count = self.chunk_count;
            let chunked_ids = chunked_task_ids.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let result = downloader::download_single(
                    &client,
                    &task,
                    &urls,
                    self_chunk_count,
                    Some(limiter),
                    source_mode,
                    Some(prog.clone()),
                    Some(chunked_ids.clone()),
                )
                .await;

                {
                    let mut p = prog.lock().unwrap();
                    match &result.status {
                        DownloadStatus::Completed => {
                            p.completed_files += 1;
                            // 分片下载和单流下载过程中都已增量更新 downloaded_bytes，
                            // 这里不再重复加（避免进度偏高/超过 total）
                        }
                        DownloadStatus::Failed => p.failed_files += 1,
                        DownloadStatus::Skipped => {
                            p.skipped_files += 1;
                            let skipped_size = result.total;
                            p.total_bytes = p.total_bytes.saturating_sub(skipped_size);
                        }
                        _ => {}
                    }
                }

                {
                    let p = prog.lock().unwrap();
                    if let Some(ref cb) = callback {
                        cb(p.clone());
                    }
                }

                results.lock().await.push(result);
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        {
            let mut p = progress.lock().unwrap();
            p.is_active = false;
        }

        let _ = timer_handle.await;

        let final_results = results.lock().await.clone();
        final_results
    }

    /// 获取当前进度
    pub async fn get_progress(&self) -> GlobalProgress {
        self.progress.lock().unwrap().clone()
    }
}
