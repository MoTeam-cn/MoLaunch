//! 远程文件大小探测
//!
//! 用 GET + `Range: bytes=0-0` 而不是 HEAD：部分 CDN 的 HEAD 会虚假声明支持 Range，
//! 只有真实 GET 才反映服务端的实际行为。探测统一带 `Accept-Encoding: identity`，
//! 因为协商 gzip 后 Content-Length / Content-Range 报的是压缩后长度，与落盘字节数不符。

use std::time::Duration;

use crate::log_debug;
use crate::utils::format;

/// 探测远程文件大小（0 表示无法探测）
pub async fn probe_file_size(client: &reqwest::Client, url: &str) -> u64 {
    let resp = match client
        .get(url)
        .header(reqwest::header::RANGE, "bytes=0-0")
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .timeout(Duration::from_secs(15))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log_debug!(
                "[Chunk] 大小探测失败: {}",
                crate::http::request_error_msg(&e)
            );
            return 0;
        }
    };

    let status = resp.status().as_u16();
    let total = if status == 206 {
        parse_content_range_total(
            resp.headers()
                .get(reqwest::header::CONTENT_RANGE)
                .and_then(|value| value.to_str().ok()),
        )
    } else if resp.status().is_success() {
        resp.content_length().unwrap_or(0)
    } else {
        0
    };

    if total > 0 {
        log_debug!("[Chunk] 探测文件大小: {} ({})", format::bytes(total), url);
    }
    total
}

/// 解析 `Content-Range: bytes 0-0/1234` 里的总长度（无法解析返回 0）
fn parse_content_range_total(value: Option<&str>) -> u64 {
    value
        .and_then(|s| s.rsplit('/').next())
        .and_then(|total| total.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "probe_test.rs"]
mod probe_test;
