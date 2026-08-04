//! frpc TOML 配置生成工具（frp 0.x 原版格式）
//!
//! 参考原版 frp 配置（如 Lolia 等厂商 config 接口返回格式）：
//!
//! ```toml
//! serverAddr = 'hk-6.qwq.fan'
//! serverPort = 17000
//! user = '60'
//!
//! [auth]
//! token = 'va3xljq0469rzujuwzapt1fdmkoiiu32'
//!
//! [[proxies]]
//! name = 'deffb45553f74606b2380db8b868facf'
//! type = 'tcp'
//! localIP = '127.0.0.1'
//! localPort = 3000
//! remotePort = 30919
//!
//! [proxies.transport]
//! bandwidthLimit = '4MB'
//! bandwidthLimitMode = 'server'
//! ```
//!
//! 启动器本地自建隧道 / 无 config 端点时用本工具生成同构配置；
//! 有 config 端点时优先使用厂商返回的原版配置（见 `executor` 拉取链路），
//! 本工具负责为原版配置叠加逆向防封字段（metadatas/transport 等）。

/// frpc 全局连接配置（顶层字段）
#[derive(Debug, Clone, Default)]
pub struct ServerConn {
    pub server_addr: String,
    pub server_port: u16,
    /// 登录用户名（`user` 顶层字段，frp 中标识账户归属）
    pub user: Option<String>,
    /// 鉴权 token（放 `[auth] token`）
    pub token: Option<String>,
    /// 是否启用 TLS（`transport.tls.enable`）
    pub use_tls: bool,
}

/// 单个代理（隧道）配置
#[derive(Debug, Clone)]
pub struct Proxy {
    pub name: String,
    pub proxy_type: String,
    pub local_ip: String,
    pub local_port: u16,
    pub remote_port: u16,
    /// 自定义域名（http/https 类型）
    pub custom_domains: Option<String>,
    /// 带宽限制（如 "4MB"），映射 `[proxies.transport] bandwidthLimit`
    pub bandwidth_limit: Option<String>,
    /// 带宽限制模式（如 "server"），映射 `[proxies.transport] bandwidthLimitMode`
    pub bandwidth_limit_mode: Option<String>,
    /// Proxy 传输加密
    pub use_encryption: Option<bool>,
    /// Proxy 传输压缩
    pub use_compression: Option<bool>,
    /// Proxy 协议版本
    pub protocol_version: Option<String>,
}

/// 生成 frpc TOML 配置（frp 0.x 统一格式）
///
/// 参考原版 frpc 配置格式：
/// 顶层 `serverAddr/serverPort/user`、`[auth] token`、`[[proxies]]`。
pub fn build_frpc_toml(conn: &ServerConn, proxies: &[Proxy]) -> String {
    let mut out = String::new();

    // 全局连接配置
    out.push_str(&format!(
        "serverAddr = '{}'\n",
        toml_escape(&conn.server_addr)
    ));
    out.push_str(&format!("serverPort = {}\n", conn.server_port));
    if let Some(user) = conn.user.as_deref() {
        if !user.is_empty() {
            out.push_str(&format!("user = '{}'\n", toml_escape(user)));
        }
    }
    if conn.use_tls {
        out.push_str("transport.tls.enable = true\n");
    }
    out.push('\n');

    // 鉴权 token：`[auth] token`
    if let Some(token) = conn.token.as_deref() {
        if !token.is_empty() {
            out.push_str("[auth]\n");
            out.push_str(&format!("token = '{}'\n", toml_escape(token)));
            out.push('\n');
        }
    }

    // 代理段
    for p in proxies {
        out.push_str("[[proxies]]\n");
        out.push_str(&format!("name = '{}'\n", toml_escape(&p.name)));
        out.push_str(&format!("type = '{}'\n", toml_escape(&p.proxy_type)));
        out.push_str(&format!("localIP = '{}'\n", toml_escape(&p.local_ip)));
        out.push_str(&format!("localPort = {}\n", p.local_port));
        if p.remote_port != 0 {
            out.push_str(&format!("remotePort = {}\n", p.remote_port));
        }
        if let Some(domains) = p.custom_domains.as_deref() {
            if !domains.is_empty() {
                out.push_str(&format!("customDomains = '{}'\n", toml_escape(domains)));
            }
        }
        // transport 子表（带宽限制等）
        if p.bandwidth_limit.is_some() || p.bandwidth_limit_mode.is_some() {
            out.push_str("\n[proxies.transport]\n");
            if let Some(bl) = p.bandwidth_limit.as_deref() {
                if !bl.is_empty() {
                    out.push_str(&format!("bandwidthLimit = '{}'\n", toml_escape(bl)));
                }
            }
            if let Some(blm) = p.bandwidth_limit_mode.as_deref() {
                if !blm.is_empty() {
                    out.push_str(&format!("bandwidthLimitMode = '{}'\n", toml_escape(blm)));
                }
            }
            if let Some(value) = p.use_encryption {
                out.push_str(&format!("useEncryption = {}\n", value));
            }
            if let Some(value) = p.use_compression {
                out.push_str(&format!("useCompression = {}\n", value));
            }
            if let Some(value) = p.protocol_version.as_deref() {
                if !value.is_empty() {
                    out.push_str(&format!("protocolVersion = '{}'\n", toml_escape(value)));
                }
            }
        }
        out.push('\n');
    }

    out
}

