//! API 响应 DTO 与声明式字段映射。

use serde::Serialize;

use crate::commands::frp::api_spec::{envelope, jsonpath};
use crate::commands::frp::{EndpointDef, FieldMapping};

/// 隧道信息（从厂商 API 响应映射后的统一格式）
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInfo {
    pub id: String,
    pub name: String,
    pub remark: String,
    pub tunnel_type: String,
    pub status: String,
    pub server_host: String,
    pub server_port: String,
    pub token: String,
    pub local_host: String,
    pub local_port: String,
    pub remote_port: String,
    pub custom_domain: String,
    pub raw_config: Option<String>,
    pub source_data: serde_json::Value,
}

/// 账号信息（从厂商 API 响应映射）
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub token: String,
}

pub(super) fn map_account(
    response: &serde_json::Value,
    endpoint: &EndpointDef,
) -> Result<AccountInfo, String> {
    let data = envelope::extract_data(
        response,
        endpoint.envelope.as_ref(),
        endpoint.response.data_field.as_deref(),
    )?;
    let value = data.as_ref().unwrap_or(response);
    let fields = &endpoint.response.fields;
    Ok(AccountInfo {
        id: get_field(value, fields.get("id")),
        username: get_field(value, fields.get("username")),
        email: get_field(value, fields.get("email")),
        token: get_field(value, fields.get("token")),
    })
}

pub(super) fn map_tunnels(
    response: &serde_json::Value,
    endpoint: &EndpointDef,
    account: &AccountInfo,
) -> Result<Vec<TunnelInfo>, String> {
    let items = if let Some(ref items_field) = endpoint.response.items_field {
        jsonpath::extract_array(response, items_field)?
    } else {
        match envelope::extract_data(
            response,
            endpoint.envelope.as_ref(),
            endpoint.response.data_field.as_deref(),
        )? {
            Some(serde_json::Value::Array(arr)) => arr,
            Some(v) => vec![v],
            None => vec![],
        }
    };

    let fields = &endpoint.response.fields;
    Ok(items
        .into_iter()
        .map(|item| TunnelInfo {
            id: resolve_field(&item, fields.get("id"), account, None),
            name: resolve_field(&item, fields.get("name"), account, None),
            remark: resolve_field(&item, fields.get("remark"), account, None),
            tunnel_type: resolve_field(&item, fields.get("type"), account, None),
            status: resolve_field(&item, fields.get("status"), account, None),
            server_host: resolve_field(&item, fields.get("serverHost"), account, Some(0)),
            server_port: resolve_field(&item, fields.get("serverPort"), account, Some(1)),
            token: resolve_field(&item, fields.get("token"), account, None),
            local_host: resolve_field(&item, fields.get("localHost"), account, None),
            local_port: resolve_field(&item, fields.get("localPort"), account, None),
            remote_port: resolve_field(&item, fields.get("remotePort"), account, None),
            custom_domain: resolve_field(&item, fields.get("customDomain"), account, None),
            raw_config: None,
            source_data: item,
        })
        .collect())
}

fn get_field(value: &serde_json::Value, mapping: Option<&FieldMapping>) -> String {
    mapping
        .map(|mapping| resolve_field(value, Some(mapping), &AccountInfo::default(), None))
        .unwrap_or_default()
}

pub(super) fn resolve_field(
    item: &serde_json::Value,
    mapping: Option<&FieldMapping>,
    account: &AccountInfo,
    split_index: Option<usize>,
) -> String {
    let Some(mapping) = mapping else {
        return String::new();
    };
    if let Some(value) = &mapping.value {
        return resolve_value_template(value, account);
    }
    let Some(field_name) = &mapping.field else {
        return String::new();
    };
    let raw = item
        .get(field_name)
        .map(|value| match value {
            serde_json::Value::String(value) => value.clone(),
            serde_json::Value::Number(value) => value.to_string(),
            serde_json::Value::Bool(value) => value.to_string(),
            value => value.to_string(),
        })
        .unwrap_or_default();
    if let Some(separator) = &mapping.split {
        if let Some(index) = split_index {
            if let Some(part) = raw.split(separator).collect::<Vec<_>>().get(index) {
                return part.trim().to_string();
            }
        }
    }
    raw
}

fn resolve_value_template(template: &str, account: &AccountInfo) -> String {
    template
        .replace("{account.token}", &account.token)
        .replace("{account.id}", &account.id)
        .replace("{account.username}", &account.username)
        .replace("{account.email}", &account.email)
}

pub(super) fn parse_config_fields(
    config: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let mut server_addr = None;
    let mut server_port = None;
    let mut remote_port = None;
    for line in config.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let Some(eq) = line.find('=') else { continue };
        let key = line[..eq].trim();
        let value = line[eq + 1..].trim().trim_matches('"').trim_matches('\'');
        match key {
            "serverAddr" | "server_addr" if server_addr.is_none() && !value.is_empty() => {
                server_addr = Some(value.to_string())
            }
            "serverPort" | "server_port"
                if server_port.is_none() && value.parse::<u16>().is_ok() =>
            {
                server_port = Some(value.to_string())
            }
            "remotePort" | "remote_port"
                if remote_port.is_none() && value.parse::<u16>().is_ok() =>
            {
                remote_port = Some(value.to_string())
            }
            _ => {}
        }
    }
    (server_addr, server_port, remote_port)
}
