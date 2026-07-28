//! HTTP 客户端模块
//! 统一管理 reqwest 客户端构建，支持代理配置 + IP 协议版本偏好
//!
//! 代理热更新：`init_client` 使用 `RwLock<Option<Client>>` 而非 `OnceLock`，
//! 用户在设置页修改代理或 IP 版本偏好后 `apply_config` 会再次调用 `init_client` 重建客户端，
//! 无需重启应用即可生效。

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

/// 全局 HTTP 客户端（可热重建）
///
/// 启动时由 `lib.rs` 调用 `init_client` 写入；代理配置变更时由
/// `apply_config` 再次调用 `init_client` 覆盖。读取端通过 `get_client`
/// 拿到当前快照（`reqwest::Client` 内部 `Arc`，clone 廉价）。
static HTTP_CLIENT: RwLock<Option<reqwest::Client>> = RwLock::new(None);

/// 编译时生成的 User-Agent 字符串
/// 格式：`MoLaunch/<os> <version>`（如 `MoLaunch/windows 0.1.0`）
/// - `<os>` 来自 `std::env::consts::OS`（编译期不可用，运行时取值）
/// - `<version>` 来自 Cargo.toml 的 `version` 字段（编译期通过 `env!` 注入）
static USER_AGENT: OnceLock<String> = OnceLock::new();

/// 获取 User-Agent 字符串（运行时拼接 OS + 编译时版本号）
fn user_agent() -> &'static str {
    USER_AGENT.get_or_init(|| {
        format!(
            "MoLaunch/{} {}",
            std::env::consts::OS,
            env!("CARGO_PKG_VERSION")
        )
    })
}

/// 初始化或重建全局 HTTP 客户端
///
/// - 应用启动时调用一次（`lib.rs`）
/// - 代理/IP 版本配置变更后再次调用（`apply_config` 副作用阶段）
///
/// 重复调用安全：直接覆盖旧客户端，进行中的请求仍使用旧客户端完成。
pub fn init_client(proxy_mode: &str, proxy_type: &str, proxy_url: &str, ip_version: &str) {
    let client = build_client(proxy_mode, proxy_type, proxy_url, ip_version, Duration::from_secs(30));
    let mut guard = HTTP_CLIENT.write().expect("HTTP_CLIENT poisoned");
    *guard = Some(client);
}

/// 获取全局 HTTP 客户端
/// 如果未初始化，返回一个无代理的默认客户端
pub fn get_client() -> reqwest::Client {
    {
        let guard = HTTP_CLIENT.read().expect("HTTP_CLIENT poisoned");
        if let Some(ref client) = *guard {
            return client.clone();
        }
    }
    // 未初始化兜底（理论上 lib.rs 启动时已 init，此处防御性处理）
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(user_agent())
        .no_proxy()
        .build()
        .expect("Failed to build default HTTP client")
}

/// 构建 HTTP 客户端
///
/// `ip_version` 控制 IP 协议偏好：
/// - `"v4"`: 强制 IPv4（`local_address = 0.0.0.0`，reqwest 仅解析 A 记录）
/// - `"auto"`: 自动选择（测试 v4/v6 连通性，选稳定的那个）
/// - `"any"` 或其他: 随意解析（不设置 `local_address`，跟随 DNS 服务器）
pub fn build_client(
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
    ip_version: &str,
    timeout: Duration,
) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(user_agent());

    // IP 协议版本偏好
    let local_addr = resolve_local_address(ip_version);
    if let Some(addr) = local_addr {
        builder = builder.local_address(addr);
    }

    match proxy_mode {
        "system" => {
            // reqwest 默认使用系统代理，无需额外配置
        }
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
        }
        _ => {
            // "none" - 不使用代理
            builder = builder.no_proxy();
        }
    }

    builder.build().expect("Failed to build HTTP client")
}

/// 根据 `ip_version` 策略解析 `local_address`
///
/// - `"v4"`: 返回 `Some(Ipv4Addr::UNSPECIFIED)`（强制 IPv4）
/// - `"auto"`: 测试 v4/v6 连通性，选稳定的那个（均不可达时返回 None 兜底）
/// - `"any"` 或其他: 返回 `None`（不设置 `local_address`，跟随 DNS）
fn resolve_local_address(ip_version: &str) -> Option<IpAddr> {
    match ip_version {
        "v4" => Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        "auto" => auto_detect_ip_version(),
        _ => None, // "any" 或其他：随意解析
    }
}

