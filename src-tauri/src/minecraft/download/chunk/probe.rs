//! 探测：服务器 Range 支持检测与远程文件大小探测

use std::time::Duration;

use crate::log_info;
use crate::utils::format;

/// 检测服务器是否真正支持 Range 请求
///
/// 用 GET + Range:bytes=0-0 检测，检查 HTTP 206 Partial Content。
/// 不用 HEAD 预检：CF CDN（edge.forgecdn.net）HEAD 会虚假返回
/// `accept-ranges: bytes`，但实际 GET + Range 返回 404，导致分片必然失败。
/// GET + Range 能准确反映服务端对 Range 请求的真实响应。
pub async fn supports_range(client: &reqwest::Client, url: &str) -> bool {
    match client
        .get(url)
        .header("Range", "bytes=0-0")
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            // 206 Partial Content = 服务端正确处理了 Range 请求
            // 200 = 服务端忽略了 Range（返回完整文件），不支持分片
            // 404 = 文件不存在或对 Range 请求返回 404（CF CDN 就是这种行为）
            resp.status().as_u16() == 206
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
