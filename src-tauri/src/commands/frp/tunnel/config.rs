use crate::commands::frp::{ensure_dir, frp_config_dir, Tunnel, TunnelType};
use crate::log_info;

pub fn generate_config(tunnel: &Tunnel) -> Result<std::path::PathBuf, String> {
    let config_dir = frp_config_dir();
    ensure_dir(&config_dir)?;
    let config_path = config_dir.join(format!("{}.toml", tunnel.id));
    std::fs::write(&config_path, build_frpc_toml(tunnel))
        .map_err(|e| format!("写入 frpc 配置失败: {}", e))?;
    log_info!("[Frp] frpc 配置已生成: {}", config_path.display());
    Ok(config_path)
}

fn build_frpc_toml(tunnel: &Tunnel) -> String {
    use crate::commands::frp::frpc_config::{build_frpc_toml, Proxy, ServerConn};
    let conn = ServerConn {
        server_addr: tunnel.server_addr.clone(),
        server_port: tunnel.server_port,
        user: Some(tunnel.name.clone()),
        token: tunnel.token.clone(),
        use_tls: tunnel.use_tls,
    };
    let proxy = Proxy {
        name: tunnel
            .remote_tunnel_name
            .clone()
            .unwrap_or_else(|| tunnel.name.clone()),
        proxy_type: match tunnel.tunnel_type {
            TunnelType::Tcp => "tcp",
            TunnelType::Udp => "udp",
        }
        .to_string(),
        local_ip: tunnel.local_ip.clone(),
        local_port: tunnel.local_port,
        remote_port: tunnel.remote_port,
        custom_domains: None,
        bandwidth_limit: tunnel.bandwidth_limit.clone(),
        bandwidth_limit_mode: tunnel.bandwidth_limit_mode.clone(),
        use_encryption: tunnel.proxy_use_encryption,
        use_compression: tunnel.proxy_use_compression,
        protocol_version: tunnel.proxy_protocol_version.clone(),
    };
    build_frpc_toml(&conn, &[proxy])
}
