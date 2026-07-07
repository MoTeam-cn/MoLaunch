//! 单文件分片并发下载模块
//!
//! 将大文件拆分为多个 chunk，使用 HTTP Range 请求并发下载，最后合并。

use crate::{log_debug, log_info, log_warn};
use futures_util::StreamExt;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use super::types::{DownloadStatus, GlobalProgress};
use super::rate_limiter::RateLimiter;

/// 分片下载结果
pub struct ChunkDownloadResult {
    pub downloaded: u64,
    pub total: u64,
    pub speed: u64,
    pub status: DownloadStatus,
    pub error: Option<String>,
}

/// 检测服务器是否支持 Range 请求
pub async fn supports_range(client: &reqwest::Client, url: &str) -> bool {
    match client.head(url).timeout(Duration::from_secs(10)).send().await {
        Ok(resp) => {
            if let Some(accept_ranges) = resp.headers().get("accept-ranges") {
                accept_ranges.to_str().is_ok_and(|v| v.contains("bytes"))
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// 分片下载单个文件
///
/// - `file_size` 必须是已知的精确大小（来自 Content-Length 或版本 JSON）
/// - `chunk_count` 分片数量
/// - 所有 chunk 共享同一个 RateLimiter
/// - 进度通过 `file_progress` 实时更新，供速度计算使用
pub async fn download_chunked(
    client: &reqwest::Client,
    url: &str,
    local_path: &str,
    file_size: u64,
    chunk_count: usize,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    file_progress: Option<Arc<StdMutex<GlobalProgress>>>,
) -> ChunkDownloadResult {
    if file_size == 0 || chunk_count <= 1 {
        return ChunkDownloadResult {
            downloaded: 0,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some("invalid chunk params".into()),
        };
    }

    // 确保目录存在
    if let Some(parent) = Path::new(local_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ChunkDownloadResult {
                downloaded: 0,
                total: file_size,
                speed: 0,
                status: DownloadStatus::Failed,
                error: Some(format!("创建目录失败：{}", e)),
            };
        }
    }

    let chunk_size = file_size / chunk_count as u64;
    if chunk_size == 0 {
        // file_size < chunk_count，无法分片，避免整数下溢
        return ChunkDownloadResult {
            downloaded: 0,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some("file size too small to chunk".into()),
        };
    }
    let mut ranges: Vec<(u64, u64)> = Vec::with_capacity(chunk_count);

    for i in 0..chunk_count {
        let start = i as u64 * chunk_size;
        let end = if i == chunk_count - 1 {
            file_size - 1
        } else {
            start + chunk_size - 1
        };
        ranges.push((start, end));
    }

    log_info!(
        "[Chunk] 开始分片下载: {} ({} bytes, {} chunks, 每块约 {} bytes)",
        local_path,
        file_size,
        chunk_count,
        chunk_size
    );

    // 分片进度追踪：每个 chunk 已下载字节数
    let chunk_progress: Arc<Vec<StdMutex<u64>>> = Arc::new(
        (0..chunk_count).map(|_| StdMutex::new(0u64)).collect(),
    );

    let start_time = Instant::now();
    let mut handles = Vec::with_capacity(chunk_count);

    for (i, (start, end)) in ranges.into_iter().enumerate() {
        let client = client.clone();
        let url = url.to_string();
        let part_path = format!("{}.part{}", local_path, i);
        let limiter = rate_limiter.clone();
        let prog = chunk_progress.clone();
        let file_prog = file_progress.clone();

        let handle = tokio::spawn(async move {
            let result = download_chunk(
                &client,
                &url,
                &part_path,
                start,
                end,
                limiter,
                prog,
                i,
                file_prog,
            )
            .await;
            (i, result)
        });

        handles.push(handle);
    }

    // 等待所有 chunk 完成
    let mut all_ok = true;
    let mut total_downloaded: u64 = 0;
    let mut last_error = String::new();

    for handle in handles {
        match handle.await {
            Ok((idx, Ok(bytes))) => {
                total_downloaded += bytes;
                log_debug!("[Chunk] chunk {} 完成: {} bytes", idx, bytes);
            }
            Ok((idx, Err(e))) => {
                all_ok = false;
                last_error = format!("chunk {} 失败: {}", idx, e);
                log_warn!("[Chunk] {}", last_error);
            }
            Err(e) => {
                all_ok = false;
                last_error = format!("chunk task panic: {}", e);
                log_warn!("[Chunk] {}", last_error);
            }
        }
    }

    if !all_ok {
        // 清理临时文件
        for i in 0..chunk_count {
            let _ = std::fs::remove_file(format!("{}.part{}", local_path, i));
        }
        return ChunkDownloadResult {
            downloaded: total_downloaded,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some(last_error),
        };
    }

    // 合并分片到目标文件
    if let Err(e) = merge_chunks(local_path, chunk_count) {
        for i in 0..chunk_count {
            let _ = std::fs::remove_file(format!("{}.part{}", local_path, i));
        }
        return ChunkDownloadResult {
            downloaded: total_downloaded,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some(format!("合并分片失败：{}", e)),
        };
    }

    // 清理临时文件
    for i in 0..chunk_count {
        let _ = std::fs::remove_file(format!("{}.part{}", local_path, i));
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 {
        (total_downloaded as f64 / elapsed) as u64
    } else {
        0
    };

    log_info!(
        "[Chunk] 分片下载完成: {} ({} bytes, {:.1}s, {}/s)",
        local_path,
        total_downloaded,
        elapsed,
        format_speed(speed)
    );

    ChunkDownloadResult {
        downloaded: total_downloaded,
        total: file_size,
        speed,
        status: DownloadStatus::Completed,
        error: None,
    }
}

/// 下载单个分片
#[allow(clippy::too_many_arguments)]
async fn download_chunk(
    client: &reqwest::Client,
    url: &str,
    part_path: &str,
    start: u64,
    end: u64,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    chunk_progress: Arc<Vec<StdMutex<u64>>>,
    chunk_index: usize,
    file_progress: Option<Arc<StdMutex<GlobalProgress>>>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let range_header = format!("bytes={}-{}", start, end);

    let response = client
        .get(url)
        .header("Range", &range_header)
        .timeout(Duration::from_secs(60))
        .send()
        .await?;

    if !response.status().is_success() && response.status().as_u16() != 206 {
        return Err(format!("HTTP 错误：{}", response.status()).into());
    }

    let mut stream = response.bytes_stream();
    let mut file = std::fs::File::create(part_path)?;
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
        let chunk_len = chunk.len() as u64;

        // 限速处理
        {
            let mut limiter = rate_limiter.lock().await;
            let mut remaining = chunk_len;
            let mut offset: usize = 0;

            while remaining > 0 {
                let granted = limiter.acquire(remaining);
                if granted == 0 {
                    let wait_ms = limiter.wait_time_ms(remaining);
                    drop(limiter);
                    tokio::time::sleep(Duration::from_millis(wait_ms.max(10))).await;
                    limiter = rate_limiter.lock().await;
                    continue;
                }

                let end_pos = (offset + granted as usize).min(chunk.len());
                file.write_all(&chunk[offset..end_pos])?;
                offset = end_pos;
                remaining -= granted;
                downloaded += granted;
            }
        }

        // 更新本 chunk 的进度
        {
            let mut cp = chunk_progress[chunk_index].lock().unwrap();
            *cp = downloaded;
        }

        // 更新文件级进度（聚合所有 chunk）供速度计算使用
        if let Some(ref fp) = file_progress {
            let total_chunk_bytes: u64 = chunk_progress
                .iter()
                .map(|c| *c.lock().unwrap())
                .sum();
            let mut p = fp.lock().unwrap();
            p.downloaded_bytes = total_chunk_bytes;
        }
    }

    Ok(downloaded)
}

/// 按序合并分片到目标文件
fn merge_chunks(local_path: &str, chunk_count: usize) -> std::io::Result<()> {
    let tmp_path = format!("{}.merging", local_path);
    {
        let mut output = std::fs::File::create(&tmp_path)?;
        for i in 0..chunk_count {
            let part_path = format!("{}.part{}", local_path, i);
            let mut part_file = std::fs::File::open(&part_path)?;
            std::io::copy(&mut part_file, &mut output)?;
        }
        output.flush()?;
    }
    // 原子替换
    std::fs::rename(&tmp_path, local_path)?;
    Ok(())
}

/// 格式化速度
fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec >= 1_048_576 {
        format!("{:.1} MB/s", bytes_per_sec as f64 / 1_048_576.0)
    } else if bytes_per_sec >= 1024 {
        format!("{:.1} KB/s", bytes_per_sec as f64 / 1024.0)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}
