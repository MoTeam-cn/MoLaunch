//! frpc stdout/stderr 异步捕获：脱敏 → 推送 frpc-log event → 批量写日志文件

use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::commands::frp::log_redact;
use crate::log_warn;

/// 单流日志捕获上限（1MB），超过后停止捕获防止内存膨胀
const MAX_STREAM_BYTES: usize = 1024 * 1024;

/// 异步捕获 frpc stdout/stderr 并写入日志文件 + 推送 frpc-log event
///
/// 每行格式：`[HH:MM:SS.ms] [LEVEL] <line>`。
/// LEVEL 推断：行内含 [E]/error/panic → ERROR，stderr → WARN，stdout → INFO。
///
/// 安全措施（对应设计文档 §7.3）：
/// - 每行经 `log_redact::redact_log` 脱敏后再写入文件/推送前端
/// - 单流捕获上限 1MB，超过后写入截断提示并停止捕获
pub(super) async fn capture_stream(
    reader: impl tokio::io::AsyncRead + Unpin,
    log_path: std::path::PathBuf,
    tunnel_id: &str,
    tunnel_name: &str,
    source: &str,
    app: AppHandle,
) {
    let mut reader = BufReader::new(reader);
    let mut lines = Vec::new();
    let mut total_bytes: usize = 0;
    let mut buf = String::new();

    loop {
        buf.clear();
        match reader.read_line(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let raw = buf.trim_end();
                if raw.is_empty() {
                    continue;
                }
                // 日志脱敏：将 token / 密码等敏感值替换为 ***
                let line = log_redact::redact_log(raw);

                // 1MB 截断检查：超过上限后写入截断提示并停止捕获该流
                let line_len = line.len();
                if total_bytes + line_len > MAX_STREAM_BYTES {
                    let timestamp = chrono_now();
                    let truncated = format!(
                        "[{}] [WARN] 日志输出已超过 1MB 上限，停止捕获该流\n",
                        timestamp
                    );
                    lines.push(truncated.clone());
                    flush_log(&log_path, &mut lines, tunnel_id);
                    let _ = app.emit(
                        "frpc-log",
                        serde_json::json!({
                            "tunnelId": tunnel_id,
                            "tunnelName": tunnel_name,
                            "line": truncated.trim_end(),
                            "timestamp": now_ms(),
                            "level": "WARN",
                        }),
                    );
                    log_warn!(
                        "[Frp] {} 日志超过 1MB 上限，停止捕获 ({})",
                        source,
                        tunnel_id
                    );
                    return;
                }
                total_bytes += line_len;

                let level = infer_log_level(source, &line);
                let timestamp = chrono_now();
                let formatted = format!("[{}] [{}] {}\n", timestamp, level, line);
                lines.push(formatted.clone());

                // 推送 frpc-log event
                let _ = app.emit(
                    "frpc-log",
                    serde_json::json!({
                        "tunnelId": tunnel_id,
                        "tunnelName": tunnel_name,
                        "line": formatted.trim_end(),
                        "timestamp": now_ms(),
                        "level": level,
                    }),
                );

                // 批量写入（每 50 行或达到一定大小写一次）
                if lines.len() >= 50 {
                    flush_log(&log_path, &mut lines, tunnel_id);
                }
            }
            Err(e) => {
                log_warn!("[Frp] 读取 {} 日志失败 ({}): {}", source, tunnel_id, e);
                break;
            }
        }
    }

    // 刷新剩余行
    flush_log(&log_path, &mut lines, tunnel_id);
}

/// 推断日志级别
///
/// - 行内含 "[E]" / "error" / "panic"（大小写不敏感） → ERROR
/// - source=stderr → WARN
/// - source=stdout → INFO
fn infer_log_level(source: &str, line: &str) -> &'static str {
    let lower = line.to_lowercase();
    if line.contains("[E]") || lower.contains("error") || lower.contains("panic") {
        "ERROR"
    } else if source == "stderr" {
        "WARN"
    } else {
        "INFO"
    }
}

/// 批量写入日志文件（追加模式）
fn flush_log(log_path: &std::path::Path, lines: &mut Vec<String>, tunnel_id: &str) {
    if lines.is_empty() {
        return;
    }
    use std::io::Write;
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        Ok(f) => f,
        Err(e) => {
            log_warn!("[Frp] 写入日志失败 ({}): {}", tunnel_id, e);
            return;
        }
    };
    for line in lines.drain(..) {
        let _ = file.write_all(line.as_bytes());
    }
}

/// 当前时间字符串（用于日志前缀，格式 HH:MM:SS.ms）
fn chrono_now() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

/// 当前 Unix 毫秒时间戳
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
