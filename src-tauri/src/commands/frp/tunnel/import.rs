use crate::commands::frp::TunnelType;

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedFrpcConfig {
    pub server_addr: Option<String>,
    pub server_port: Option<u16>,
    pub user: Option<String>,
    pub token: Option<String>,
    pub name: Option<String>,
    pub tunnel_type: Option<TunnelType>,
    pub local_ip: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub use_tls: bool,
    pub bandwidth_limit: Option<String>,
    pub bandwidth_limit_mode: Option<String>,
    pub proxy_use_encryption: Option<bool>,
    pub proxy_use_compression: Option<bool>,
    pub proxy_protocol_version: Option<String>,
}

pub fn import_frpc_config(path: String) -> Result<ImportedFrpcConfig, String> {
    let extension = std::path::Path::new(&path)
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if extension != "toml" && extension != "conf" {
        return Err("仅支持 .toml 或 .conf 配置文件".to_string());
    }
    let metadata = std::fs::metadata(&path).map_err(|e| format!("读取配置文件失败: {}", e))?;
    if metadata.len() > 1024 * 1024 {
        return Err("配置文件超过 1 MB，已拒绝导入".to_string());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {}", e))?;
    if content.contains('\0') {
        return Err("配置文件包含非法字符".to_string());
    }
    let mut section = String::new();
    let mut result = ImportedFrpcConfig::default();
    for raw_line in content.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("[[") && line.ends_with("]]") {
            section = line[2..line.len() - 2].trim().to_string();
            if section != "proxies" {
                return Err(format!("不支持的配置段: {}", section));
            }
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if !matches!(
                section.as_str(),
                "auth" | "metadatas" | "proxies.transport" | "transport.tls"
            ) {
                return Err(format!("不支持的配置段: {}", section));
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err("配置存在无法解析的行".to_string());
        };
        let key = key.trim();
        let value = value.trim();
        match (section.as_str(), key) {
            ("", "serverAddr") => result.server_addr = Some(parse_string(value)?),
            ("", "serverPort") => result.server_port = Some(parse_u16(value, "serverPort")?),
            ("", "user") => result.user = Some(parse_string(value)?),
            ("auth" | "metadatas", "token") => result.token = Some(parse_string(value)?),
            ("proxies", "name") => result.name = Some(parse_string(value)?),
            ("proxies", "type") => result.tunnel_type = Some(parse_type(&parse_string(value)?)?),
            ("proxies", "localIP") => result.local_ip = Some(parse_string(value)?),
            ("proxies", "localPort") => result.local_port = Some(parse_u16(value, "localPort")?),
            ("proxies", "remotePort") => result.remote_port = Some(parse_u16(value, "remotePort")?),
            ("proxies.transport", "bandwidthLimit") => {
                result.bandwidth_limit = Some(parse_string(value)?)
            }
            ("proxies.transport", "bandwidthLimitMode") => {
                result.bandwidth_limit_mode = Some(parse_string(value)?)
            }
            ("proxies.transport", "useEncryption") => {
                result.proxy_use_encryption = Some(parse_bool(value, "useEncryption")?)
            }
            ("proxies.transport", "useCompression") => {
                result.proxy_use_compression = Some(parse_bool(value, "useCompression")?)
            }
            ("proxies.transport", "protocolVersion") => {
                result.proxy_protocol_version = Some(parse_string(value)?)
            }
            ("transport.tls", "enable") => {
                result.use_tls = parse_bool(value, "transport.tls.enable")?
            }
            _ => return Err(format!("配置字段不在允许列表: {}.{}", section, key)),
        }
    }
    if result.server_addr.is_none()
        || result.server_port.is_none()
        || result.local_port.is_none()
        || result.remote_port.is_none()
    {
        return Err("配置缺少必要字段：serverAddr/serverPort/localPort/remotePort".to_string());
    }
    Ok(result)
}

fn parse_string(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.len() < 2
        || !((value.starts_with('\'') && value.ends_with('\''))
            || (value.starts_with('"') && value.ends_with('"')))
    {
        return Err("仅支持单行 TOML 字符串".to_string());
    }
    let inner = &value[1..value.len() - 1];
    if inner.contains('\n') || inner.contains('\r') {
        return Err("字符串包含换行".to_string());
    }
    Ok(inner.replace("\\'", "'").replace("\\\"", "\""))
}
fn parse_u16(value: &str, key: &str) -> Result<u16, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("{} 必须是有效端口", key))
}
fn parse_bool(value: &str, key: &str) -> Result<bool, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("{} 必须是布尔值", key))
}
fn parse_type(value: &str) -> Result<TunnelType, String> {
    match value {
        "tcp" => Ok(TunnelType::Tcp),
        "udp" => Ok(TunnelType::Udp),
        _ => Err("仅支持 tcp/udp 配置".to_string()),
    }
}
