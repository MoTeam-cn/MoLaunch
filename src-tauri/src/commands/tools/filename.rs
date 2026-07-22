//! 从 URL 获取文件名
//!
//! 流程：
//! 1. 校验 http/https 协议
//! 2. 优先发送 HEAD 请求（超时 10 秒），解析 `Content-Disposition` 响应头
//! 3. 如果 HEAD 不支持或没有 Content-Disposition，发送 GET 请求 with `Range: bytes=0-0`
//! 4. 解析 `Content-Disposition`：
//!    - 优先 `filename*=UTF-8''xxx`（RFC 5987，URL 解码）
//!    - 其次 `filename="xxx"` 或 `filename=xxx`
//!    - 都没有则从 URL 路径最后一段提取
//! 5. 获取 `Content-Length` 作为 file_size
//! 6. 返回 `FetchFilenameResult { filename, file_size }`

use std::time::Duration;

use reqwest::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, RANGE};

use crate::http::get_client;
use crate::commands::tools::types::{FetchFilenameParams, FetchFilenameResult};

/// 从 URL 获取文件名（与可选的文件大小）
pub async fn fetch_filename(params: FetchFilenameParams) -> Result<serde_json::Value, String> {
    let url = params.url;

    // 1. 协议白名单校验
    let lower_url = url.to_lowercase();
    if !lower_url.starts_with("http://") && !lower_url.starts_with("https://") {
        return Err("地址必须以 http:// 或 https:// 开头".to_string());
    }

    let client = get_client();

    // 2. 优先发送 HEAD 请求（10 秒超时）
    let head_result: Result<reqwest::Response, String> = async {
        let req = client
            .request(reqwest::Method::HEAD, &url)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("构建 HEAD 请求失败: {}", e))?;
        client
            .execute(req)
            .await
            .map_err(|e| format!("HEAD 请求失败: {}", e))
    }
    .await;

    let (filename, file_size) = match head_result {
        Ok(resp) => {
            let cd = resp.headers().get(CONTENT_DISPOSITION).and_then(|v| v.to_str().ok()).map(|s| s.to_string());
            let len = parse_content_length(resp.headers().get(CONTENT_LENGTH));
            match cd.as_deref() {
                Some(cd_str) => {
                    let parsed = parse_content_disposition(cd_str);
                    match parsed {
                        Some(name) => (name, len),
                        None => (extract_filename_from_url(&url), len),
                    }
                }
                None => (extract_filename_from_url(&url), len),
            }
        }
        Err(_) => {
            // 3. HEAD 不支持或失败：发送 GET 请求 with Range: bytes=0-0
            let resp = client
                .get(&url)
                .header(RANGE, "bytes=0-0")
                .timeout(Duration::from_secs(10))
                .send()
                .await
                .map_err(|e| format!("请求失败: {}", e))?;

            let cd = resp
                .headers()
                .get(CONTENT_DISPOSITION)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let len = parse_content_length(resp.headers().get(CONTENT_LENGTH));

            match cd.as_deref() {
                Some(cd_str) => match parse_content_disposition(cd_str) {
                    Some(name) => (name, len),
                    None => (extract_filename_from_url(&url), len),
                },
                None => (extract_filename_from_url(&url), len),
            }
        }
    };

    if filename.is_empty() {
        return Err("无法从 URL 解析文件名".to_string());
    }

    let result = FetchFilenameResult {
        filename,
        file_size: file_size.unwrap_or(0),
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 解析 `Content-Length` 响应头为 u64
fn parse_content_length(header: Option<&reqwest::header::HeaderValue>) -> Option<u64> {
    header
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
}

/// 解析 Content-Disposition 头，提取文件名
///
/// 优先匹配 `filename*=UTF-8''xxx`（RFC 5987），URL 解码；
/// 其次匹配 `filename="xxx"` 或 `filename=xxx`。
/// 都不匹配时返回 None（由调用方回退到 URL 路径提取）。
fn parse_content_disposition(cd: &str) -> Option<String> {
    // 1. 优先 RFC 5987 格式：filename*=UTF-8''xxx（或 charset''xxx）
    //    分号或逗号分隔的多个参数中查找
    for part in cd.split(|c| c == ';' || c == ',') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("filename*") {
            let rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '=');
            // rest 形如 "UTF-8''xxx" 或 "utf-8''xxx"
            // 找到 '' 分隔符
            if let Some(idx) = rest.find("''") {
                let encoded = &rest[idx + 2..];
                let decoded = urlencoding::decode(encoded)
                    .map(|c| c.into_owned())
                    .unwrap_or_else(|_| encoded.to_string());
                let decoded = decoded.trim();
                if !decoded.is_empty() {
                    return Some(decoded.to_string());
                }
            }
        }
    }

    // 2. 普通 filename="xxx" 或 filename=xxx
    for part in cd.split(|c| c == ';' || c == ',') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("filename") {
            let rest = rest.trim_start();
            let rest = rest.strip_prefix('=').unwrap_or(rest).trim();
            let name = if let Some(stripped) = rest.strip_prefix('"') {
                // 取首尾引号之间的内容
                if let Some(end) = stripped.find('"') {
                    &stripped[..end]
                } else {
                    stripped
                }
            } else {
                // 取到下一个分号或行尾为止
                rest.split(';').next().unwrap_or(rest).trim()
            };
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// 从 URL 路径最后一段提取文件名（URL 解码）
fn extract_filename_from_url(url: &str) -> String {
    // 去掉 query 和 fragment
    let no_query = url.split(['?', '#']).next().unwrap_or(url);
    // 取路径最后一段
    let last_segment = no_query
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .trim();
    if last_segment.is_empty() {
        return String::new();
    }
    urlencoding::decode(last_segment)
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| last_segment.to_string())
}
