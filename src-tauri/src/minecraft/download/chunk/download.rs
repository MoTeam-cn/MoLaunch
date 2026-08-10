//! 单个分片的下载实现

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::Mutex;

use super::super::rate_limiter::RateLimiter;
use super::super::types::GlobalProgress;
use crate::utils::format;
use crate::{log_debug, log_warn};

/// 下载单个分片
///
/// 新增 `pause_flag` / `cancel_flag` 参数：分片数据流 loop 里检查暂停/取消信号，
/// 暂停时 sleep 等待恢复，取消时立即返回错误。
/// 修复：之前分片下载完全不受暂停/取消控制，一旦开始就停不下来。
///
/// 断点续传：入口检测 `.part` 文件已下载字节数，调整 Range 起点 + append 模式打开文件。
/// 失败时 `.part` 文件被保留（由 `download_chunked` 控制清理），重试时可续传。
#[allow(clippy::too_many_arguments)]
pub(super) async fn download_chunk(
    client: &reqwest::Client,
    url: &str,
    part_path: &str,
    start: u64,
    end: u64,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    chunk_progress: Arc<Vec<StdMutex<u64>>>,
    chunk_index: usize,
    file_progress: Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: Option<Arc<AtomicBool>>,
    cancel_flag: Option<Arc<AtomicBool>>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let expected_chunk_bytes = end.saturating_sub(start).saturating_add(1);

    // 断点续传：检测 .part 文件已下载字节数
    let resume_from = match std::fs::metadata(part_path) {
        Ok(meta) => {
            let existing = meta.len();
            if existing == expected_chunk_bytes {
                // .part 文件已完成，直接返回（可能是上次下载成功但未清理）
                log_debug!(
                    "[Chunk] chunk {} .part 已完成 ({} bytes)，跳过下载",
                    chunk_index,
                    existing
                );
                if let Some(cp) = chunk_progress.get(chunk_index) {
                    let mut cp = cp.lock().unwrap();
                    *cp = existing;
                }
                if let Some(ref fp) = file_progress {
                    let mut p = fp.lock().unwrap();
                    p.downloaded_bytes = p.downloaded_bytes.saturating_add(existing);
                }
                return Ok(existing);
            } else if existing > expected_chunk_bytes {
                // .part 文件比期望大（异常情况，可能是上次下载损坏），删除重新下载
                log_warn!(
                    "[Chunk] chunk {} .part 文件大小异常 ({} > {})，删除重新下载",
                    chunk_index,
                    existing,
                    expected_chunk_bytes
                );
                let _ = std::fs::remove_file(part_path);
                0
            } else if existing > 0 {
                // 续传：从 existing 字节开始
                log_debug!(
                    "[Chunk] chunk {} 续传: 已有 {} / {} bytes",
                    chunk_index,
                    existing,
                    expected_chunk_bytes
                );
                existing
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    let actual_start = start + resume_from;
    let range_header = format!("bytes={}-{}", actual_start, end);

    // 覆盖全局客户端的 30s timeout：大文件分片下载需要更长时间
    // 实际超时由下方 loop 里的"无数据流动 15s"控制，reqwest timeout 仅作 24h 兜底
    let response = client
        .get(url)
        .header("Range", &range_header)
        .timeout(Duration::from_secs(86400))
        .send()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            crate::http::request_error_msg(&e).into()
        })?;

    if !response.status().is_success() && response.status().as_u16() != 206 {
        return Err(format!("HTTP 错误：{}", response.status()).into());
    }

    let mut stream = response.bytes_stream();
    // append 模式打开文件（保留已有内容用于续传）
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(part_path)?;
    let mut downloaded: u64 = resume_from;

    // 单分片字节数上限：期望为 end-start+1，允许 2 倍冗余，
    // 防止被劫持镜像源在 Range 请求中返回超量数据导致磁盘耗尽
    let chunk_byte_limit = expected_chunk_bytes.saturating_mul(2);

    // 回滚闭包：失败时回滚本次增量加到 file_progress 的字节数
    // （失败 chunk 的 .part 文件会被 download_chunked 删除，已加的进度必须回滚，
    // 否则重试时 downloaded_bytes 会偏高甚至超过 total）
    let rollback = |downloaded: u64, file_progress: &Option<Arc<StdMutex<GlobalProgress>>>| {
        if downloaded > 0 {
            if let Some(ref p) = file_progress {
                let mut p = p.lock().unwrap();
                p.downloaded_bytes = p.downloaded_bytes.saturating_sub(downloaded);
            }
        }
    };

    // 分片下载是持续数据流：只要数据在流动就让它继续下载，
    // 真正卡死（15s 内没收到任何字节）才报错。
    // 这样慢速网络不会被误判 timeout，只有真断流才会失败。
    loop {
        // 检查取消信号：立即返回错误
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::Relaxed) {
                rollback(downloaded, &file_progress);
                return Err("下载已取消".into());
            }
        }

        // 检查暂停信号：暂停时 sleep 等待恢复或取消
        if let Some(ref flag) = pause_flag {
            while flag.load(Ordering::Relaxed) {
                // 暂停期间也检查取消信号
                if let Some(ref cf) = cancel_flag {
                    if cf.load(Ordering::Relaxed) {
                        rollback(downloaded, &file_progress);
                        return Err("下载已取消".into());
                    }
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        let next_chunk = tokio::time::timeout(Duration::from_secs(15), stream.next()).await;
        let chunk = match next_chunk {
            Err(_elapsed) => {
                rollback(downloaded, &file_progress);
                return Err(format!(
                    "chunk {} 下载超时（15s 无数据流动，已下载 {} / {}）",
                    chunk_index,
                    format::bytes(downloaded),
                    format::bytes(expected_chunk_bytes)
                )
                .into());
            }
            Ok(None) => break, // 流结束
            Ok(Some(Err(e))) => {
                rollback(downloaded, &file_progress);
                return Err(Box::from(e) as Box<dyn std::error::Error + Send + Sync>);
            }
            Ok(Some(Ok(c))) => c,
        };
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
                if let Err(e) = file.write_all(&chunk[offset..end_pos]) {
                    rollback(downloaded, &file_progress);
                    return Err(e.into());
                }
                offset = end_pos;
                remaining -= granted;
                downloaded += granted;
            }
        }

        // 更新本 chunk 的进度
        let prev_chunk_bytes = {
            let mut cp = chunk_progress[chunk_index].lock().unwrap();
            let prev = *cp;
            *cp = downloaded;
            prev
        };

        // 更新文件级进度（增量累加，避免多文件并发时覆盖）
        if let Some(ref fp) = file_progress {
            let delta = downloaded.saturating_sub(prev_chunk_bytes);
            let mut p = fp.lock().unwrap();
            p.downloaded_bytes = p.downloaded_bytes.saturating_add(delta);
        }

        // max_bytes 上限校验，防止被劫持镜像源返回超量数据导致磁盘耗尽
        if downloaded > chunk_byte_limit {
            rollback(downloaded, &file_progress);
            return Err(format!(
                "Chunk {} download size exceeded limit: {} > {}",
                chunk_index, downloaded, chunk_byte_limit
            )
            .into());
        }
    }

    Ok(downloaded)
}
