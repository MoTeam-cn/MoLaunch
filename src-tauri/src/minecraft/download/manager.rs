//! 下载管理器 - 多线程下载、进度追踪、文件校验、限速

use crate::{log_warn, log_debug};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use super::super::utils::file_checker::FileChecker;

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
struct RateLimiter {
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
    fn new(bytes_per_second: u64) -> Self {
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
    fn acquire(&mut self, requested: u64) -> u64 {
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
    fn wait_time_ms(&self, requested: u64) -> u64 {
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

/// 下载源模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DownloadSourceMode {
    /// 官方源优先
    Official,
    /// 镜像源优先
    Mirror,
    /// 智能模式（自动检测）
    Smart,
}

impl DownloadSourceMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "official" => Self::Official,
            "mirror" => Self::Mirror,
            "smart" => Self::Smart,
            _ => Self::Smart,
        }
    }
}

/// 下载管理器
pub struct DownloadManager {
    client: reqwest::Client,
    max_threads: usize,
    speed_limit: u64, // bytes/sec, 0 = 不限速
    source_mode: DownloadSourceMode,
    progress: Arc<Mutex<GlobalProgress>>,
}

impl DownloadManager {
    pub fn new(
        max_threads: usize,
        speed_limit: u64,
        source_mode: DownloadSourceMode,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            max_threads,
            speed_limit,
            source_mode,
            progress: Arc::new(Mutex::new(GlobalProgress::default())),
        }
    }

    /// 根据源模式重新排序 URLs
    /// Smart 模式：官方源和镜像源交替排列，先尝试官方，失败后自动切换
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
                official_urls.extend(mirror_urls);
                official_urls
            }
            DownloadSourceMode::Mirror => {
                mirror_urls.extend(official_urls);
                mirror_urls
            }
            DownloadSourceMode::Smart => {
                // 智能模式：交替排列，先官方后镜像
                // 这样下载时会先尝试官方源，失败后再尝试镜像源
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

    /// 根据源模式获取超时时间
    /// Smart 模式下官方源使用较短超时，快速失败后切换到镜像源
    #[allow(dead_code)]
    fn get_timeout_for_url(&self, url: &str) -> Duration {
        match self.source_mode {
            DownloadSourceMode::Smart => {
                if url.contains("bmclapi") || url.contains("mirror") {
                    // 镜像源：较长超时
                    Duration::from_secs(30)
                } else {
                    // 官方源：较短超时（3秒），快速失败后切换
                    Duration::from_secs(3)
                }
            }
            _ => Duration::from_secs(30),
        }
    }

    /// 批量下载文件
    pub async fn download_batch(
        &self,
        tasks: Vec<DownloadTask>,
        progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>>,
    ) -> Vec<DownloadProgress> {
        let total_bytes: u64 = tasks.iter().map(|t| t.expected_size.max(0) as u64).sum();
        
        let progress = Arc::new(Mutex::new(GlobalProgress {
            total_files: tasks.len(),
            total_bytes,
            is_active: true,
            ..Default::default()
        }));

        let results = Arc::new(Mutex::new(Vec::new()));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_threads));
        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(self.speed_limit)));

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

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let result = Self::download_single(
                    &client,
                    &task,
                    &urls,
                    Some(limiter),
                    source_mode,
                ).await;

                {
                    let mut p = prog.lock().await;
                    match &result.status {
                        DownloadStatus::Completed => p.completed_files += 1,
                        DownloadStatus::Failed => p.failed_files += 1,
                        DownloadStatus::Skipped => p.skipped_files += 1,
                        _ => {}
                    }
                    p.downloaded_bytes += result.downloaded;

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

        let mut p = progress.lock().await;
        p.is_active = false;

        let final_results = results.lock().await.clone();
        final_results
    }

    /// 下载单个文件
    async fn download_single(
        client: &reqwest::Client,
        task: &DownloadTask,
        urls: &[String],
        rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
        source_mode: DownloadSourceMode,
    ) -> DownloadProgress {
        // 检查文件是否已存在且有效
        let checker = FileChecker::new()
            .with_actual_size(task.expected_size)
            .with_hash(task.expected_hash.clone());

        if checker.is_valid(&task.local_path) {
            return DownloadProgress {
                task_id: task.id.clone(),
                downloaded: 0,
                total: 0,
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

        // 尝试每个 URL
        for url in urls {
            // 根据源模式获取超时时间
            let timeout = match source_mode {
                DownloadSourceMode::Smart => {
                    if url.contains("bmclapi") || url.contains("mirror") {
                        Duration::from_secs(30)
                    } else {
                        Duration::from_secs(3)
                    }
                }
                _ => Duration::from_secs(30),
            };

            match Self::download_from_url(client, url, &task.local_path, rate_limiter.clone(), timeout).await {
                Ok((downloaded, total, speed)) => {
                    // 校验
                    if let Some(err) = checker.check(&task.local_path) {
                        log_warn!("文件校验失败：{} - {}", task.local_path, err);
                        let _ = std::fs::remove_file(&task.local_path);
                        continue;
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
                    log_debug!("从 {} 下载失败：{}", url, e);
                    continue;
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
                        // 需要等待
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
        self.progress.lock().await.clone()
    }
}
