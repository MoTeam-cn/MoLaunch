//! 响应映射：按 response_mapping 将厂商响应映射到 ConfigPayload
//!
//! mapping 的 key 是厂商响应的 JSON 路径（如 "data.server_addr"），
//! value 是 ConfigPayload 的字段名（如 "serverAddr"）。
//! 标准字段名匹配后写入对应字段；非标准字段名视为自定义变量。

use super::ConfigPayload;
use crate::log_debug;
use std::collections::HashMap;

/// 按 response_mapping 将厂商响应映射到 ConfigPayload
pub(super) fn map_response(
    response: &serde_json::Value,
    mapping: &HashMap<String, String>,
) -> Result<ConfigPayload, String> {
    let mut payload = ConfigPayload::default();

    for (vendor_path, field_name) in mapping {
        match get_json_path(response, vendor_path) {
            Some(value) => {
                set_payload_field(&mut payload, field_name, value)?;
            }
            None => {
                if is_required_field(field_name) {
                    return Err(format!(
                        "厂商响应缺少必填字段: {}（JSON 路径: {}）",
                        field_name, vendor_path
                    ));
                }
                log_debug!(
                    "[Frp] 厂商响应可选字段缺失: {}（路径: {}）",
                    field_name,
                    vendor_path
                );
            }
        }
    }

    // 必填字段最终校验
    if payload.server_addr.is_empty() {
        return Err("厂商响应未提供服务器地址 (serverAddr)".to_string());
    }
    if payload.server_port == 0 {
        return Err("厂商响应未提供服务器端口 (serverPort)".to_string());
    }

    Ok(payload)
}

/// 按 dot 分隔路径从 JSON Value 取值
///
/// 支持如 "data.server_addr" 的路径，逐层深入。
/// 路径段为空时跳过。任一段不存在返回 None。
pub(super) fn get_json_path(value: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    let mut current = value;
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        current = current.get(segment)?;
    }
    Some(current.clone())
}

/// 写入 ConfigPayload 对应字段
///
/// 同时兼容 camelCase（schema 中的写法）和 snake_case（Rust 字段名）。
/// 非标准字段名视为自定义变量，写入 custom_variables。
fn set_payload_field(
    payload: &mut ConfigPayload,
    field_name: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    match field_name {
        "serverAddr" | "server_addr" => {
            payload.server_addr = value_as_string(&value)
                .ok_or_else(|| format!("字段 {} 的值不是有效字符串", field_name))?;
        }
        "serverPort" | "server_port" => {
            payload.server_port = value_as_u16(&value)
                .ok_or_else(|| format!("字段 {} 的值不是有效端口", field_name))?;
        }
        "token" => {
            payload.token = value_as_string(&value);
        }
        "assignedRemotePort" | "assigned_remote_port" => {
            payload.assigned_remote_port = value_as_u16(&value);
        }
        "assignedSubdomain" | "assigned_subdomain" => {
            payload.assigned_subdomain = value_as_string(&value);
        }
        // 非标准字段名 → 自定义变量
        other => {
            let str_val = value_as_string(&value).unwrap_or_else(|| value.to_string());
            payload
                .custom_variables
                .get_or_insert_with(HashMap::new)
                .insert(other.to_string(), str_val);
        }
    }
    Ok(())
}

/// 判断字段是否为必填（serverAddr / serverPort）
fn is_required_field(field_name: &str) -> bool {
    matches!(
        field_name,
        "serverAddr" | "server_addr" | "serverPort" | "server_port"
    )
}

/// JSON Value → String（字符串原样返回，数字/布尔转字符串）
fn value_as_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// JSON Value → u16（数字直接转换，字符串解析）
fn value_as_u16(value: &serde_json::Value) -> Option<u16> {
    match value {
        serde_json::Value::Number(n) => n.as_u64().and_then(|v| u16::try_from(v).ok()),
        serde_json::Value::String(s) => s.parse::<u16>().ok(),
        _ => None,
    }
}
