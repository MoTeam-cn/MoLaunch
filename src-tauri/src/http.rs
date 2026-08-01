//! HTTP 客户端模块：统一管理 reqwest 客户端构建，支持代理 + IP 协议版本偏好 + TLS 信任源
//! 代理热更新：`init_client` 用 `RwLock<Option<Client>>`，`apply_config` 修改代理/IP 偏好后
//! 再次调用 `init_client` 重建客户端，无需重启应用即可生效。
//! TLS 信任源：`trust_mode` 控制 builtin/system/custom 三种根证书来源组合；
//! `ignore_tls=true`（开发者模式注册表 IgnoreTls）跳过所有证书校验，用于联机自签名证书调试。

use std::net::{IpAddr, Ipv4Addr};
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

/// 全局 HTTP 客户端（可热重建）
///
/// 启动时由 `lib.rs` 调用 `init_client` 写入；代理配置变更时由
/// `apply_config` 再次调用 `init_client` 覆盖。读取端通过 `get_client`
/// 拿到当前快照（`reqwest::Client` 内部 `Arc`，clone 廉价）。
static HTTP_CLIENT: RwLock<Option<reqwest::Client>> = RwLock::new(None);

/// 编译时生成的 User-Agent 字符串（缓存）
/// 格式：`Molaunch/{主版本}.{clientType}`（如 `Molaunch/1.0.0.10`）
/// 生成逻辑见 `utils::client_type`：平台码由编译目标推导，渠道码由版本后缀推导
static USER_AGENT: OnceLock<String> = OnceLock::new();

/// 获取 User-Agent 字符串（统一格式 `Molaunch/{主版本}.{clientType}`，见 utils::client_type）
fn user_agent() -> &'static str {
    USER_AGENT.get_or_init(crate::utils::client_type::user_agent)
}

/// 初始化或重建全局 HTTP 客户端
///
/// - 应用启动时调用一次（`lib.rs`）
/// - 代理/IP 版本/TLS 配置变更后再次调用（`apply_config` 副作用阶段）
///
/// 重复调用安全：直接覆盖旧客户端，进行中的请求仍使用旧客户端完成。
///
/// - `trust_mode`：信任源模式（builtin/system/custom/组合/all），见 `state::TlsConfig`
/// - `ignore_tls`：是否跳过证书校验（开发者模式注册表键，开启后 `trust_mode` 被忽略）
pub fn init_client(
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
    ip_version: &str,
    trust_mode: &str,
    ignore_tls: bool,
) {
    let client = build_client(
        proxy_mode,
        proxy_type,
        proxy_url,
        ip_version,
        Duration::from_secs(30),
        trust_mode,
        ignore_tls,
    );
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
    // 启用内置根证书确保 TLS 正常（与初始化路径默认 builtin 模式一致）
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(user_agent())
        .no_proxy()
        .tls_built_in_root_certs(true)
        .build()
        .expect("Failed to build default HTTP client")
}

/// 构建 HTTP 客户端
///
/// - `ip_version`：`"v4"` 强制 IPv4；`"auto"` 测试连通性自动选；`"any"` 跟随 DNS
/// - `trust_mode`（`ignore_tls=false` 时生效）：含 `builtin`/`system`/`custom` 或 `all`
///   分别启用 webpki-roots、OS 根证书、certs 目录自定义 PEM
/// - `ignore_tls=true`：跳过所有证书校验（`danger_accept_invalid_certs`）
pub fn build_client(
    proxy_mode: &str,
    proxy_type: &str,
    proxy_url: &str,
    ip_version: &str,
    timeout: Duration,
    trust_mode: &str,
    ignore_tls: bool,
) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(user_agent());

    // IP 协议版本偏好
    let local_addr = resolve_local_address(ip_version);
    if let Some(addr) = local_addr {
        builder = builder.local_address(addr);
    }

    // TLS 信任源配置
    if ignore_tls {
        // 开发者模式：跳过所有证书校验（仅用于自签名证书调试）
        builder = builder.danger_accept_invalid_certs(true);
    } else {
        // 解析信任源模式（支持组合，如 "system+custom"）
        let use_builtin = trust_mode.contains("builtin") || trust_mode == "all";
        let use_system = trust_mode.contains("system") || trust_mode == "all";
        let use_custom = trust_mode.contains("custom") || trust_mode == "all";

        // 关闭默认的内置根证书，改由下方精确控制
        // （reqwest rustls-tls 后端默认加载 webpki-roots，需显式关闭后按需开启）
        builder = builder.tls_built_in_root_certs(use_builtin);

        if use_system {
            for cert in crate::certs::load_system_root_certificates() {
                builder = builder.add_root_certificate(cert);
            }
        }
        if use_custom {
            for cert in crate::certs::load_custom_root_certificates() {
                builder = builder.add_root_certificate(cert);
            }
        }
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

/// 根据 `ip_version` 策略解析 `local_address`（客户端级）
///
/// - `"v4"`: 返回 `Some(Ipv4Addr::UNSPECIFIED)`（强制 IPv4）
/// - `"auto"`: 返回 `None` —— 不固定任何地址族。由 reqwest/hyper 底层的
///   **Happy Eyeballs** 机制对**目标域名**实时解析 A/AAAA 记录，并发尝试
///   连接并自动选择先连通的一方（v4/v6 均有则并发择优；单栈域名自然落到
///   对应地址族）。这比固定测 Cloudflare 更准确：域名可能只在某一地址族
///   可达，且 Cloudflare 连通 ≠ 目标域名连通
/// - `"any"` 或其他: 返回 `None`（不设置 `local_address`，跟随 DNS）
fn resolve_local_address(ip_version: &str) -> Option<IpAddr> {
    match ip_version {
        "v4" => Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        _ => None, // "auto"（Happy Eyeballs 按目标域名自动选优）/ "any" / 其他
    }
}

/// GET 请求并返回 (HTTP 状态码, 响应体文本)
///
/// 核心 GET 原语。与薄包装 [`fetch_url`] 不同，本函数保留状态码信息，
/// 便于调用方按状态码做差异化错误处理（如 yggdrasil 协议中 204 表示
/// validate 成功，403 表示 token 失效）。
///
/// 网络错误（无法连接服务器）时返回 Err；HTTP 任意状态码（含 4xx/5xx）均返回 Ok。
pub async fn get_text_with_status(url: &str) -> anyhow::Result<(u16, String)> {
    let client = get_client();
    let resp = client.get(url).send().await?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok((status, text))
}

/// GET 请求并返回响应体文本，HTTP 非 2xx 时返回 Err
///
/// [`get_text_with_status`] 的薄包装：多数调用方只需"成功拿文本，失败报错"。
pub async fn fetch_url(url: &str) -> anyhow::Result<String> {
    let (status, text) = get_text_with_status(url).await?;
    if !(200..300).contains(&status) {
        return Err(anyhow::anyhow!("HTTP error: {}", status));
    }
    Ok(text)
}

/// GET 请求并把响应体保存到文件
///
/// [`fetch_url`] 的薄包装：多一步写盘。成功返回响应体文本。
pub async fn fetch_url_to_file(url: &str, local_path: &std::path::Path) -> anyhow::Result<String> {
    let content = fetch_url(url).await?;
    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(local_path, &content)?;
    Ok(content)
}

/// POST JSON 请求并返回 (HTTP 状态码, 响应体文本)
///
/// 统一的 POST JSON 入口，自动设置 `Content-Type: application/json; charset=utf-8`
/// 和 `Accept-Language: zh-CN` 请求头。
/// 与 [`get_text_with_status`] 一样保留状态码信息，便于调用方按状态码做差异化错误处理。
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
