//! 单文件分片并发下载模块
//! 将大文件拆分为多个 chunk，使用 HTTP Range 请求并发下载，最后合并。
//! 子模块：`probe`（Range 检测/大小探测）、`download`（分片下载）、`merge`（合并）

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;
use tokio::sync::Mutex;

use crate::utils::format;
use crate::{log_debug, log_warn};

use self::download::download_chunk;
use self::merge::merge_chunks;
use self::probe::probe_file_size;
use super::rate_limiter::RateLimiter;
use super::types::{DownloadStatus, GlobalProgress};

pub mod download;
pub mod merge;
pub mod probe;

// 对外保持 `super::chunk::supports_range` 调用路径稳定
pub use probe::supports_range;

/// 分片下载结果
pub struct ChunkDownloadResult {
    pub downloaded: u64,
    pub total: u64,
    pub speed: u64,
    pub status: DownloadStatus,
    pub error: Option<String>,
}

/// 分片下载单个文件
///
/// - `file_size` 为 0 时会自动探测（GET + Range:bytes=0-0，通过 Content-Range 拿总大小）
/// - `chunk_count` 分片数量
/// - 所有 chunk 共享同一个 RateLimiter
/// - 进度通过 `file_progress` 实时更新，供速度计算使用
pub async fn download_chunked(
    client: &reqwest::Client,
    url: &str,
    local_path: &str,
    mut file_size: u64,
    chunk_count: usize,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    file_progress: Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
) -> ChunkDownloadResult {
    // file_size=0 时自动探测（GET + Range:bytes=0-0，通过 Content-Range 拿总大小）
    // 不用 HEAD 是因为 Modrinth CDN 307 重定向后 HEAD 不返回 Content-Length
    if file_size == 0 {
        file_size = probe_file_size(client, url).await;
        if file_size == 0 {
            return ChunkDownloadResult {
                downloaded: 0,
                total: 0,
                speed: 0,
                status: DownloadStatus::Failed,
                error: Some("无法探测文件大小".into()),
            };
        }
        // 探测到的真实大小回写到全局 total_bytes（download_batch 初始化时按 expected_size=0 求和，
        // total_bytes 为 0，前端会一直显示「计算中...」）
        if let Some(ref fp) = file_progress {
            let mut p = fp.lock().unwrap();
            p.total_bytes = p.total_bytes.saturating_add(file_size);
        }
    }

    if file_size == 0 || chunk_count <= 1 {
        return ChunkDownloadResult {
            downloaded: 0,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some("invalid chunk params".into()),
        };
    }

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

    log_debug!(
        "[Chunk] 开始分片下载: {} ({}, {} chunks, 每块约 {})",
        local_path,
        format::bytes(file_size),
        chunk_count,
        format::bytes(chunk_size)
    );

    // 分片进度追踪：每个 chunk 已下载字节数
    let chunk_progress: Arc<Vec<StdMutex<u64>>> =
        Arc::new((0..chunk_count).map(|_| StdMutex::new(0u64)).collect());

    let start_time = Instant::now();
    let mut handles = Vec::with_capacity(chunk_count);

    for (i, (start, end)) in ranges.into_iter().enumerate() {
        let client = client.clone();
        let url = url.to_string();
        let part_path = format!("{}.part{}", local_path, i);
        let limiter = rate_limiter.clone();
        let prog = chunk_progress.clone();
        let file_prog = file_progress.clone();
        let pause = pause_flag.clone();
        let cancel = cancel_flag.clone();

        let handle = tokio::spawn(async move {
            let result = download_chunk(
                &client, &url, &part_path, start, end, limiter, prog, i, file_prog, pause, cancel,
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
                log_debug!("[Chunk] chunk {} 完成: {}", idx, format::bytes(bytes));
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
        // 断点续传：不清理 .part 文件，保留用于重试时续传
        // 回滚 file_progress：本次分片下载增量加的部分无效（文件将被重新下载），
        // 避免重试时 downloaded_bytes 持续累加导致进度偏高/超过 total
        if let Some(ref fp) = file_progress {
            let mut p = fp.lock().unwrap();
            p.downloaded_bytes = p.downloaded_bytes.saturating_sub(total_downloaded);
        }
        return ChunkDownloadResult {
            downloaded: total_downloaded,
            total: file_size,
            speed: 0,
            status: DownloadStatus::Failed,
            error: Some(last_error),
        };
    }

    // 合并分片到目标文件（合并前会校验每个 .part 大小匹配期望值，
    // 避免服务端提前断流导致的部分下载被误合并为损坏文件）
    if let Err(e) = merge_chunks(local_path, chunk_count, file_size) {
        // 断点续传：不清理 .part 文件，保留用于重试时续传
        // （合并失败通常是某分片大小不匹配，保留已下载部分可避免重头下载）
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

    log_debug!(
        "[Chunk] 分片下载完成: {} ({}, {:.1}s, {})",
        local_path,
        format::bytes(total_downloaded),
        elapsed,
        format::speed(speed)
    );

    ChunkDownloadResult {
        downloaded: total_downloaded,
        total: file_size,
        speed,
        status: DownloadStatus::Completed,
        error: None,
    }
}
