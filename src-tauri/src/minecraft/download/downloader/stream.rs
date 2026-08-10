//! 单 URL 流式下载（HTTP 请求 + 限速 + 暂停/取消 + 进度更新）

use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use futures_util::StreamExt;

use super::super::rate_limiter::RateLimiter;
use super::super::types::GlobalProgress;

#[allow(clippy::too_many_arguments)]
/// 从单个 URL 下载（支持限速和动态超时，实时更新进度）
///
/// 超时策略（与 chunk 下载一致，避免大文件被整体超时误杀）：
/// - 连接 + 响应头阶段：用传入的 `timeout`（Smart 模式 5s/10s）
/// - body 流式读取阶段：无数据流动 15s 才报错（大文件慢速网络不受影响）
pub(super) async fn download_from_url(
    client: &reqwest::Client,
    url: &str,
    local_path: &str,
    expected_size: u64,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    timeout: Duration,
    progress: Option<Arc<StdMutex<GlobalProgress>>>,
    pause_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> Result<(u64, u64, u64), Box<dyn std::error::Error + Send + Sync>> {
    // 连接 + 响应头阶段用 tokio::time::timeout 包裹 send()（5s/10s）
    // 覆盖全局客户端的 30s timeout 为 24h 兜底，避免大文件 body 读取被误杀
    // 实际超时由下方 loop 里的"无数据流动 15s"控制
    let response = tokio::time::timeout(
        timeout,
        client.get(url).timeout(Duration::from_secs(86400)).send(),
    )
    .await
    .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> {
        format!("连接超时（{}s）", timeout.as_secs()).into()
    })??;

    if !response.status().is_success() {
        return Err(format!("HTTP 错误：{}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);

    // 回填全局 total_bytes：download_batch 初始化时按 expected_size 求和，
    // 整合包归档 expected_size=0 导致初始 total_bytes=0。单流路径拿到 content_length
    // 后必须回填，否则前端 stage 显示「0/1 文件」、global_bytes_total 显示「计算中...」。
    // 仅 expected_size 未知(=0)时才回填——已知大小文件已在 download_batch 初始化时计入，
    // 无条件回填会把该文件大小重复累加，导致 total_bytes 虚高、随下载过程持续增长。
    if expected_size == 0 && total_size > 0 {
        if let Some(ref p) = progress {
            let mut p = p.lock().unwrap();
            p.total_bytes = p.total_bytes.saturating_add(total_size);
        }
    }

    let mut downloaded: u64 = 0;
    let start_time = Instant::now();

    // 下载流字节数上限：已知期望大小时允许 2 倍冗余，否则使用绝对上限
    let byte_limit = if expected_size > 0 {
        expected_size.saturating_mul(2)
    } else {
        super::MAX_UNVERIFIED_BYTES
    };

    let mut stream = response.bytes_stream();
    // 确保父目录存在（取消/清理可能导致目录被删除）
    if let Some(parent) = Path::new(local_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = std::fs::File::create(local_path)?;

    // 回滚已增量加到 progress 的字节数和 total_bytes（downloaded>0 或已回填过 total_size 时才需要）
    // 失败时回滚 total 避免 download_single 的 3 次重试导致 total 翻倍
    let rollback_progress =
        move |downloaded: u64, progress: &Option<Arc<StdMutex<GlobalProgress>>>| {
            if downloaded > 0 || (expected_size == 0 && total_size > 0) {
                if let Some(ref p) = progress {
                    let mut p = p.lock().unwrap();
                    p.downloaded_bytes = p.downloaded_bytes.saturating_sub(downloaded);
                    p.total_bytes = p.total_bytes.saturating_sub(total_size);
                }
            }
        };

    // body 读取阶段：无数据流动 15s 才报错（与 chunk 下载一致）
    // 这样大文件慢速网络不会被整体超时误杀，只有真断流才会失败
    const STREAM_IDLE_TIMEOUT_SECS: u64 = 15;

    loop {
        // 检查取消信号
        if let Some(ref flag) = cancel_flag {
            if flag.load(std::sync::atomic::Ordering::Relaxed) {
                rollback_progress(downloaded, &progress);
                return Err("下载已取消".into());
            }
        }
        // 检查暂停信号
        if let Some(ref flag) = pause_flag {
            while flag.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(ref cf) = cancel_flag {
                    if cf.load(std::sync::atomic::Ordering::Relaxed) {
                        rollback_progress(downloaded, &progress);
                        return Err("下载已取消".into());
                    }
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        let next_chunk =
            tokio::time::timeout(Duration::from_secs(STREAM_IDLE_TIMEOUT_SECS), stream.next())
                .await;
        let chunk = match next_chunk {
            Err(_elapsed) => {
                rollback_progress(downloaded, &progress);
                return Err(format!(
                    "单流下载超时（{}s 无数据流动，已下载 {}）",
                    STREAM_IDLE_TIMEOUT_SECS,
                    crate::utils::format::bytes(downloaded)
                )
                .into());
            }
            Ok(None) => break, // 流结束
            Ok(Some(Err(e))) => {
                rollback_progress(downloaded, &progress);
                return Err(Box::from(e) as Box<dyn std::error::Error + Send + Sync>);
            }
            Ok(Some(Ok(c))) => c,
        };
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
                if let Err(e) = file.write_all(&chunk[offset..end]) {
                    rollback_progress(downloaded, &progress);
                    return Err(e.into());
                }
                offset = end;
                remaining -= granted;
                downloaded += granted;
            }
        } else {
            if let Err(e) = file.write_all(&chunk) {
                rollback_progress(downloaded, &progress);
                return Err(e.into());
            }
            downloaded += chunk_size;
        }

        // 增量更新全局进度（与分片下载保持一致，让前端实时看到下载进度）
        if let Some(ref p) = progress {
            let mut p = p.lock().unwrap();
            p.downloaded_bytes = p.downloaded_bytes.saturating_add(chunk_size);
        }

        // max_bytes 上限校验，防止被劫持镜像源返回无限流导致磁盘耗尽
        if downloaded > byte_limit {
            rollback_progress(downloaded, &progress);
            return Err(format!(
                "Download size exceeded limit: {} > {}",
                downloaded, byte_limit
            )
            .into());
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
