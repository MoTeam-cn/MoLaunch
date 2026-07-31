//! URL 与客户端辅助函数

use super::{DEFAULT_TIMEOUT_MS, MAX_TIMEOUT_MS, MIN_TIMEOUT_MS};
use std::time::Duration;

/// 拼接 baseUrl + path
///
/// 处理 trailing slash：`https://api.x.com` + `/v1/config` → `https://api.x.com/v1/config`。
/// path 若已是绝对 URL（http/https 开头）直接返回。
pub(super) fn build_url(base_url: &str, path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Ok(base_url.trim_end_matches('/').to_string());
    }
    if path.starts_with("http://") || path.starts_with("https://") {
        return Ok(path.to_string());
    }
    let base = base_url.trim_end_matches('/');
    let p = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    Ok(format!("{}{}", base, p))
}

/// 计算请求超时
///
/// schema.timeout 缺省时取默认 10s，超过 30s 截断，低于 1s 抬升。
pub(super) fn compute_timeout(timeout_ms: Option<u64>) -> Duration {
    let ms = timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS);
    let clamped = ms.clamp(MIN_TIMEOUT_MS, MAX_TIMEOUT_MS);
    Duration::from_millis(clamped)
}

/// 构建厂商 API 专用 HTTP 客户端（no-redirect + 内置根证书）
///
/// 不复用 `crate::http::get_client()`：全局客户端 redirect policy 不可覆盖，
/// 无法实现 §7.6.6 要求的同域重定向白名单校验。
pub(super) fn build_vendor_client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(timeout)
        .tls_built_in_root_certs(true)
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {}", e))
}

/// 填充参数模板中的上下文占位符
///
/// 当前支持：`{device_id}` / `{provider_id}`。
/// 未知占位符保持原样（厂商 schema 配置错误时由 API 端拒绝）。
pub(super) fn fill_param_template(template: &str, device_id: &str, provider_id: &str) -> String {
    template
        .replace("{device_id}", device_id)
        .replace("{provider_id}", provider_id)
}

/// 从 URL 提取主机名（小写，不含端口）
pub(super) fn extract_host(url: &str) -> Option<String> {
    let no_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host_end = no_scheme
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(no_scheme.len());
    let host_with_port = &no_scheme[..host_end];
    let host = host_with_port.split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

/// 解析相对 Location 为绝对 URL
///
/// - 绝对 URL（http/https 开头）直接返回
/// - 以 `/` 开头的相对路径：拼接 scheme + host
/// - 其他相对路径：拼接 scheme + host + `/`
pub(super) fn resolve_url(base: &str, location: &str) -> Result<String, String> {
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

/// 截断字符串到指定长度（超出追加 "..."）
pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// TOML 字符串转义（处理反斜杠和引号）
///
/// 与 tunnel.rs 的 escape_toml_string 逻辑一致，此处为模板渲染独立保留
/// （tunnel.rs 的同名函数为私有，且本模块不在修改范围内）。
pub(super) fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
