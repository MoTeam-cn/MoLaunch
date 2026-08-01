//! 深度链接安全校验
//!
//! 防止恶意网站通过 `molaunch://install?url=<恶意地址>` 诱导启动器下载任意 URL
//! （钓鱼 / 病毒下发）。校验规则：
//! 1. scheme 必须为 `https`
//! 2. 域名必须在 [`ALLOWED_DOWNLOAD_HOSTS`] 白名单内（含子域名）
//! 3. 禁止 URL 内嵌 userinfo（`user:pass@host`，常见迷惑手法）
//!
//! 白名单只收录可信任的整合包/Mod 下载源，新增来源需人工审核后加入。

use url::Url;

use crate::log_info;

/// 可信任的下载源域名白名单（支持子域名通配，如 `moiu.cn` 匹配 `*.moiu.cn`）
///
/// 收录原则：仅正式运营的整合包/Mod 分发渠道，域名需人工审核。
const ALLOWED_DOWNLOAD_HOSTS: &[&str] = &[
    // CurseForge 文件 CDN（forgecdn 系列，均为官方分发节点）
    "media.forgecdn.net",
    "edge.forgecdn.net",
    "mediafilez.forgecdn.net",
    // Modrinth 文件 CDN 与主站
    "cdn.modrinth.com",
    "modrinth.com",
    // MoLaunch 官方域名（api/download 等子域）
    "moiu.cn",
    "mocdn.net",
];

/// 校验一个下载 URL 是否可信任
///
/// - Ok(())：通过校验（https + 白名单域名 + 无 userinfo）
/// - Err(msg)：原因说明，供调用方日志/提示
pub fn validate_download_url(raw: &str) -> Result<(), String> {
    let url = Url::parse(raw).map_err(|_| format!("URL 非法: {}", raw))?;

    if url.scheme() != "https" {
        return Err(format!("仅允许 https 下载链接，收到: {}", url.scheme()));
    }

    if !url.username().is_empty() || url.password().is_some() {
        // username 非空 或 存在 password（即使为空串），均视为可疑的 userinfo 注入
        return Err("URL 包含 userinfo，已拦截（潜在欺骗）".to_string());
    }

    let Some(host) = url.host_str() else {
        return Err("URL 缺少域名".to_string());
    };

    if !is_host_allowed(host) {
        log_info!("[Deeplink] 拦截非白名单下载域名: {}", host);
        return Err(format!("域名 {} 不在可信任下载源白名单内", host));
    }

    Ok(())
}

/// 判断域名是否命中白名单（精确匹配或匹配 `.<白名单>` 子域后缀）
fn is_host_allowed(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    ALLOWED_DOWNLOAD_HOSTS.iter().any(|allowed| {
        let allowed = allowed.to_ascii_lowercase();
        host == allowed || host.ends_with(&format!(".{}", allowed))
    })
}

#[cfg(test)]
mod tests {
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
        assert!(
            validate_download_url("https://user:pass@media.forgecdn.net/x").is_err()
        );
    }

    #[test]
    fn rejects_malformed_url() {
        assert!(validate_download_url("not a url").is_err());
        assert!(validate_download_url("ftp://media.forgecdn.net/x").is_err());
    }
}