/// 为厂商返回的原版配置叠加逆向字段
///
/// 厂商 config 接口返回的配置（如 Lolia 的 base64 解码结果）已包含
/// serverAddr/serverPort/user/metadatas/proxies 等完整字段，但部分逆向
/// 字段（如 `[metadatas] token`、`[proxies.transport]` 带宽限制）可能缺失。
/// 本函数在保留原配置全部内容的前提下追加缺失的逆向字段：
///
/// 1. `[metadatas] token`：本地隧道 token 优先（厂商配置一般已含，不重复写）
/// 2. `[[proxies]]` 的 `[proxies.transport] bandwidthLimit / bandwidthLimitMode`：
///    Lolia 等厂商限速需在客户端声明，缺失时按 `bandwidthLimit`/`bandwidthLimitMode`
///    参数叠加（防止厂商服务端默认限速导致带宽异常）。
///
/// 由于 TOML 数组表需在原表后追加子表，本函数采用**文本追加**方式：
/// 在配置末尾追加缺失的 `[proxies.transport]` 子表（TOML 允许对已存在的
/// `[[proxies]]` 元素追加子表，frp 解析时合并）。若配置中已含 transport 子表
/// 则跳过，避免重复定义。
pub fn overlay_extra_fields(
    raw_config: &str,
    bandwidth_limit: Option<&str>,
    bandwidth_limit_mode: Option<&str>,
) -> String {
    // 原配置直接保留
    let mut out = raw_config.trim_end().to_string();

    // 需要叠加的字段
    let mut overlay = String::new();
    if let Some(bl) = bandwidth_limit {
        if !bl.is_empty() {
            overlay.push_str(&format!("bandwidthLimit = '{}'\n", toml_escape(bl)));
        }
    }
    if let Some(blm) = bandwidth_limit_mode {
        if !blm.is_empty() {
            overlay.push_str(&format!("bandwidthLimitMode = '{}'\n", toml_escape(blm)));
        }
    }

    if overlay.is_empty() {
        return out;
    }

    // 若已存在 proxies.transport 子表则不再叠加（避免 TOML 重复键冲突）
    let lower = raw_config.to_lowercase();
    if lower.contains("[proxies.transport]") {
        return out;
    }

    // 追加 transport 子表。TOML 中 `[proxies.transport]` 作用于最后一个 [[proxies]]，
    // 直接追加在文件末尾即可。
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("\n[proxies.transport]\n");
    out.push_str(&overlay);
    out
}

/// TOML 字符串转义（单引号字面量字符串：只转义单引号）
fn toml_escape(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
#[path = "frpc_config_tests.rs"]
mod tests;
