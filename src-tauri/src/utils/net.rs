//! 网络地址判定工具
//!
//! 提供内网/回环地址判定，供 SSRF 防护等场景复用。

/// 判断地址是否为内网/回环地址
///
/// 支持 `host` 和 `host:port` 两种形式。非字面量 IP（域名）仅检查 `localhost`。
/// 覆盖范围：10.0.0.0/8、172.16.0.0/12、192.168.0.0/16（`Ipv4Addr::is_private`）、
/// 127.0.0.0/8（`Ipv4Addr::is_loopback`）。
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
            IpAddr::V4(v4) => v4.is_private() || v4.is_loopback(),
            IpAddr::V6(v6) => v6.is_loopback(),
        };
    }
    // 非字面量 IP（域名）：仅检查 localhost
    addr.eq_ignore_ascii_case("localhost")
}

#[cfg(test)]
mod tests {
    use super::is_private_address;

    #[test]
    fn test_is_private_address() {
        assert!(is_private_address("127.0.0.1"));
        assert!(is_private_address("127.0.0.1:8080"));
        assert!(is_private_address("192.168.1.1"));
        assert!(is_private_address("10.0.0.1"));
        assert!(is_private_address("localhost"));
        assert!(!is_private_address("example.com"));
        assert!(!is_private_address("8.8.8.8"));
        assert!(!is_private_address("1.1.1.1:53"));
    }
}
