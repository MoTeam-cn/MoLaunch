//! 厂商 API 请求构造所需的 URL 与客户端能力。

pub(super) const DEFAULT_TIMEOUT_MS: u64 = 10_000;

/// 拼接 baseUrl + path。
pub(super) fn build_url(base_url: &str, path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Ok(base_url.trim_end_matches('/').to_string());
    }
    if path.starts_with("http://") || path.starts_with("https://") {
        return Ok(path.to_string());
    }
    let base = base_url.trim_end_matches('/');
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    Ok(format!("{}{}", base, path))
}

/// 构建厂商 API 专用 HTTP 客户端（复用全局无重定向单例，超时 per-request 覆盖）。
pub(super) fn build_vendor_client() -> Result<reqwest::Client, String> {
    Ok(crate::http::no_redirect_client())
}
