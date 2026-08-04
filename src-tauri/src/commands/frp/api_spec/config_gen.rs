//! frpc 配置生成
//!
//! 按配置模式（url/fields/args）生成 frpc 配置内容或启动参数。

use super::{AccountInfo, TunnelInfo};
use crate::log_info;

/// 配置生成结果
#[derive(Debug, Clone)]
pub struct GeneratedConfig {
    /// frpc 配置文件内容（mode=url/fields 时有值，mode=args 时为空）
    pub content: Option<String>,
    /// frpc 启动参数（mode=args 时有值，附加到 frpc 命令行）
    pub args: Vec<String>,
    /// 配置模式
    pub mode: String,
}

/// 按隧道信息生成 frpc 配置
///
/// mode=url：调用厂商 config 端点获取配置字符串（由调用方提前获取并传入）
/// mode=fields：按隧道字段拼装 ini 配置
/// mode=args：生成启动参数（如 -u {token} -p {ids}）
pub fn generate(
    mode: &str,
    _format: &str,
    args_template: &[String],
    tunnel: &TunnelInfo,
    account: &AccountInfo,
    raw_config: Option<&str>,
    encoding: Option<&str>,
) -> Result<GeneratedConfig, String> {
    match mode {
        "url" => {
            // url 模式：直写厂商返回的配置（按 encoding 解码，如 base64）
            let raw = raw_config.ok_or_else(|| {
                "config.mode=url 但未提供配置内容（config 端点未调用或返回空）".to_string()
            })?;
            let content = decode_config(raw, encoding)?;
            log_info!(
                "[Frp] 配置生成: mode=url, encoding={:?}, 长度={}",
                encoding,
                content.len()
            );
            Ok(GeneratedConfig {
                content: Some(content),
                args: Vec::new(),
                mode: "url".to_string(),
            })
        }
        "fields" => {
            // fields 模式：按隧道字段拼装 ini
            let content = build_ini_config(tunnel, account)?;
            log_info!(
                "[Frp] 配置生成: mode=fields, 隧道={}, 长度={}",
                tunnel.name,
                content.len()
            );
            Ok(GeneratedConfig {
                content: Some(content),
                args: Vec::new(),
                mode: "fields".to_string(),
            })
        }
        "args" => {
            // args 模式：生成启动参数
            let args = build_args(args_template, tunnel, account)?;
            log_info!(
                "[Frp] 配置生成: mode=args, 隧道={}, 参数={:?}",
                tunnel.name,
                args
            );
            Ok(GeneratedConfig {
                content: None,
                args,
                mode: "args".to_string(),
            })
        }
        other => Err(format!(
            "不支持的 config.mode: {}（可选 url/fields/args）",
            other
        )),
    }
}

/// 按 ini 格式拼装 frpc 配置（fields 模式）
///
/// 生成标准 frpc ini 配置，包含 [common] 和隧道段。
fn build_ini_config(tunnel: &TunnelInfo, account: &AccountInfo) -> Result<String, String> {
    let mut lines = Vec::new();

    // [common] 段
    lines.push("[common]".to_string());
    if !tunnel.server_host.is_empty() {
        lines.push(format!("server_addr = {}", tunnel.server_host));
    }
    if !tunnel.server_port.is_empty() {
        lines.push(format!("server_port = {}", tunnel.server_port));
    }
    // token 优先用隧道的，其次 account.token
    let token = if !tunnel.token.is_empty() {
        &tunnel.token
    } else {
        &account.token
    };
    if !token.is_empty() {
        lines.push(format!("token = {}", token));
    }

    // 隧道段（按类型生成）
    let section = match tunnel.tunnel_type.as_str() {
        "tcp" | "udp" => {
            let mut s = Vec::new();
            s.push(format!("[{}]", tunnel.name));
            s.push(format!("type = {}", tunnel.tunnel_type));
            if !tunnel.local_host.is_empty() {
                s.push(format!("local_ip = {}", tunnel.local_host));
            }
            if !tunnel.local_port.is_empty() {
                s.push(format!("local_port = {}", tunnel.local_port));
            }
            if !tunnel.remote_port.is_empty() {
                s.push(format!("remote_port = {}", tunnel.remote_port));
            }
            s
        }
        "http" | "https" => {
            let mut s = Vec::new();
            s.push(format!("[{}]", tunnel.name));
            s.push(format!("type = {}", tunnel.tunnel_type));
            if !tunnel.local_host.is_empty() {
                s.push(format!("local_ip = {}", tunnel.local_host));
            }
            if !tunnel.local_port.is_empty() {
                s.push(format!("local_port = {}", tunnel.local_port));
            }
            if !tunnel.custom_domain.is_empty() {
                s.push(format!("custom_domains = {}", tunnel.custom_domain));
            }
            s
        }
        _ => {
            // 未知类型，按 tcp 处理
            let mut s = Vec::new();
            s.push(format!("[{}]", tunnel.name));
            s.push("type = tcp".to_string());
            if !tunnel.local_host.is_empty() {
                s.push(format!("local_ip = {}", tunnel.local_host));
            }
            if !tunnel.local_port.is_empty() {
                s.push(format!("local_port = {}", tunnel.local_port));
            }
            if !tunnel.remote_port.is_empty() {
                s.push(format!("remote_port = {}", tunnel.remote_port));
            }
            s
        }
    };
    lines.extend(section);

    Ok(lines.join("\n") + "\n")
}

/// 按参数模板生成启动参数（args 模式）
///
/// 替换占位符：{token} / {ids} / {account.token} 等
fn build_args(
    template: &[String],
    tunnel: &TunnelInfo,
    account: &AccountInfo,
) -> Result<Vec<String>, String> {
    let token = if !tunnel.token.is_empty() {
        &tunnel.token
    } else {
        &account.token
    };

    let result: Vec<String> = template
        .iter()
        .map(|arg| {
            arg.replace("{token}", token)
                .replace("{ids}", &tunnel.id)
                .replace("{account.token}", &account.token)
                .replace("{account.id}", &account.id)
                .replace("{tunnel.id}", &tunnel.id)
                .replace("{tunnel.name}", &tunnel.name)
        })
        .collect();

    Ok(result)
}

/// 解码配置内容（支持 base64）
///
/// `encoding` 取值：
/// - text：直接返回原始文本
/// - base64：Base64 解码后返回文本
///
/// Lolia 等厂商的 config 接口返回 base64 编码的 frpc 配置（`data.config`），
/// 需解码后才能得到可写盘的配置内容。
pub fn decode_config(raw: &str, encoding: Option<&str>) -> Result<String, String> {
    match encoding {
        Some("base64") => decode_base64(raw),
        _ => Ok(raw.to_string()),
    }
}

/// Base64 解码（标准 RFC 4648，容忍空白字符）
fn decode_base64(raw: &str) -> Result<String, String> {
    let cleaned: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&cleaned)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("Base64 解码结果非 UTF-8: {}", e))
}

#[cfg(test)]
#[path = "config_gen_tests.rs"]
mod tests;