/// 自动检测 IPv4/IPv6 连通性，返回更稳定的协议版本
///
/// 通过 TCP 连接 Cloudflare DNS（v4: 1.1.1.1:443 / v6: [2606:4700:4700::1111]:443）
/// 测试连通性和延迟，选择更优的一方：
/// - 仅 v4 可达 → 返回 `Some(Ipv4)`
/// - 仅 v6 可达 → 返回 `Some(Ipv6)`
/// - 均可达且延迟接近（差异 < 50ms）→ 返回 `None`（让 OS 决定）
/// - 均可达且一方明显更快 → 返回更快的一方
/// - 均不可达 → 返回 `None`（兜底，让 OS 尝试）
fn auto_detect_ip_version() -> Option<IpAddr> {
    let v4_target: std::net::SocketAddr = "1.1.1.1:443".parse().ok()?;
    let v6_target: std::net::SocketAddr = "[2606:4700:4700::1111]:443".parse().ok()?;

    let v4_time = test_tcp_connect(v4_target);
    let v6_time = test_tcp_connect(v6_target);

    match (v4_time, v6_time) {
        (Some(v4), Some(v6)) => {
            // 均可达：延迟差异 < 50ms 视为接近，让 OS 决定
            let diff = if v4 > v6 { v4 - v6 } else { v6 - v4 };
            if diff < 50 {
                None
            } else if v4 < v6 {
                Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
            } else {
                Some(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
            }
        }
        (Some(_), None) => Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        (None, Some(_)) => Some(IpAddr::V6(Ipv6Addr::UNSPECIFIED)),
        (None, None) => None,
    }
}

/// 测试 TCP 连接连通性，返回连接延迟（毫秒）
fn test_tcp_connect(target: std::net::SocketAddr) -> Option<u128> {
    let start = Instant::now();
    match std::net::TcpStream::connect_timeout(&target, Duration::from_secs(2)) {
        Ok(_) => Some(start.elapsed().as_millis()),
        Err(_) => None,
    }
}

/// 获取 URL 内容（使用全局客户端）
pub async fn fetch_url(url: &str) -> anyhow::Result<String> {
    let client = get_client();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    Ok(response.text().await?)
}

/// 获取 URL 内容并保存到文件
pub async fn fetch_url_to_file(url: &str, local_path: &std::path::Path) -> anyhow::Result<String> {
    let client = get_client();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    let content = response.text().await?;

    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(local_path, &content)?;
    Ok(content)
}

/// GET 请求并返回 (HTTP 状态码, 响应体文本)
///
/// 与 `fetch_url` 不同，本函数保留状态码信息，便于调用方按状态码做差异化错误处理
/// （如 yggdrasil 协议中 204 表示 validate 成功，403 表示 token 失效）。
///
/// 网络错误（无法连接服务器）时返回 Err；HTTP 任意状态码（含 4xx/5xx）均返回 Ok。
pub async fn get_text_with_status(url: &str) -> anyhow::Result<(u16, String)> {
    let client = get_client();
    let resp = client.get(url).send().await?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok((status, text))
}

/// POST JSON 请求并返回 (HTTP 状态码, 响应体文本)
///
/// 统一的 POST JSON 入口，自动设置 `Content-Type: application/json; charset=utf-8`
/// 和 `Accept-Language: zh-CN` 请求头。
/// 与 `fetch_url` 不同，本函数保留状态码信息，便于调用方按状态码做差异化错误处理。
///
/// 网络错误（无法连接服务器）时返回 Err；HTTP 任意状态码（含 4xx/5xx）均返回 Ok。
pub async fn post_json_with_status<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> anyhow::Result<(u16, String)> {
    let client = get_client();
    let resp = client
        .post(url)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Accept-Language", "zh-CN")
        .json(body)
        .send()
        .await?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok((status, text))
}

/// GET 请求并返回二进制内容
///
/// 用于下载 jar/图片等二进制资源。HTTP 非 2xx 返回 Err。
pub async fn fetch_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let client = get_client();
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", resp.status()));
    }
    Ok(resp.bytes().await?.to_vec())
}
