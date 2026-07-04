//! 下载管理器 - 多线程下载、进度追踪、文件校验、限速

use crate::{log_warn, log_debug, log_info};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use super::super::utils::file_checker::FileChecker;
use super::super::sources::DownloadSourceMode;

/// 下载任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub urls: Vec<String>,
    pub local_path: String,
    pub expected_size: i64,
    pub expected_hash: Option<String>,
}

/// 下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub task_id: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed: u64,
    pub status: DownloadStatus,
    pub error: Option<String>,
}

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Waiting,
    Connecting,
    Downloading,
    Verifying,
    Completed,
    Failed,
    Skipped,
}

/// 全局下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub skipped_files: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub current_speed: u64,
    pub is_active: bool,
}

impl Default for GlobalProgress {
    fn default() -> Self {
        Self {
            total_files: 0,
            completed_files: 0,
            failed_files: 0,
            skipped_files: 0,
            total_bytes: 0,
            downloaded_bytes: 0,
            current_speed: 0,
            is_active: false,
        }
    }
}

/// 令牌桶限速器
pub struct RateLimiter {
    /// 每秒允许的字节数
    bytes_per_second: u64,
    /// 当前可用令牌（字节）
    available_tokens: f64,
    /// 上次补充时间
    last_refill: Instant,
    /// 桶容量（允许突发）
    max_tokens: f64,
}

impl RateLimiter {
    pub fn new(bytes_per_second: u64) -> Self {
        let max_tokens = if bytes_per_second > 0 {
            bytes_per_second as f64 * 0.5 // 允许0.5秒的突发
        } else {
            f64::MAX
        };

        Self {
            bytes_per_second,
            available_tokens: max_tokens,
            last_refill: Instant::now(),
            max_tokens,
        }
    }

    /// 尝试获取令牌（字节数），返回实际可用的字节数
    pub fn acquire(&mut self, requested: u64) -> u64 {
        if self.bytes_per_second == 0 {
            return requested; // 不限速
        }

        self.refill();

        let available = self.available_tokens.min(requested as f64);
        if available >= 1.0 {
            let granted = available.floor() as u64;
            self.available_tokens -= granted as f64;
            granted
        } else {
            0 // 需要等待
        }
    }

    /// 补充令牌
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;

        let new_tokens = elapsed * self.bytes_per_second as f64;
        self.available_tokens = (self.available_tokens + new_tokens).min(self.max_tokens);
    }

    /// 获取需要等待的时间（毫秒）
    pub fn wait_time_ms(&self, requested: u64) -> u64 {
        if self.bytes_per_second == 0 {
            return 0;
        }

        let needed = requested as f64 - self.available_tokens;
        if needed <= 0.0 {
            return 0;
        }

        (needed / self.bytes_per_second as f64 * 1000.0) as u64
    }
}

