//! 网络地址判定工具
//!
//! 提供内网/回环地址判定与 HTTP(S) 下载 URL 校验，供 SSRF 防护等场景复用。

use url::Url;

/// 判断地址是否为内网/回环/链路本地地址
///
/// 支持 `host` 和 `host:port` 两种形式。非字面量 IP（域名）仅检查 `localhost`。
/// 覆盖范围：10.0.0.0/8、172.16.0.0/12、192.168.0.0/16（`Ipv4Addr::is_private`）、
/// 127.0.0.0/8（`Ipv4Addr::is_loopback`）、169.254.0.0/16（链路本地）、
/// IPv6 回环/链路本地/唯一本地/未指定/映射地址。
pub fn is_private_address(addr: &str) -> bool {
    use std::net::{IpAddr, SocketAddr};
    // 优先按 SocketAddr 解析（处理 host:port），再按裸 IP 解析
    let ip = if let Ok(s) = addr.parse::<SocketAddr>() {
        Some(s.ip())
    } else {
        addr.parse::<IpAddr>().ok()
    };
    if let Some(ip) = ip {
        return match ip {
            IpAddr::V4(v4) => {
                v4.is_private()
                    || v4.is_loopback()
                    || v4.is_link_local()
                    || v4.is_unspecified()
                    || v4.is_broadcast()
            }
            IpAddr::V6(v6) => {
                v6.is_loopback()
                    || v6.is_unspecified()
                    || v6.is_unique_local()
                    || v6.is_unicast_link_local()
                    || v6.to_ipv4_mapped().is_some()
            }
        };
    }
    // 非字面量 IP（域名）：仅检查 localhost
    addr.eq_ignore_ascii_case("localhost")
}

/// 校验 HTTP(S) 下载 URL 是否安全（防 SSRF）
///
/// - 协议白名单：仅 http/https
/// - 禁止 userinfo 注入（潜在欺骗）
/// - 拒绝内网/回环/链路本地地址与 localhost（含 `*.localhost` 虚拟域名）
pub fn validate_public_http_url(raw: &str) -> Result<(), String> {
    let url = Url::parse(raw).map_err(|_| format!("URL 非法: {}", raw))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(format!(
            "仅允许 http/https 下载链接，收到: {}",
            url.scheme()
        ));
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err("URL 包含 userinfo，已拦截（潜在欺骗）".to_string());
    }

    let Some(host) = url.host_str() else {
        return Err("URL 缺少域名".to_string());
    };

    if is_private_address(host) || host.to_ascii_lowercase().ends_with(".localhost") {
        return Err(format!("拒绝内网/本地地址: {}", host));
    }

    Ok(())
}

/// updater 下载 URL 白名单域名（防 SSRF/防 manifest 篡改）
const UPDATER_ALLOWED_HOSTS: [&str; 2] = ["api.molaunch.moiu.cn", "download.mocdn.net"];

/// 校验 updater 下载 URL：协议 http/https + 域名白名单
///
/// 在 `validate_public_http_url` 基础上追加域名白名单限制，
/// 仅允许官方 API/CDN 域名，拒绝其他任何域名（含内网地址）。
pub fn validate_updater_download_url(raw: &str) -> Result<(), String> {
    validate_public_http_url(raw)?;
    let url = Url::parse(raw).map_err(|_| format!("URL 非法: {}", raw))?;
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if !UPDATER_ALLOWED_HOSTS.contains(&host.as_str()) {
        return Err(format!("更新下载域名不在白名单内: {}", host));
    }
    Ok(())
}

/// 脱敏 URL 用于日志：仅保留 scheme + host + path，去掉 query/fragment/userinfo
///
/// 防止 query 中的 token 等敏感参数进入日志；解析失败时原样返回（不丢信息）。
pub fn sanitize_url_for_log(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }
    match Url::parse(raw) {
        Ok(url) => {
            let mut out = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));
            if let Some(port) = url.port() {
                out.push_str(&format!(":{}", port));
            }
            out.push_str(url.path());
            out
        }
        Err(_) => raw.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_private_address, sanitize_url_for_log, validate_public_http_url};

    #[test]
    fn test_is_private_address() {
        assert!(is_private_address("127.0.0.1"));
        assert!(is_private_address("127.0.0.1:8080"));
        assert!(is_private_address("192.168.1.1"));
        assert!(is_private_address("10.0.0.1"));
        assert!(is_private_address("169.254.1.1"));
        assert!(is_private_address("0.0.0.0"));
        assert!(is_private_address("::1"));
        assert!(is_private_address("fe80::1"));
        assert!(is_private_address("fc00::1"));
        assert!(is_private_address("::ffff:127.0.0.1"));
        assert!(is_private_address("localhost"));
        assert!(!is_private_address("example.com"));
        assert!(!is_private_address("8.8.8.8"));
        assert!(!is_private_address("1.1.1.1:53"));
    }

    #[test]
    fn test_validate_public_http_url() {
        assert!(validate_public_http_url("https://textures.minecraft.net/abc.png").is_ok());
        assert!(validate_public_http_url("http://example.com/a.png").is_ok());
        assert!(validate_public_http_url("file:///etc/passwd").is_err());
        assert!(validate_public_http_url("ftp://example.com/a").is_err());
        assert!(validate_public_http_url("https://user:pass@example.com/a").is_err());
        assert!(validate_public_http_url("https://127.0.0.1/a").is_err());
        assert!(validate_public_http_url("https://192.168.1.1/a").is_err());
        assert!(validate_public_http_url("https://localhost/a").is_err());
        assert!(validate_public_http_url("https://cache-image.localhost/a.png").is_err());
        assert!(validate_public_http_url("https://169.254.1.1/a").is_err());
    }

    #[test]
    fn test_sanitize_url_for_log() {
        assert_eq!(
            sanitize_url_for_log("https://api.example.com/v1/manifest?token=secret&a=1#frag"),
            "https://api.example.com/v1/manifest"
        );
        assert_eq!(
            sanitize_url_for_log("https://user:pass@example.com:8443/a/b.png?t=1"),
            "https://example.com:8443/a/b.png"
        );
        assert_eq!(
            sanitize_url_for_log("https://example.com/"),
            "https://example.com/"
        );
        assert_eq!(sanitize_url_for_log(""), "");
        // 解析失败原样返回
        assert_eq!(sanitize_url_for_log("not a url"), "not a url");
    }
}
