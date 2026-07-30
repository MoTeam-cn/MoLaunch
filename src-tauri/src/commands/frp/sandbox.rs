//! 安全沙箱：隧道配置校验
//!
//! 校验用户输入的隧道参数，防止注入和非法值。
//! 启动隧道前调用 `validate_tunnel` 进行校验。

use super::tunnel::CreateTunnelParams;
use super::{validate_provider_id, TunnelType};

/// 校验创建隧道参数
pub fn validate_tunnel(params: &CreateTunnelParams) -> Result<(), String> {
    // 厂商 ID 校验（非空 + kebab-case 格式）
    let provider_id = params.provider_id.trim();
    if provider_id.is_empty() {
        return Err("厂商 ID 不能为空".to_string());
    }
    validate_provider_id(provider_id)?;

    // 名称校验
    let name = params.name.trim();
    if name.is_empty() {
        return Err("隧道名称不能为空".to_string());
    }
    if name.len() > 64 {
        return Err("隧道名称不能超过 64 字符".to_string());
    }
    // 禁止换行和引号（防止 TOML 注入）
    if name.contains('\n') || name.contains('\r') || name.contains('"') || name.contains('\\') {
        return Err("隧道名称包含非法字符".to_string());
    }

    // 服务端地址校验
    let server_addr = params.server_addr.trim();
    if server_addr.is_empty() {
        return Err("服务器地址不能为空".to_string());
    }
    if server_addr.len() > 255 {
        return Err("服务器地址过长".to_string());
    }
    // 禁止换行、引号、反斜杠（防止 TOML 注入和路径遍历）
    if server_addr.contains('\n')
        || server_addr.contains('\r')
        || server_addr.contains('"')
        || server_addr.contains('\\')
    {
        return Err("服务器地址包含非法字符".to_string());
    }
    // 禁止 file:// 等协议前缀
    if server_addr.contains("://") {
        return Err("服务器地址不能包含协议前缀".to_string());
    }

    // 本地 IP 校验（如有）
    if let Some(ref local_ip) = params.local_ip {
        let local_ip = local_ip.trim();
        if !local_ip.is_empty() {
            if local_ip.contains('\n') || local_ip.contains('\r') || local_ip.contains('"') {
                return Err("本地 IP 包含非法字符".to_string());
            }
        }
    }

    // 端口校验
    if params.server_port == 0 {
        return Err("服务器端口不能为 0".to_string());
    }
    if params.local_port == 0 {
        return Err("本地端口不能为 0".to_string());
    }
    if params.remote_port == 0 {
        return Err("远程端口不能为 0".to_string());
    }

    // Token 校验（如有）
    if let Some(ref token) = params.token {
        if !token.is_empty() {
            if token.contains('\n') || token.contains('\r') || token.contains('"') || token.contains('\\') {
                return Err("Token 包含非法字符".to_string());
            }
            if token.len() > 512 {
                return Err("Token 过长".to_string());
            }
        }
    }

    // 隧道类型校验
    match params.tunnel_type {
        TunnelType::Tcp | TunnelType::Udp => {}
    }

    Ok(())
}
