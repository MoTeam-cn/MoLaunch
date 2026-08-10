//! HTTP 状态码处理 + 统一请求函数

use crate::minecraft::community::common::fmt_elapsed;
use crate::{log_debug, log_info, log_warn};
use std::time::Instant;

/// 判断 HTTP 状态码是否应该直接跳过（不重试）
/// - 403 Forbidden: 服务器拒绝，重试无意义
/// - 429 Too Many Requests: 频率限制，重试会继续被拒
pub fn should_skip_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 403 || status.as_u16() == 429
}

/// 判断 URL 是否为镜像源 URL
///
/// 用于 `DownloadManager::reorder_urls` 按源模式重排候选 URL。
/// 识别 BMCLAPI、mocdn.net、mcimirror.top 三类镜像域名。
pub fn is_mirror_url(url: &str) -> bool {
    url.contains("bmclapi") || url.contains("mocdn") || url.contains("mcimirror")
}

/// 统一的带回退的 HTTP GET 请求
///
/// 依次尝试 URLs，遇到 403/429 直接跳过，其他错误可重试。
pub async fn fetch_with_fallback(urls: &[String]) -> anyhow::Result<String> {
    let client = crate::http::get_client();
    let mut last_err = String::new();

    for url in urls {
        let start = Instant::now();
        match client.get(url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.text().await {
                        Ok(text) => {
                            log_info!("[Sources] 请求成功: {} ({})", url, fmt_elapsed(start));
                            return Ok(text);
                        }
                        Err(e) => {
                            log_debug!(
                                "[Sources] 读取响应失败 {}: {} ({})",
                                url,
                                e,
                                fmt_elapsed(start)
                            );
                            last_err = format!("{}: 读取失败 - {}", url, e);
                        }
                    }
                } else if should_skip_status(status) {
                    log_warn!(
                        "[Sources] {} 返回 {}，跳过不重试 ({})",
                        url,
                        status,
                        fmt_elapsed(start)
                    );
                    last_err = format!("{}: HTTP {}", url, status);
                    continue;
                } else {
                    log_debug!("[Sources] {} 返回 {} ({})", url, status, fmt_elapsed(start));
                    last_err = format!("{}: HTTP {}", url, status);
                }
            }
            Err(e) => {
                log_debug!("[Sources] 请求失败 {}: {} ({})", url, e, fmt_elapsed(start));
                last_err = format!("{}: {}", url, crate::http::request_error_msg(&e));
            }
        }
    }

    Err(anyhow::anyhow!("所有源均失败: {}", last_err))
}
