//! HTTP 请求日志（联机 API 调用追踪）
//!
//! 将客户端对 api-server 的每次请求记录到 `.Molaunch/logs/http_YYYY-MM-DD.log`，
//! 供开发者模式侧边栏加载表格展示，方便通过 `req_id` 追踪请求链路。

use crate::storage::Storage;
use serde::Serialize;
use std::path::PathBuf;

/// HTTP 日志条目（结构化，供 IPC 返回前端表格展示）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpLogEntry {
    /// 时间戳（本地时间，`YYYY-MM-DD HH:MM:SS.mmm`）
    pub timestamp: String,
    /// HTTP 方法（GET/POST/PUT/DELETE）
    pub method: String,
    /// 请求路径（不含 base_url，如 `/v3/auth/refresh`）
    pub path: String,
    /// HTTP 状态码
    pub status: u16,
    /// 响应中的 `req_id`（可能为空）
    pub req_id: String,
}

/// 当前日期对应的 HTTP 日志文件名（`http_YYYY-MM-DD.log`）
fn current_log_filename() -> String {
    let now = chrono::Local::now();
    format!("http_{}.log", now.format("%Y-%m-%d"))
}

/// HTTP 日志文件完整路径
fn log_file_path() -> PathBuf {
    let storage = Storage::instance();
    storage.logs_dir().join(current_log_filename())
}

/// 记录一条 HTTP 请求日志
///
/// - `method`: HTTP 方法（`GET`/`POST`/`PUT`/`DELETE`）
/// - `path`: 请求路径（建议传入不含 base_url 的路径，如 `/v3/auth/refresh`）
/// - `status`: HTTP 状态码
/// - `req_id`: 响应中的请求 ID（可能为空）
pub fn log_http_request(method: &str, path: &str, status: u16, req_id: &str) {
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let line = format!(
        "[{}] {} {} {} req_id={}\n",
        timestamp, method, path, status, req_id
    );

    let path_buf = log_file_path();
    // 确保日志目录存在
    if let Some(parent) = path_buf.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // 追加写入（失败仅记 warn，不影响业务流程）
    if let Err(e) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path_buf)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()))
    {
        crate::log_warn!("[HttpLog] 写入 HTTP 日志失败: {}", e);
    }
}

/// 从响应体文本中提取 `req_id`
///
/// 响应体为 JSON 时尝试解析 `req_id` 字段；非 JSON 或解析失败返回空字符串。
pub fn extract_req_id(body_text: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body_text) {
        if let Some(id) = v.get("req_id").and_then(|v| v.as_str()) {
            return id.to_string();
        }
    }
    String::new()
}

/// 读取指定日期的 HTTP 日志（结构化）
///
/// - `date`: 日期字符串（`YYYY-MM-DD`），None 表示今天
/// - `limit`: 最多返回条数（从末尾截取最新的），None 表示全部
pub fn read_http_logs(date: Option<&str>, limit: Option<usize>) -> Vec<HttpLogEntry> {
    let filename = match date {
        Some(d) => format!("http_{}.log", d),
        None => current_log_filename(),
    };
    let storage = Storage::instance();
    let path = storage.logs_dir().join(&filename);

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut entries: Vec<HttpLogEntry> = content
        .lines()
        .filter_map(parse_log_line)
        .collect();

    // 截取最新的 limit 条
    if let Some(n) = limit {
        let len = entries.len();
        if len > n {
            entries = entries.split_off(len - n);
        }
    }
    entries
}

/// 列出所有 HTTP 日志文件名（`http_*.log`，最新的在前）
pub fn list_http_log_files() -> Vec<String> {
    let storage = Storage::instance();
    let logs_dir = storage.logs_dir();

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("http_") && name.ends_with(".log") {
                files.push(name);
            }
        }
    }
    files.sort();
    files.reverse();
    files
}

/// 解析单行日志为 `HttpLogEntry`
///
/// 格式：`[2026-07-29 19:47:32.123] POST /v3/auth/refresh 200 req_id=xxx`
fn parse_log_line(line: &str) -> Option<HttpLogEntry> {
    // 去掉首尾的方括号包裹的时间戳
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let close_bracket = line.find(']')?;
    let timestamp = line[1..close_bracket].to_string();
    let rest = line[close_bracket + 1..].trim();

    // 拆分: METHOD PATH STATUS req_id=xxx
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    let status: u16 = parts[2].parse().ok()?;
    let req_id = parts
        .get(3)
        .and_then(|s| s.strip_prefix("req_id=").map(|s| s.to_string()))
        .unwrap_or_default();

    Some(HttpLogEntry {
        timestamp,
        method,
        path,
        status,
        req_id,
    })
}

#[cfg(test)]
#[path = "http_log_tests.rs"]
mod tests;
