//! 配置补丁校验
//!
//! - `validate_mirror_url`：镜像 URL SSRF 防护（拒绝环回 / 链路本地 / 私有地址）
//! - `validate_patch`：整合 mirror_url + download_source + meta_source 校验，
//!   供 `apply_config_inner` 在字段更新前调用

use super::types::ConfigPatch;

/// 校验镜像 URL：拒绝环回、链路本地、私有网络地址（SSRF 防护）
pub fn validate_mirror_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("镜像 URL 必须以 http:// 或 https:// 开头".to_string());
    }
    let after_scheme = if let Some(rest) = url.strip_prefix("https://") {
        rest
    } else {
        url.strip_prefix("http://").unwrap_or(url)
    };
    let host_part = after_scheme.split('@').last().unwrap_or(after_scheme);
    let host_end = host_part
        .find(|c| c == '/' || c == ':' || c == '?' || c == '#')
        .unwrap_or(host_part.len());
    let host = &host_part[..host_end];
    let host = host.trim_start_matches('[').trim_end_matches(']');

    if host.is_empty() {
        return Err("镜像 URL 主机不能为空".to_string());
    }
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return Err("镜像 URL 不能指向环回地址".to_string());
    }
    if host.starts_with("169.254.") {
        return Err("镜像 URL 不能指向链路本地地址".to_string());
    }
    if host.starts_with("10.") || host.starts_with("192.168.") {
        return Err("镜像 URL 不能指向私有网络地址".to_string());
    }
    if host.starts_with("172.") {
        if let Some(second) = host.split('.').nth(1) {
            if let Ok(n) = second.parse::<u32>() {
                if (16..=31).contains(&n) {
                    return Err("镜像 URL 不能指向私有网络地址".to_string());
                }
            }
        }
    }
    Ok(())
}

/// 校验配置补丁：mirror_url SSRF、download_source / meta_source 枚举
pub fn validate_patch(patch: &ConfigPatch) -> Result<(), String> {
    if let Some(Some(ref url)) = patch.mirror_url {
        validate_mirror_url(url)?;
    }
    if let Some(ref s) = patch.download_source {
        if !matches!(s.as_str(), "official" | "mirror" | "smart") {
            return Err(format!("无效的 download_source: {}", s));
        }
    }
    if let Some(ref s) = patch.meta_source {
        if !matches!(s.as_str(), "official" | "mirror" | "smart") {
            return Err(format!("无效的 meta_source: {}", s));
        }
    }
    Ok(())
}
