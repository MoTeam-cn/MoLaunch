//! 游戏 stdout/stderr 的统一日志读取。

use super::log_parser::parse_log_line;
use crate::log_warn;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::sync::Mutex;

pub(crate) async fn read_logs<R, F, Fut>(
    stream: R,
    source: &'static str,
    log_buffer: Arc<Mutex<std::collections::VecDeque<super::types::LogEntry>>>,
    max_lines: usize,
    mut on_line: F,
) where
    R: AsyncRead + Unpin,
    F: FnMut(String) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut reader = BufReader::new(stream);
    let mut buf: Vec<u8> = Vec::with_capacity(1024);

    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf).await {
            Ok(0) => break,
            Ok(_) => {
                if buf.last() == Some(&b'\n') {
                    buf.pop();
                    if buf.last() == Some(&b'\r') {
                        buf.pop();
                    }
                }
                let line = String::from_utf8_lossy(&buf).to_string();
                let entry = parse_log_line(&line, source);
                on_line(line).await;

                let mut buffer = log_buffer.lock().await;
                buffer.push_back(entry);
                if buffer.len() > max_lines {
                    buffer.pop_front();
                }
            }
            Err(e) => {
                log_warn!("[Watcher] {} 读取异常: {}", source, e);
                break;
            }
        }
    }
}

/// 增量读取游戏日志文件（logs/latest.log）的新增行。
///
/// stdout 在部分整合包/环境（SysOut appender 被移除、Java 输出缓冲等）下拿不到完整日志，
/// 而 latest.log 由 MC 的 File appender 保证写入，作为端口/进度检测的兜底来源。
/// 进程退出后停止；支持文件轮转（长度倒退时从头重新跟踪）。
pub(crate) async fn tail_latest_log<F, Fut>(
    log_path: std::path::PathBuf,
    process_exited: Arc<AtomicBool>,
    mut on_line: F,
) where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut offset: u64 = 0;
    let mut tail: Vec<u8> = Vec::new();

    loop {
        if process_exited.load(Ordering::Relaxed) {
            break;
        }
        let Ok(meta) = tokio::fs::metadata(&log_path).await else {
            offset = 0;
            tail.clear();
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        };
        let len = meta.len();
        if len < offset {
            offset = 0;
            tail.clear();
        }
        if len > offset {
            if let Ok(mut file) = tokio::fs::OpenOptions::new()
                .read(true)
                .open(&log_path)
                .await
            {
                if file.seek(SeekFrom::Start(offset)).await.is_ok() {
                    let mut chunk = Vec::new();
                    if file.read_to_end(&mut chunk).await.is_ok() {
                        tail.extend_from_slice(&chunk);
                        while let Some(idx) = tail.iter().position(|&b| b == b'\n') {
                            let line: Vec<u8> = tail.drain(..=idx).collect();
                            let mut text = line[..line.len() - 1].to_vec();
                            if text.last() == Some(&b'\r') {
                                text.pop();
                            }
                            if let Ok(text) = String::from_utf8(text) {
                                on_line(text).await;
                            }
                        }
                        offset = len;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}
