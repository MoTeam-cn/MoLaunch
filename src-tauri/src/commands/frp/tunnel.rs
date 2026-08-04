//! 隧道管理：CRUD + frpc TOML 配置生成
//!
//! 隧道配置持久化到 `<base_dir>/frp/tunnels.json`。
//! 启动隧道时生成 TOML 配置文件到 `<base_dir>/frp/config/<tunnel_id>.toml`。

use super::{ensure_dir, frp_config_dir, frp_logs_dir, tunnels_path, Tunnel, TunnelType};
use crate::log_info;
use serde::{Deserialize, Deserializer};
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

    // 清理日志文件（隧道已删除，残留日志无意义且占存储）
    let log_path = frp_logs_dir().join(format!("{}.log", id));
    if log_path.exists() {
        std::fs::remove_file(&log_path).ok();
    }

    log_info!("[Frp] 隧道已删除: {} (含配置与日志文件)", id);
    Ok(())
}

/// 更新隧道配置
///
/// 更新持久化的 tunnels.json 并重新生成 frpc TOML 配置文件。
/// 若隧道正在运行，调用方应先停止隧道再更新（本函数不处理进程）。
pub async fn update_tunnel(params: UpdateTunnelParams) -> Result<Tunnel, String> {
    let mut tunnels = read_tunnels()?;

    // 名称唯一性校验（排除自身）——须在 iter_mut 借用前完成，避免可变/不可变借用冲突
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

    // 重新生成 frpc TOML 配置（覆盖旧文件）
    generate_config(&updated)?;

    log_info!("[Frp] 隧道已更新: {} ({})", updated.name, updated.id);
    Ok(updated)
}

/// 生成 frpc TOML 配置文件
///
/// 写入 `<base_dir>/frp/config/<tunnel_id>.toml`，返回文件路径。
/// 统一使用与厂商 config 接口返回相同的原版格式：
/// `serverAddr/serverPort/user` + `[metadatas] token` + `[[proxies]]`。
pub fn generate_config(tunnel: &Tunnel) -> Result<std::path::PathBuf, String> {
    let config_dir = frp_config_dir();
    ensure_dir(&config_dir)?;

    let config_path = config_dir.join(format!("{}.toml", tunnel.id));
    let toml = build_frpc_toml(tunnel);

    std::fs::write(&config_path, toml).map_err(|e| format!("写入 frpc 配置失败: {}", e))?;

    log_info!("[Frp] frpc 配置已生成: {}", config_path.display());
    Ok(config_path)
}

