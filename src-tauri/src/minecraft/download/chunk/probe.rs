//! 探测：服务器 Range 支持检测与远程文件大小探测

use std::time::Duration;

use crate::log_info;
use crate::utils::format;

/// 检测服务器是否支持 Range 请求
pub async fn supports_range(client: &reqwest::Client, url: &str) -> bool {
    match client
        .head(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
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

/// 探测远程文件大小（GET + Range:bytes=0-0，通过 Content-Range 拿总大小）
pub async fn probe_file_size(client: &reqwest::Client, url: &str) -> u64 {
    if let Ok(resp) = client
        .get(url)
        .header("Range", "bytes=0-0")
        .timeout(Duration::from_secs(15))
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Some(cr) = resp.headers().get("content-range") {
                if let Ok(s) = cr.to_str() {
                    if let Some(total) = s.rsplit('/').next() {
                        if let Ok(n) = total.parse::<u64>() {
                            log_info!("[Chunk] 探测文件大小: {} ({})", format::bytes(n), url);
                            return n;
                        }
                    }
                }
            }
        }
    }
    0
}
