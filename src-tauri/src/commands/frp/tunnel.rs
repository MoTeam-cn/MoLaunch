//! 隧道管理：CRUD + frpc TOML 配置生成
//!
//! 隧道配置持久化到 `<base_dir>/frp/tunnels.json`。
//! 启动隧道时生成 TOML 配置文件到 `<base_dir>/frp/config/<tunnel_id>.toml`。

use super::{ensure_dir, frp_config_dir, tunnels_path, Tunnel, TunnelType};
use crate::log_info;
use std::time::{SystemTime, UNIX_EPOCH};

/// 列出所有隧道
pub async fn list_tunnels() -> Result<Vec<Tunnel>, String> {
    let tunnels = read_tunnels()?;
    Ok(tunnels)
}

/// 创建隧道
pub async fn create_tunnel(params: CreateTunnelParams) -> Result<Tunnel, String> {
    let mut tunnels = read_tunnels()?;

    // 校验名称不重复
    if tunnels.iter().any(|t| t.name == params.name) {
        return Err(format!("隧道名称已存在: {}", params.name));
    }

    let tunnel = Tunnel {
        id: generate_id(),
        name: params.name,
        provider_id: params.provider_id,
        tunnel_type: params.tunnel_type,
        local_ip: params.local_ip.unwrap_or_else(|| "127.0.0.1".to_string()),
        local_port: params.local_port,
        server_addr: params.server_addr,
        server_port: params.server_port,
        remote_port: params.remote_port,
        token: params.token,
        use_tls: params.use_tls.unwrap_or(false),
        created_at: now_ms(),
    };

    tunnels.push(tunnel.clone());
    write_tunnels(&tunnels)?;
    log_info!("[Frp] 隧道已创建: {} ({})", tunnel.name, tunnel.id);
    Ok(tunnel)
}

/// 删除隧道
pub async fn delete_tunnel(id: String) -> Result<(), String> {
    let mut tunnels = read_tunnels()?;
    let before = tunnels.len();
    tunnels.retain(|t| t.id != id);
    if tunnels.len() == before {
        return Err(format!("隧道不存在: {}", id));
    }
    write_tunnels(&tunnels)?;

    // 清理配置文件
    let config_path = frp_config_dir().join(format!("{}.toml", id));
    if config_path.exists() {
        std::fs::remove_file(&config_path).ok();
    }

    log_info!("[Frp] 隧道已删除: {}", id);
    Ok(())
}

/// 生成 frpc TOML 配置文件
///
/// 写入 `<base_dir>/frp/config/<tunnel_id>.toml`，返回文件路径。
pub fn generate_config(tunnel: &Tunnel) -> Result<std::path::PathBuf, String> {
    let config_dir = frp_config_dir();
    ensure_dir(&config_dir)?;

    let config_path = config_dir.join(format!("{}.toml", tunnel.id));
    let toml = build_frpc_toml(tunnel);

    std::fs::write(&config_path, toml)
        .map_err(|e| format!("写入 frpc 配置失败: {}", e))?;

    log_info!("[Frp] frpc 配置已生成: {}", config_path.display());
    Ok(config_path)
}

/// 构建 frpc TOML 配置字符串
///
/// frpc v0.51+ TOML 格式：
/// ```toml
/// serverAddr = "x.x.x.x"
/// serverPort = 7000
/// auth.token = "xxx"
/// transport.tls.enable = true
///
/// [[proxies]]
/// name = "tunnel-name"
/// type = "tcp"
/// localIP = "127.0.0.1"
/// localPort = 25565
/// remotePort = 30001
/// ```
fn build_frpc_toml(tunnel: &Tunnel) -> String {
    let mut lines = Vec::new();

    // 服务端连接配置
    lines.push(format!("serverAddr = \"{}\"", tunnel.server_addr));
    lines.push(format!("serverPort = {}", tunnel.server_port));

    // 鉴权 token
    if let Some(ref token) = tunnel.token {
        if !token.is_empty() {
            lines.push(format!("auth.token = \"{}\"", escape_toml_string(token)));
        }
    }

    // TLS
    if tunnel.use_tls {
        lines.push("transport.tls.enable = true".to_string());
    }

    // 日志配置
    lines.push(String::new()); // 空行分隔
    lines.push(format!("log.to = \"{}\"", escape_toml_string(
        &super::frp_logs_dir().join(format!("{}.log", tunnel.id)).to_string_lossy()
    )));
    lines.push("log.level = \"info\"".to_string());
    lines.push("log.maxDays = 3".to_string());

    // 代理配置
    lines.push(String::new());
    lines.push("[[proxies]]".to_string());
    lines.push(format!("name = \"{}\"", escape_toml_string(&tunnel.name)));
    lines.push(format!("type = \"{}\"", match tunnel.tunnel_type {
        TunnelType::Tcp => "tcp",
        TunnelType::Udp => "udp",
    }));
    lines.push(format!("localIP = \"{}\"", escape_toml_string(&tunnel.local_ip)));
    lines.push(format!("localPort = {}", tunnel.local_port));
    lines.push(format!("remotePort = {}", tunnel.remote_port));

    lines.join("\n") + "\n"
}

/// TOML 字符串转义（处理引号和反斜杠）
fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ============================================================
// 持久化
// ============================================================

/// 读取 tunnels.json
fn read_tunnels() -> Result<Vec<Tunnel>, String> {
    let path = tunnels_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取隧道配置失败: {}", e))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let tunnels: Vec<Tunnel> = serde_json::from_str(&content)
        .map_err(|e| format!("解析隧道配置失败: {}", e))?;
    Ok(tunnels)
}

/// 写入 tunnels.json
fn write_tunnels(tunnels: &Vec<Tunnel>) -> Result<(), String> {
    let path = tunnels_path();
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let content = serde_json::to_string_pretty(tunnels)
        .map_err(|e| format!("序列化隧道配置失败: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("写入隧道配置失败: {}", e))?;
    Ok(())
}

/// 生成唯一 ID（时间戳 + 随机数）
fn generate_id() -> String {
    let ts = now_ms();
    let random: u64 = {
        let mut buf = [0u8; 8];
        // 使用系统时间纳秒作为随机源（无 uuid crate）
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(0);
        let seed = ts.wrapping_mul(2654435761).wrapping_add(nanos);
        // 简单的 xorshift
        let mut x = seed.wrapping_add(0x9E3779B97F4A7C15);
        x ^= x >> 30;
        x = x.wrapping_mul(0xBF58476D1CE4E5B9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94D049BB133111EB);
        x ^= x >> 31;
        buf.copy_from_slice(&x.to_le_bytes());
        u64::from_le_bytes(buf)
    };
    format!("tunnel-{:013x}{:08x}", ts, random)
}

/// 当前 Unix 毫秒时间戳
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================
// 参数结构体
// ============================================================

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTunnelParams {
    pub name: String,
    pub provider_id: String,
    pub tunnel_type: TunnelType,
    pub local_ip: Option<String>,
    pub local_port: u16,
    pub server_addr: String,
    pub server_port: u16,
    pub remote_port: u16,
    pub token: Option<String>,
    pub use_tls: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelIdParams {
    pub id: String,
}
