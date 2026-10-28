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
    /// 是否应用下载专用调优（HTTP/2 大窗口、连接池）
    pub h2_tuned: bool,
    /// 是否挂载带缓存的 DNS 解析器（仅下载客户端）
    pub cache_dns: bool,
    /// 把某个 host 钉到指定地址（边缘节点优选用；不影响其它 host）
    pub dns_pin: Option<(&'a str, std::net::SocketAddr)>,
}

/// HTTP/2 单流初始窗口（1 MiB）
///
/// 默认 64 KiB 在千兆链路上要几十个 RTT 才能爬满带宽；大文件下载的单连接
/// 吞吐几乎完全取决于这个窗口。
const DOWNLOAD_STREAM_WINDOW: u32 = 1024 * 1024;

/// HTTP/2 连接级初始窗口（64 MiB）：多条并发流共享，避免互相抢窗口
const DOWNLOAD_CONN_WINDOW: u32 = 64 * 1024 * 1024;

/// 每个 host 保留的空闲连接数：换源/重试时复用连接，省掉重复握手
const DOWNLOAD_POOL_IDLE: usize = 8;

/// 构建下载专用 HTTP 客户端：无整体超时 + HTTP/2 调优 + DNS 缓存。
///
/// 与全局客户端分开，是为了让下载路径的调优（大窗口、连接池、DNS 缓存）
/// 不影响 API/鉴权这类小请求的连接策略。
pub fn build_download_client(
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
        h2_tuned: true,
        cache_dns: true,
        dns_pin: None,
    })
}

/// 构建把指定 host 钉到指定地址的下载客户端（边缘节点优选专用）
///
/// 证书校验、Host 头、SNI 都仍是原域名，只把连接地址换掉——直接请求 IP 会因为
/// 证书与 Host 不匹配而失败，这也是优选能安全工作的前提。
pub fn build_pinned_client(host: &str, addr: std::net::SocketAddr) -> Option<reqwest::Client> {
    let (mode, kind, url, ip_version, trust_mode) = current_http_config();
    try_build_client_inner(ClientBuildParams {
        proxy_mode: &mode,
        proxy_type: &kind,
        proxy_url: &url,
        ip_version: &ip_version,
        timeout: Duration::from_secs(0),
        trust_mode: &trust_mode,
        ignore_tls: crate::commands::system::developer::is_ignore_tls(),
        redirect: None,
        user_agent: None,
        no_timeout: true,
        h2_tuned: true,
        cache_dns: false,
        dns_pin: Some((host, addr)),
    })
}

/// 是否配置了代理
///
/// 走代理时换 IP 没有意义（流量全进代理隧道），边缘优选必须整段跳过。
pub fn proxy_configured() -> bool {
    let (mode, _, _, _, _) = current_http_config();
    mode == "custom" || mode == "system"
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
        h2_tuned: false,
        cache_dns: false,
        dns_pin: None,
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
        h2_tuned: false,
        cache_dns: false,
        dns_pin: None,
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
        h2_tuned: false,
        cache_dns: false,
        dns_pin: None,
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
        h2_tuned: false,
        cache_dns: false,
        dns_pin: None,
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
        h2_tuned: false,
        cache_dns: false,
        dns_pin: None,
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
    try_build_client_inner(params).expect("Failed to build HTTP client")
}

/// 构建失败返回 None（调用方自行决定降级，不 panic）
fn try_build_client_inner(params: ClientBuildParams<'_>) -> Option<reqwest::Client> {
    let user_agent = params
        .user_agent
        .map(str::to_string)
        .unwrap_or_else(|| super::user_agent().to_string());
    let mut builder = reqwest::Client::builder().user_agent(user_agent);
    if !params.no_timeout {
        builder = builder.timeout(params.timeout);
    }
    // 未显式指定策略时用「逐跳校验」策略：保留跟随重定向，但每一跳目标必须是公网，
    // 防止校验过的公网 URL 经开放重定向落到内网（utils/net.rs::safe_redirect_policy）
    builder = builder.redirect(
        params
            .redirect
            .unwrap_or_else(crate::utils::net::safe_redirect_policy),
    );

    if let Some(addr) = resolve_local_address(params.ip_version) {
        builder = builder.local_address(addr);
    }
    if params.h2_tuned {
        builder = apply_download_tuning(builder);
    }
    if let Some((host, addr)) = params.dns_pin {
        builder = builder.resolve_to_addrs(host, &[addr]);
    }
    if params.cache_dns {
        builder = builder.dns_resolver(std::sync::Arc::new(super::dns::CacheResolver::new()));
    }
    // 通用客户端不绑定 base_url，无法低成本获知目标 host，保守禁用 IgnoreTls
    builder = tls::configure(builder, params.trust_mode, None);
    builder = configure_proxy(
        builder,
        params.proxy_mode,
        params.proxy_type,
        params.proxy_url,
    );
    builder.build().ok()
}

/// 下载专用传输层调优
fn apply_download_tuning(builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    builder
        .http2_adaptive_window(true)
        .http2_initial_stream_window_size(DOWNLOAD_STREAM_WINDOW)
        .http2_initial_connection_window_size(DOWNLOAD_CONN_WINDOW)
        .pool_max_idle_per_host(DOWNLOAD_POOL_IDLE)
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
