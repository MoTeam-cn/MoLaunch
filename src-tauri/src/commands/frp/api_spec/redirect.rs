//! 厂商 API 重定向安全校验。

use crate::log_debug;

/// 最大重定向次数
const MAX_REDIRECT_HOPS: usize = 5;

/// 从初始 URL提取重定向白名单主机。
pub(super) fn allowed_host(url: &str) -> Result<String, String> {
    extract_host(url).ok_or_else(|| format!("无法解析 API URL 主机: {}", url))
}

/// 校验并解析单次重定向目标。
pub(super) fn next_url(
    current_url: &str,
    response: &reqwest::Response,
    allowed_host: &str,
    hops: &mut usize,
) -> Result<String, String> {
    *hops += 1;
    if *hops > MAX_REDIRECT_HOPS {
        return Err(format!(
            "厂商 API 重定向次数过多（超过 {} 次）",
            MAX_REDIRECT_HOPS
        ));
    }

    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            format!(
                "厂商 API 返回重定向但无 Location 头: HTTP {}",
                response.status()
            )
        })?;
    let next_url = resolve_url(current_url, location)?;
    let next_host =
        extract_host(&next_url).ok_or_else(|| format!("无法解析重定向 URL 主机: {}", next_url))?;

    if next_host != allowed_host {
        return Err(format!(
            "厂商 API 重定向到非白名单域名: {}（仅允许 {}）",
            next_host, allowed_host
        ));
    }

    log_debug!("[Frp] 厂商 API 重定向: {} -> {}", current_url, next_url);
    Ok(next_url)
}

/// 从 URL 提取主机名（小写，不含端口）。
fn extract_host(url: &str) -> Option<String> {
    let no_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host_end = no_scheme.find(['/', '?', '#']).unwrap_or(no_scheme.len());
    let host_with_port = &no_scheme[..host_end];
    let host = host_with_port.split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

/// 解析相对 Location 为绝对 URL。
fn resolve_url(base: &str, location: &str) -> Result<String, String> {
    if location.starts_with("http://") || location.starts_with("https://") {
        return Ok(location.to_string());
    }
    let (scheme, no_scheme) = base
        .strip_prefix("https://")
        .map(|rest| ("https", rest))
        .or_else(|| base.strip_prefix("http://").map(|rest| ("http", rest)))
        .ok_or_else(|| format!("无效的基准 URL: {}", base))?;
    let host_end = no_scheme.find('/').unwrap_or(no_scheme.len());
    let host = &no_scheme[..host_end];
    if location.starts_with('/') {
        Ok(format!("{}://{}{}", scheme, host, location))
    } else {
        Ok(format!("{}://{}/{}", scheme, host, location))
    }
}
