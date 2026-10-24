//! 探测：服务器 Range 支持检测与远程文件大小探测
//!
//! 两个坑必须同时规避：
//! 1. Range 请求拿到的是**压缩实体**的字节（服务器对 206 也可能带 Content-Encoding），
//!    分片合并出来是 gzip 流而不是原文 —— 因此响应带 Content-Encoding 时一律判为不支持分片。
//! 2. 探测请求默认会协商 gzip，此时 Content-Length / Content-Range 报的是**压缩后**长度，
//!    与实际落盘的解压长度不一致 —— 因此探测统一带 `Accept-Encoding: identity`。

use std::time::Duration;

use crate::log_debug;
use crate::utils::format;

/// Range 探测结果
pub struct RangeProbe {
    /// 服务端是否真的支持分片（206 且响应未被压缩）
    pub supports_range: bool,
    /// 服务端声明的文件大小：206 取 Content-Range 的 total，200 取 Content-Length；0 表示未知
    ///
    /// 注意：部分服务端（如 Fabric 官方元数据）会把**压缩后**的长度写进 Content-Range，
    /// 与完整 GET 解压后的字节数不一致，因此该值只用于分片切分与进度估算，不可作为校验依据。
    pub total: u64,
}

/// 探测服务端对 Range 的真实支持情况与文件大小
///
/// 不用 HEAD 预检：CF CDN（edge.forgecdn.net）HEAD 会虚假返回 `accept-ranges: bytes`，
/// 但实际 GET + Range 返回 404，导致分片必然失败。GET + Range 能准确反映真实响应。
pub async fn probe_range(client: &reqwest::Client, url: &str) -> RangeProbe {
    let unknown = RangeProbe {
        supports_range: false,
        total: 0,
    };

    let resp = match client
        .get(url)
        .header(reqwest::header::RANGE, "bytes=0-0")
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .timeout(Duration::from_secs(15))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log_debug!(
                "[Chunk] Range 探测失败: {}",
                crate::http::request_error_msg(&e)
            );
            return unknown;
        }
    };

    let status = resp.status().as_u16();
    // 响应被压缩时，Range 切的是压缩流的字节，合并必然得到损坏文件
    let encoded = resp
        .headers()
        .contains_key(reqwest::header::CONTENT_ENCODING);

    let total = if status == 206 {
        parse_content_range_total(
            resp.headers()
                .get(reqwest::header::CONTENT_RANGE)
                .and_then(|cr| cr.to_str().ok()),
        )
    } else if resp.status().is_success() {
        resp.content_length().unwrap_or(0)
    } else {
        0
    };

    if status == 206 && encoded {
        log_debug!("[Chunk] 服务端对 Range 响应做了压缩，禁用分片: {}", url);
    }
    if total > 0 {
        log_debug!("[Chunk] 探测文件大小: {} ({})", format::bytes(total), url);
    }

    RangeProbe {
        // 206 = 正确响应 Range；200 = 忽略了 Range（返回完整文件）；404 = 文件不存在
        supports_range: status == 206 && !encoded,
        total,
    }
}

/// 检测服务器是否真正支持 Range 请求
pub async fn supports_range(client: &reqwest::Client, url: &str) -> bool {
    probe_range(client, url).await.supports_range
}

/// 探测远程文件大小（0 表示无法探测）
pub async fn probe_file_size(client: &reqwest::Client, url: &str) -> u64 {
    probe_range(client, url).await.total
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