/// 构建 frpc TOML 配置字符串
///
/// 复用 `frpc_config` 工具生成与厂商 config 接口返回同构的 TOML：
/// 顶层 `serverAddr/serverPort/user`、`[metadatas] token`、`[[proxies]]`。
/// `user` 取隧道 name（frp 中标识账户归属，兼容部分厂商要求）。
fn build_frpc_toml(tunnel: &Tunnel) -> String {
    use super::frpc_config::{build_frpc_toml, Proxy, ServerConn};

    let conn = ServerConn {
        server_addr: tunnel.server_addr.clone(),
        server_port: tunnel.server_port,
        user: Some(tunnel.name.clone()),
        token: tunnel.token.clone(),
        use_tls: tunnel.use_tls,
    };
    let proxy = Proxy {
        // 厂商隧道用真实隧道 name（config 接口查询、服务端识别均用该值），
        // 本地自建隧道回退用隧道 name
        name: tunnel
            .remote_tunnel_name
            .clone()
            .unwrap_or_else(|| tunnel.name.clone()),
        proxy_type: match tunnel.tunnel_type {
            TunnelType::Tcp => "tcp".to_string(),
            TunnelType::Udp => "udp".to_string(),
        },
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

// 持久化
/// 读取 tunnels.json
fn read_tunnels() -> Result<Vec<Tunnel>, String> {
    let path = tunnels_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取隧道配置失败: {}", e))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let tunnels: Vec<Tunnel> =
        serde_json::from_str(&content).map_err(|e| format!("解析隧道配置失败: {}", e))?;
    Ok(tunnels)
}

/// 写入 tunnels.json
fn write_tunnels(tunnels: &Vec<Tunnel>) -> Result<(), String> {
    let path = tunnels_path();
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let content =
        serde_json::to_string_pretty(tunnels).map_err(|e| format!("序列化隧道配置失败: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("写入隧道配置失败: {}", e))?;
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

// 参数结构体
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTunnelParams {
    pub name: String,
    pub provider_id: String,
    pub tunnel_type: TunnelType,
    pub local_ip: Option<String>,
    #[serde(deserialize_with = "deserialize_u16_flexible")]
    pub local_port: u16,
    pub server_addr: String,
    #[serde(deserialize_with = "deserialize_u16_flexible")]
    pub server_port: u16,
    #[serde(deserialize_with = "deserialize_u16_flexible")]
    pub remote_port: u16,
    pub token: Option<String>,
    pub use_tls: Option<bool>,
    /// 是否为厂商同步导入请求。仅允许存在远程隧道标识时使用。
    #[serde(default)]
    pub imported: bool,
    /// 厂商远端隧道自增 ID（从厂商 API 导入时传入，用于同步面板判断已导入）
    #[serde(default)]
    pub remote_tunnel_id: Option<String>,
    /// 厂商远端隧道真实 name（config 接口查询、frpc 代理 name 用）
    #[serde(default)]
    pub remote_tunnel_name: Option<String>,
    /// 厂商 config 接口返回的完整配置
    #[serde(default)]
    pub raw_config: Option<String>,
    /// 带宽限制（如 "4MB"），写入 `[proxies.transport] bandwidthLimit`
    #[serde(default)]
    pub bandwidth_limit: Option<String>,
    /// 带宽限制模式（如 "server"），写入 `[proxies.transport] bandwidthLimitMode`
    #[serde(default)]
    pub bandwidth_limit_mode: Option<String>,
    #[serde(default)]
    pub proxy_use_encryption: Option<bool>,
    #[serde(default)]
    pub proxy_use_compression: Option<bool>,
    #[serde(default)]
    pub proxy_protocol_version: Option<String>,
}

/// 兼容前端 number 输入控件提交的数字或数字字符串，并由 u16 负责范围限制。
fn deserialize_u16_flexible<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(number) => number
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .ok_or_else(|| serde::de::Error::custom("必须是 0-65535 的整数")),
        serde_json::Value::String(text) => text
            .trim()
            .parse::<u16>()
            .map_err(|_| serde::de::Error::custom("必须是 0-65535 的整数")),
        _ => Err(serde::de::Error::custom("必须是数字或数字字符串")),
    }
}

/// 安全导入 frpc TOML 配置：只提取受支持字段，不透传任意配置。
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
        if line.starts_with("[[") && line.ends_with("]]" ) {
            section = line[2..line.len() - 2].trim().to_string();
            if section != "proxies" {
                return Err(format!("不支持的配置段: {}", section));
            }
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "auth" && section != "metadatas" && section != "proxies.transport" && section != "transport.tls" {
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
            ("", "serverAddr") => result.server_addr = Some(parse_toml_string(value)?),
            ("", "serverPort") => result.server_port = Some(parse_toml_u16(value, "serverPort")?),
            ("", "user") => result.user = Some(parse_toml_string(value)?),
            ("auth", "token") => result.token = Some(parse_toml_string(value)?),
            ("metadatas", "token") => result.token = Some(parse_toml_string(value)?),
            ("proxies", "name") => result.name = Some(parse_toml_string(value)?),
            ("proxies", "type") => result.tunnel_type = Some(parse_tunnel_type(&parse_toml_string(value)?)?),
            ("proxies", "localIP") => result.local_ip = Some(parse_toml_string(value)?),
            ("proxies", "localPort") => result.local_port = Some(parse_toml_u16(value, "localPort")?),
            ("proxies", "remotePort") => result.remote_port = Some(parse_toml_u16(value, "remotePort")?),
            ("proxies.transport", "bandwidthLimit") => result.bandwidth_limit = Some(parse_toml_string(value)?),
            ("proxies.transport", "bandwidthLimitMode") => result.bandwidth_limit_mode = Some(parse_toml_string(value)?),
            ("proxies.transport", "useEncryption") => result.proxy_use_encryption = Some(parse_toml_bool(value, "useEncryption")?),
            ("proxies.transport", "useCompression") => result.proxy_use_compression = Some(parse_toml_bool(value, "useCompression")?),
            ("proxies.transport", "protocolVersion") => result.proxy_protocol_version = Some(parse_toml_string(value)?),
            ("transport.tls", "enable") => result.use_tls = parse_toml_bool(value, "transport.tls.enable")?,
            _ => return Err(format!("配置字段不在允许列表: {}.{}", section, key)),
        }
    }
    if result.server_addr.is_none() || result.server_port.is_none() || result.local_port.is_none() || result.remote_port.is_none() {
        return Err("配置缺少必要字段：serverAddr/serverPort/localPort/remotePort".to_string());
    }
    Ok(result)
}

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

fn parse_toml_string(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.len() < 2 || !((value.starts_with('\'') && value.ends_with('\'')) || (value.starts_with('"') && value.ends_with('"'))) {
        return Err("仅支持单行 TOML 字符串".to_string());
    }
    let inner = &value[1..value.len() - 1];
    if inner.contains('\n') || inner.contains('\r') {
        return Err("字符串包含换行".to_string());
    }
    Ok(inner.replace("\\'", "'").replace("\\\"", "\""))
}

fn parse_toml_u16(value: &str, key: &str) -> Result<u16, String> {
    value.trim().parse().map_err(|_| format!("{} 必须是有效端口", key))
}

fn parse_toml_bool(value: &str, key: &str) -> Result<bool, String> {
    value.trim().parse().map_err(|_| format!("{} 必须是布尔值", key))
}

fn parse_tunnel_type(value: &str) -> Result<TunnelType, String> {
    match value {
        "tcp" => Ok(TunnelType::Tcp),
        "udp" => Ok(TunnelType::Udp),
        _ => Err("仅支持 tcp/udp 配置".to_string()),
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelIdParams {
    pub id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTunnelParams {
    pub id: String,
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
    /// 带宽限制（如 "4MB"），写入 `[proxies.transport] bandwidthLimit`
    #[serde(default)]
    pub bandwidth_limit: Option<String>,
    /// 带宽限制模式（如 "server"），写入 `[proxies.transport] bandwidthLimitMode`
    #[serde(default)]
    pub bandwidth_limit_mode: Option<String>,
    #[serde(default)]
    pub proxy_use_encryption: Option<bool>,
    #[serde(default)]
    pub proxy_use_compression: Option<bool>,
    #[serde(default)]
    pub proxy_protocol_version: Option<String>,
}
