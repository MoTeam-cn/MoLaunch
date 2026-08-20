use super::params::{CreateTunnelParams, UpdateTunnelParams};
use crate::commands::frp::{ensure_dir, frp_config_dir, frp_logs_dir, tunnels_path, Tunnel};
use crate::log_info;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn list_tunnels() -> Result<Vec<Tunnel>, String> {
    read_tunnels()
}

pub async fn create_tunnel(params: CreateTunnelParams) -> Result<Tunnel, String> {
    let mut tunnels = read_tunnels()?;
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
        remote_tunnel_id: params.remote_tunnel_id,
        remote_tunnel_name: params.remote_tunnel_name,
        raw_config: params.raw_config,
        bandwidth_limit: params.bandwidth_limit,
        bandwidth_limit_mode: params.bandwidth_limit_mode,
        proxy_use_encryption: params.proxy_use_encryption,
        proxy_use_compression: params.proxy_use_compression,
        proxy_protocol_version: params.proxy_protocol_version,
        created_at: now_ms(),
    };
    tunnels.push(tunnel.clone());
    write_tunnels(&tunnels)?;
    log_info!("[Frp] 隧道已创建: {} ({})", tunnel.name, tunnel.id);
    Ok(tunnel)
}

pub async fn delete_tunnel(id: String) -> Result<(), String> {
    // 校验 id 仅含安全字符，防止路径穿越删除目录外文件
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!("非法隧道 id: {}", id));
    }
    let mut tunnels = read_tunnels()?;
    let before = tunnels.len();
    tunnels.retain(|t| t.id != id);
    if tunnels.len() == before {
        return Err(format!("隧道不存在: {}", id));
    }
    write_tunnels(&tunnels)?;
    let config_path = frp_config_dir().join(format!("{}.toml", id));
    if config_path.exists() {
        std::fs::remove_file(config_path).ok();
    }
    let log_path = frp_logs_dir().join(format!("{}.log", id));
    if log_path.exists() {
        std::fs::remove_file(log_path).ok();
    }
    log_info!("[Frp] 隧道已删除: {} (含配置与日志文件)", id);
    Ok(())
}

pub async fn update_tunnel(params: UpdateTunnelParams) -> Result<Tunnel, String> {
    let mut tunnels = read_tunnels()?;
    if tunnels
        .iter()
        .any(|t| t.id != params.id && t.name == params.name)
    {
        return Err(format!("隧道名称已存在: {}", params.name));
    }
    let tunnel = tunnels
        .iter_mut()
        .find(|t| t.id == params.id)
        .ok_or_else(|| format!("隧道不存在: {}", params.id))?;
    tunnel.name = params.name;
    tunnel.provider_id = params.provider_id;
    tunnel.tunnel_type = params.tunnel_type;
    tunnel.local_ip = params.local_ip.unwrap_or_else(|| "127.0.0.1".to_string());
    tunnel.local_port = params.local_port;
    tunnel.server_addr = params.server_addr;
    tunnel.server_port = params.server_port;
    tunnel.remote_port = params.remote_port;
    tunnel.token = params.token;
    tunnel.use_tls = params.use_tls.unwrap_or(false);
    tunnel.bandwidth_limit = params.bandwidth_limit;
    tunnel.bandwidth_limit_mode = params.bandwidth_limit_mode;
    tunnel.proxy_use_encryption = params.proxy_use_encryption;
    tunnel.proxy_use_compression = params.proxy_use_compression;
    tunnel.proxy_protocol_version = params.proxy_protocol_version;
    let updated = tunnel.clone();
    write_tunnels(&tunnels)?;
    super::config::generate_config(&updated)?;
    log_info!("[Frp] 隧道已更新: {} ({})", updated.name, updated.id);
    Ok(updated)
}

fn read_tunnels() -> Result<Vec<Tunnel>, String> {
    let path = tunnels_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取隧道配置失败: {}", e))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&content).map_err(|e| format!("解析隧道配置失败: {}", e))
}

fn write_tunnels(tunnels: &[Tunnel]) -> Result<(), String> {
    let path = tunnels_path();
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let content =
        serde_json::to_string_pretty(tunnels).map_err(|e| format!("序列化隧道配置失败: {}", e))?;
    std::fs::write(path, content).map_err(|e| format!("写入隧道配置失败: {}", e))
}

fn generate_id() -> String {
    let ts = now_ms();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let mut x = ts
        .wrapping_mul(2654435761)
        .wrapping_add(nanos)
        .wrapping_add(0x9E3779B97F4A7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^= x >> 31;
    format!("tunnel-{:013x}{:08x}", ts, x)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
