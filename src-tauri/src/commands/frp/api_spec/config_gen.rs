//! frpc 配置生成
//!
//! 三种模式（对应 endpoints.json config.mode）：
//! - `url`：厂商接口直返 frpc 配置字符串，启动器写盘即可
//! - `fields`：启动器按 tunnelFields 拼装 ini 配置
//! - `args`：frpc 以启动参数方式运行（如 OpenFRP `-u 密钥 -p 隧道ID`）
//!
//! 解码扩展点：url 模式预留 decoding 配置（base64/xor/aes），
//! 当前实现直通原始文本，后续可按需扩展解码逻辑。

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
    format: &str,
    args_template: &[String],
    tunnel: &TunnelInfo,
    account: &AccountInfo,
    raw_config: Option<&str>,
) -> Result<GeneratedConfig, String> {
    match mode {
        "url" => {
            // url 模式：直写厂商返回的配置（预留解码扩展点）
            let raw = raw_config.ok_or_else(|| {
                "config.mode=url 但未提供配置内容（config 端点未调用或返回空）".to_string()
            })?;
            let content = decode_config(raw)?;
            log_info!("[Frp] 配置生成: mode=url, 长度={}", content.len());
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
            s.push(format!("type = tcp"));
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

/// 解码配置内容（预留扩展点）
///
/// 当前直接返回原始文本。后续可按 decoding 配置扩展：
/// - base64：Base64 解码
/// - xor：XOR 解密（需密钥）
/// - aes：AES 解密（需密钥）
fn decode_config(raw: &str) -> Result<String, String> {
    // 当前直接返回原始文本
    // TODO: 按 decoding 配置扩展解码逻辑
    Ok(raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tunnel() -> TunnelInfo {
        TunnelInfo {
            id: "t1".to_string(),
            name: "test".to_string(),
            tunnel_type: "tcp".to_string(),
            status: "running".to_string(),
            server_host: "example.com".to_string(),
            server_port: "7000".to_string(),
            token: "secret".to_string(),
            local_host: "127.0.0.1".to_string(),
            local_port: "25565".to_string(),
            remote_port: "25565".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_fields_mode() {
        let tunnel = test_tunnel();
        let account = AccountInfo::default();
        let result = generate("fields", "ini", &[], &tunnel, &account, None).unwrap();
        let content = result.content.unwrap();
        assert!(content.contains("server_addr = example.com"));
        assert!(content.contains("server_port = 7000"));
        assert!(content.contains("[test]"));
        assert!(content.contains("type = tcp"));
    }

    #[test]
    fn test_args_mode() {
        let tunnel = test_tunnel();
        let account = AccountInfo::default();
        let template = vec!["-u".to_string(), "{token}".to_string(), "-p".to_string(), "{ids}".to_string()];
        let result = generate("args", "ini", &template, &tunnel, &account, None).unwrap();
        assert_eq!(result.args, vec!["-u", "secret", "-p", "t1"]);
    }

    #[test]
    fn test_url_mode() {
        let tunnel = test_tunnel();
        let account = AccountInfo::default();
        let raw = "[common]\nserver_addr = x\n";
        let result = generate("url", "ini", &[], &tunnel, &account, Some(raw)).unwrap();
        assert_eq!(result.content.unwrap(), raw);
    }
}
