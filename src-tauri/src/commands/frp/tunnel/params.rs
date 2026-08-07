use serde::{Deserialize, Deserializer};

use crate::commands::frp::TunnelType;

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
    #[serde(default)]
    pub imported: bool,
    #[serde(default)]
    pub remote_tunnel_id: Option<String>,
    #[serde(default)]
    pub remote_tunnel_name: Option<String>,
    #[serde(default)]
    pub raw_config: Option<String>,
    #[serde(default)]
    pub bandwidth_limit: Option<String>,
    #[serde(default)]
    pub bandwidth_limit_mode: Option<String>,
    #[serde(default)]
    pub proxy_use_encryption: Option<bool>,
    #[serde(default)]
    pub proxy_use_compression: Option<bool>,
    #[serde(default)]
    pub proxy_protocol_version: Option<String>,
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
    #[serde(default)]
    pub bandwidth_limit: Option<String>,
    #[serde(default)]
    pub bandwidth_limit_mode: Option<String>,
    #[serde(default)]
    pub proxy_use_encryption: Option<bool>,
    #[serde(default)]
    pub proxy_use_compression: Option<bool>,
    #[serde(default)]
    pub proxy_protocol_version: Option<String>,
}

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
