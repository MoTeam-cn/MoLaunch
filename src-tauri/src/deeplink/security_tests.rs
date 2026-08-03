//! security 单元测试

use super::*;

#[test]
fn allows_whitelisted_https() {
    assert!(validate_download_url("https://media.forgecdn.net/files/123/456/x.jar").is_ok());
    assert!(validate_download_url("https://edge.forgecdn.net/files/123/456/x.jar").is_ok());
    assert!(validate_download_url("https://mediafilez.forgecdn.net/files/123/456/x.jar").is_ok());
    assert!(validate_download_url("https://cdn.modrinth.com/data/abc.zip").is_ok());
    assert!(validate_download_url("https://modrinth.com/modpack/xyz").is_ok());
    // 子域名通配
    assert!(validate_download_url("https://download.moiu.cn/packs/x.zip").is_ok());
    assert!(validate_download_url("https://api.molaunch.moiu.cn/x").is_ok());
}

#[test]
fn rejects_non_whitelisted_hosts() {
    assert!(validate_download_url("https://evil.com/virus.exe").is_err());
    // 白名单域名的伪造后缀（mocdn.net 与 evilmocdn.net 需区分）
    assert!(validate_download_url("https://evilmocdn.net/x").is_err());
}

#[test]
fn rejects_http_and_userinfo() {
    assert!(validate_download_url("http://media.forgecdn.net/x").is_err());
    assert!(validate_download_url("https://user:pass@media.forgecdn.net/x").is_err());
}

#[test]
fn rejects_malformed_url() {
    assert!(validate_download_url("not a url").is_err());
    assert!(validate_download_url("ftp://media.forgecdn.net/x").is_err());
}