/// 下载管理器
pub struct DownloadManager {
    client: reqwest::Client,
    max_threads: usize,
    chunk_count: usize,
    speed_limit: u64, // bytes/sec, 0 = 不限速
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
            DownloadSourceMode::Official => {
                // Official：只使用官方源
                official_urls
            }
            DownloadSourceMode::Mirror => {
                // Mirror：只使用镜像源
                mirror_urls
            }
            DownloadSourceMode::Smart => {
                // Smart：参考 PCL2，官方源优先，超时后切换到镜像源
                // 交替排列：官方1, 镜像1, 官方2, 镜像2, ...
                // 这样先尝试官方源，失败后立即尝试镜像源
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

        // 跟踪已使用分片下载的任务，防止字节重复统计
        let chunked_task_ids: Arc<StdMutex<std::collections::HashSet<String>>> =
            Arc::new(StdMutex::new(std::collections::HashSet::new()));

        // 滑动窗口速度计算
        let speed_window: Arc<Mutex<std::collections::VecDeque<(u64, Instant)>>> =
            Arc::new(Mutex::new(std::collections::VecDeque::new()));

        // 启动定期回调任务，每300ms调用一次回调来更新进度
        let prog_for_timer = progress.clone();
        let sw_for_timer = speed_window.clone();
        let callback_for_timer = progress_callback.clone();
        let timer_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(300));
            loop {
                interval.tick().await;
                // 检查是否活跃
                {
                    let p = prog_for_timer.lock().unwrap();
                    if !p.is_active {
                        break;
                    }
                }
                // 计算速度
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
                // 更新进度并回调
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
            let sw = speed_window.clone();
            let self_chunk_count = self.chunk_count;
            let chunked_ids = chunked_task_ids.clone();

            let chunked_ids_for_single = chunked_ids.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let result = Self::download_single(
                    &client,
                    &task,
                    &urls,
                    self_chunk_count,
                    Some(limiter),
                    source_mode,
                    Some(prog.clone()),
                    Some(chunked_ids_for_single),
                ).await;

                {
                    let mut p = prog.lock().unwrap();
                    match &result.status {
                        DownloadStatus::Completed => {
                            p.completed_files += 1;
                            // 分片下载的字节已由 chunks 实时更新，不重复累加
                            let is_chunked = chunked_ids.lock().unwrap().contains(&task.id);
                            if !is_chunked {
                                p.downloaded_bytes += result.downloaded;
                            }
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

                // 滑动窗口速度计算
                {
                    let mut window = sw.lock().await;
                    let mut p = prog.lock().unwrap();
                    window.push_back((p.downloaded_bytes, Instant::now()));
                    if window.len() > 10 { window.pop_front(); }
                    if window.len() >= 2 {
                        let (first_bytes, first_time) = window.front().unwrap();
                        let (last_bytes, last_time) = window.back().unwrap();
                        let bytes_diff = last_bytes.saturating_sub(*first_bytes);
                        let time_diff = last_time.duration_since(*first_time).as_secs_f64();
                        if time_diff > 0.0 {
                            p.current_speed = (bytes_diff as f64 / time_diff) as u64;
                        }
                    }
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

        // 等待定时器任务结束
        let _ = timer_handle.await;

        let final_results = results.lock().await.clone();
        final_results
    }

    /// 下载单个文件（统一逻辑：顺序尝试 URL，超时自动切换，大文件分片下载）
    async fn download_single(
        client: &reqwest::Client,
        task: &DownloadTask,
        urls: &[String],
        chunk_count: usize,
        rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
        source_mode: DownloadSourceMode,
        progress: Option<Arc<StdMutex<GlobalProgress>>>,
        chunked_task_ids: Option<Arc<StdMutex<std::collections::HashSet<String>>>>,
    ) -> DownloadProgress {
        // 检查文件是否已存在且有效
        let checker = FileChecker::new()
            .with_actual_size(task.expected_size)
            .with_hash(task.expected_hash.clone());

        if checker.is_valid(&task.local_path) {
            return DownloadProgress {
                task_id: task.id.clone(),
                downloaded: 0,
                total: task.expected_size.max(0) as u64,
                speed: 0,
                status: DownloadStatus::Skipped,
                error: None,
            };
        }

        // 确保目录存在
        if let Some(parent) = Path::new(&task.local_path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return DownloadProgress {
                    task_id: task.id.clone(),
                    downloaded: 0,
                    total: 0,
                    speed: 0,
                    status: DownloadStatus::Failed,
                    error: Some(format!("创建目录失败：{}", e)),
                };
            }
        }

        // 分片下载阈值：file_size / chunk_count > 1MB
        let chunk_threshold: u64 = 1_048_576;
        let file_size = task.expected_size.max(0) as u64;
        let can_chunk = chunk_count > 1 && file_size > 0 && (file_size / chunk_count as u64) > chunk_threshold;

        // 顺序尝试每个 URL，超时自动重试，总共最多 3 次
        let max_retries: usize = 3;
        let mut attempt = 0;

        'url_loop: for url in urls {
            // 确定超时时间
            let timeout = match source_mode {
                DownloadSourceMode::Smart => {
                    if url.contains("bmclapi") || url.contains("mirror") {
                        Duration::from_secs(10)
                    } else {
                        Duration::from_secs(5)
                    }
                }
                _ => Duration::from_secs(30),
            };

            while attempt < max_retries {
                attempt += 1;

                // 尝试分片下载（大文件 + 服务器支持 Range）
                if can_chunk {
                    if attempt == 1 {
                        log_debug!("[Download] 检测分片支持: {}", url);
                    }
                    if super::chunk::supports_range(client, url).await {
                        log_info!("[Download] 使用分片下载: {} ({} chunks, 尝试 {}/{})", url, chunk_count, attempt, max_retries);
                        let limiter = rate_limiter.clone().unwrap_or_else(|| {
                            Arc::new(Mutex::new(RateLimiter::new(0)))
                        });
                        let chunk_result = super::chunk::download_chunked(
                            client, url, &task.local_path,
                            file_size, chunk_count, limiter, progress.clone(),
                        ).await;

                        if chunk_result.status == DownloadStatus::Completed {
                            let checker = FileChecker::new()
                                .with_actual_size(task.expected_size)
                                .with_hash(task.expected_hash.clone());
                            if let Some(err) = checker.check(&task.local_path) {
                                log_warn!("[Chunk] 文件校验失败：{} - {}", task.local_path, err);
                                let _ = std::fs::remove_file(&task.local_path);
                            } else {
                                if let Some(ref ids) = chunked_task_ids {
                                    ids.lock().unwrap().insert(task.id.clone());
                                }
                                return DownloadProgress {
                                    task_id: task.id.clone(),
                                    downloaded: chunk_result.downloaded,
                                    total: chunk_result.total,
                                    speed: chunk_result.speed,
                                    status: DownloadStatus::Completed,
                                    error: None,
                                };
                            }
                        }
                        log_debug!("[Chunk] 分片下载失败: {:?}, 回退单流", chunk_result.error);
                    }
                }

                // 单流下载
                log_debug!("[Download] 从 {} 单流下载 (超时: {}s, 尝试 {}/{})", url, timeout.as_secs(), attempt, max_retries);
                match Self::download_from_url(client, url, &task.local_path, rate_limiter.clone(), timeout, progress.clone()).await {
                    Ok((downloaded, total, speed)) => {
                        let checker = if task.expected_size == 0 && downloaded > 0 {
                            FileChecker::new()
                                .with_actual_size(downloaded as i64)
                                .with_hash(task.expected_hash.clone())
                        } else {
                            FileChecker::new()
                                .with_actual_size(task.expected_size)
                                .with_hash(task.expected_hash.clone())
                        };

                        if let Some(err) = checker.check(&task.local_path) {
                            log_warn!("文件校验失败：{} - {}", task.local_path, err);
                            let _ = std::fs::remove_file(&task.local_path);
                            continue 'url_loop;
                        }

                        return DownloadProgress {
                            task_id: task.id.clone(),
                            downloaded,
                            total,
                            speed,
                            status: DownloadStatus::Completed,
                            error: None,
                        };
                    }
                    Err(e) => {
                        log_debug!("从 {} 下载失败 (尝试 {}/{}): {}", url, attempt, max_retries, e);
                        if attempt < max_retries {
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            }
        }

        DownloadProgress {
            task_id: task.id.clone(),
            downloaded: 0,
            total: 0,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some("所有下载源均失败".to_string()),
        }
    }

    /// 从单个 URL 下载（支持限速和动态超时）
    async fn download_from_url(
        client: &reqwest::Client,
        url: &str,
        local_path: &str,
        rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
        timeout: Duration,
        _progress: Option<Arc<StdMutex<GlobalProgress>>>,
    ) -> Result<(u64, u64, u64), Box<dyn std::error::Error + Send + Sync>> {
        let response = client.get(url).timeout(timeout).send().await?;

        if !response.status().is_success() {
            return Err(format!("HTTP 错误：{}", response.status()).into());
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let start_time = Instant::now();

        let mut stream = response.bytes_stream();
        let mut file = std::fs::File::create(local_path)?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
            let chunk_size = chunk.len() as u64;

            // 限速处理
            if let Some(ref limiter) = rate_limiter {
                let mut limiter = limiter.lock().await;
                let mut remaining = chunk_size;
                let mut offset: usize = 0;

                while remaining > 0 {
                    let granted = limiter.acquire(remaining);
                    if granted == 0 {
                        let wait_ms = limiter.wait_time_ms(remaining);
                        drop(limiter);
                        tokio::time::sleep(Duration::from_millis(wait_ms.max(10))).await;
                        limiter = rate_limiter.as_ref().unwrap().lock().await;
                        continue;
                    }

                    let end = (offset + granted as usize).min(chunk.len());
                    file.write_all(&chunk[offset..end])?;
                    offset = end;
                    remaining -= granted;
                    downloaded += granted;
                }
            } else {
                file.write_all(&chunk)?;
                downloaded += chunk_size;
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (downloaded as f64 / elapsed) as u64
        } else {
            0
        };

        Ok((downloaded, total_size, speed))
    }

    /// 获取当前进度
    pub async fn get_progress(&self) -> GlobalProgress {
        self.progress.lock().unwrap().clone()
    }
}
