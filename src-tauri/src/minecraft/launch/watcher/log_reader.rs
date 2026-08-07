//! 游戏 stdout/stderr 的统一日志读取。

use super::log_parser::parse_log_line;
use crate::log_warn;
use std::future::Future;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
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
