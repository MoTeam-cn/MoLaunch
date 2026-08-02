//! 深度链接安全校验
//!
//! 校验下载 URL 是否为可信 https 来源（白名单域名 + 禁止 userinfo 注入），
//! 防止恶意网页诱导启动器下载任意地址。

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
#[path = "security_tests.rs"]
mod tests;
