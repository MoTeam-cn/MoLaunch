use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use super::tls;

/// HTTP 客户端构建参数（含可选重定向策略）。
pub struct ClientBuildParams<'a> {
    pub proxy_mode: &'a str,
    pub proxy_type: &'a str,
    pub proxy_url: &'a str,
    pub ip_version: &'a str,
    pub timeout: Duration,
    pub trust_mode: &'a str,
    pub ignore_tls: bool,
    pub redirect: Option<reqwest::redirect::Policy>,
    pub user_agent: Option<&'a str>,
    pub no_timeout: bool,
}

/// 构建流式专用 HTTP 客户端（无客户端级整体超时）。
pub fn build_stream_client(
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
    ip_version: &str,
    trust_mode: &str,
    ignore_tls: bool,
) -> reqwest::Client {
    build_client_inner(ClientBuildParams {
        proxy_mode,
        proxy_type,
        proxy_url,
        ip_version,
        timeout: Duration::from_secs(0),
        trust_mode,
        ignore_tls,
        redirect: None,
        user_agent: None,
        no_timeout: true,
    })
}

/// 构建 HTTP 客户端。
pub fn build_client(
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
    ip_version: &str,
    timeout: Duration,
    trust_mode: &str,
    ignore_tls: bool,
) -> reqwest::Client {
    build_client_inner(ClientBuildParams {
        proxy_mode,
        proxy_type,
        proxy_url,
        ip_version,
        timeout,
        trust_mode,
        ignore_tls,
        redirect: None,
        user_agent: None,
        no_timeout: false,
    })
}

/// 基于当前配置构建带自定义重定向策略的 HTTP 客户端。
pub fn build_client_with_redirect(
    redirect: reqwest::redirect::Policy,
    timeout_ms: Option<u64>,
) -> reqwest::Client {
    let (mode, kind, url, ip_version, trust_mode) = current_http_config();
    build_client_inner(ClientBuildParams {
        proxy_mode: &mode,
        proxy_type: &kind,
        proxy_url: &url,
        ip_version: &ip_version,
        timeout: timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_secs(30)),
        trust_mode: &trust_mode,
        ignore_tls: crate::commands::system::developer::is_ignore_tls(),
        redirect: Some(redirect),
        user_agent: None,
        no_timeout: false,
    })
}

/// 构建禁用自动重定向的 HTTP 客户端（与主客户端同配置，仅重定向策略不同）。
pub fn build_no_redirect_client(
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
    ip_version: &str,
    trust_mode: &str,
    ignore_tls: bool,
) -> reqwest::Client {
    build_client_inner(ClientBuildParams {
        proxy_mode,
        proxy_type,
        proxy_url,
        ip_version,
        timeout: Duration::from_secs(30),
        trust_mode,
        ignore_tls,
        redirect: Some(reqwest::redirect::Policy::none()),
        user_agent: None,
        no_timeout: false,
    })
}

/// 基于当前配置构建带自定义 User-Agent 的 HTTP 客户端。
pub fn build_client_with_user_agent(user_agent: &str, timeout_ms: Option<u64>) -> reqwest::Client {
    let (mode, kind, url, ip_version, trust_mode) = current_http_config();
    build_client_inner(ClientBuildParams {
        proxy_mode: &mode,
        proxy_type: &kind,
        proxy_url: &url,
        ip_version: &ip_version,
        timeout: timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_secs(30)),
        trust_mode: &trust_mode,
        ignore_tls: crate::commands::system::developer::is_ignore_tls(),
        redirect: None,
        user_agent: Some(user_agent),
        no_timeout: false,
    })
}

fn current_http_config() -> (String, String, String, String, String) {
    crate::config::load_config()
        .ok()
        .flatten()
        .map(|config| {
            (
                config.proxy.mode,
                config.proxy.kind,
                config.proxy.url,
                config.proxy.ip_version,
                config.tls.trust_mode,
            )
        })
        .unwrap_or_else(|| {
            (
                "none".to_string(),
                "http".to_string(),
                String::new(),
                "auto".to_string(),
                "builtin".to_string(),
            )
        })
}

fn build_client_inner(params: ClientBuildParams<'_>) -> reqwest::Client {
    let user_agent = params
        .user_agent
        .map(str::to_string)
        .unwrap_or_else(|| super::user_agent().to_string());
    let mut builder = reqwest::Client::builder().user_agent(user_agent);
    if !params.no_timeout {
        builder = builder.timeout(params.timeout);
    }
    if let Some(policy) = params.redirect {
        builder = builder.redirect(policy);
    }

    if let Some(addr) = resolve_local_address(params.ip_version) {
        builder = builder.local_address(addr);
    }
    // 通用客户端不绑定 base_url，无法低成本获知目标 host，保守禁用 IgnoreTls
    builder = tls::configure(builder, params.trust_mode, None);
    builder = configure_proxy(
        builder,
        params.proxy_mode,
        params.proxy_type,
        params.proxy_url,
    );
    builder.build().expect("Failed to build HTTP client")
}

fn configure_proxy(
    mut builder: reqwest::ClientBuilder,
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
) -> reqwest::ClientBuilder {
    match proxy_mode {
        "system" => builder,
        "custom" => {
            if !proxy_url.is_empty() {
                let full_url = match proxy_type {
                    "socks5" => format!("socks5://{}", proxy_url),
                    "https" => format!("https://{}", proxy_url),
                    _ => format!("http://{}", proxy_url),
                };
                if let Ok(proxy) = reqwest::Proxy::all(&full_url) {
                    builder = builder.proxy(proxy);
                }
            }
            builder
        }
        _ => builder.no_proxy(),
    }
}

fn resolve_local_address(ip_version: &str) -> Option<IpAddr> {
    match ip_version {
        "v4" => Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        _ => None,
    }
}
