use super::super::binary::host_matches;

#[test]
fn host_matches_wildcard() {
    // 通配符：匹配任意子域名
    assert!(host_matches("jp-4.qwq.fan", "*.qwq.fan"));
    assert!(host_matches("us-1.qwq.fan", "*.qwq.fan"));
    // 通配符不匹配裸域名（需至少一层子域名）
    assert!(!host_matches("qwq.fan", "*.qwq.fan"));
    // 不匹配其他域名
    assert!(!host_matches("evil.example.com", "*.qwq.fan"));
}

#[test]
fn host_matches_exact() {
    assert!(host_matches(
        "frps.acme.example.com",
        "frps.acme.example.com"
    ));
    assert!(!host_matches(
        "other.acme.example.com",
        "frps.acme.example.com"
    ));
    // host:port 项比较时调用方先剥离端口，这里验证纯 host 匹配
    assert!(!host_matches(
        "frps.acme.example.com:7000",
        "frps.acme.example.com"
    ));
}

#[test]
fn host_matches_whitelist_forms() {
    // 模拟 validate_network_permissions 的白名单匹配逻辑：
    // 完整 host:port 匹配、host 匹配、通配符匹配三种形式
    let server_addr = "jp-4.qwq.fan";
    let addr_host = server_addr.split(':').next().unwrap_or(server_addr);
    let allowed = ["*.qwq.fan"];

    let matched = allowed.iter().any(|s| {
        let s = s.trim();
        if s == server_addr {
            return true;
        }
        let s_host = s.split(':').next().unwrap_or(s);
        s_host == addr_host || host_matches(addr_host, s_host)
    });
    assert!(matched);
}
