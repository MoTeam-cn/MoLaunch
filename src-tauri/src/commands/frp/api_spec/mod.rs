//! 厂商 Open API 引擎：endpoints.json 解析 + API 调用 + 配置生成
//!
//! 设计参考：docs/Frp Test/frp/api/endpoints.json
//!
//! 厂商接口响应结构各不相同（成功判断、字段命名、数据位置、配置获取方式均有差异）。
//! 本模块将这些差异全部做成可配置项，厂商只需在 endpoints.json 中声明接口与响应解析，
//! 启动器即可正确调用并生成 frpc 配置。
//!
//! 子模块：
//! - `jsonpath`：JSONPath 解析（支持 `$.a.b` 和 `$.data[*].items[*]` 展平）
//! - `envelope`：响应包裹解析（成功判断 + 数据提取 + 错误消息）
//! - `http`：HTTP 请求发送（含重定向防护 + token 注入）
//! - `config_gen`：frpc 配置生成（url 直写 / fields 拼装 / args 启动参数）

use super::{providers_root, validate_provider_id, ApiSpec};
use crate::log_info;
use crate::state::AppState;
use std::collections::HashMap;

pub mod config_gen;
pub mod envelope;
pub mod http;
pub mod jsonpath;

// ============================================================
// 统一隧道数据（API 响应映射后的标准格式）
// ============================================================

/// 隧道信息（从厂商 API 响应映射后的统一格式）
///
/// 对应 endpoints.json 中 tunnelFields 定义的字段。
/// fields 模式下启动器按这些字段拼装 frpc 配置。
#[derive(Debug, Clone, Default)]
pub struct TunnelInfo {
    pub id: String,
    pub name: String,
    pub tunnel_type: String,
    pub status: String,
    pub server_host: String,
    pub server_port: String,
    pub token: String,
    pub local_host: String,
    pub local_port: String,
    pub remote_port: String,
    pub custom_domain: String,
}

/// 账号信息（从厂商 API 响应映射）
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub token: String,
}

// ============================================================
// 公共函数
// ============================================================

/// 读取并解析厂商目录下的 endpoints.json
///
/// 文件位置：`<providers_root>/<provider_id>/<endpoints_file>`
/// endpoints_file 由 manifest.api.endpointsFile 指定，默认 "api/endpoints.json"。
///
/// 校验：provider_id 格式 + 文件存在 + JSON 可解析 + baseUrl 为 HTTPS。
pub fn load_api_spec(provider_id: &str, endpoints_file: &str) -> Result<ApiSpec, String> {
    validate_provider_id(provider_id)?;
    let path = providers_root()
        .join(provider_id)
        .join(endpoints_file);
    if !path.exists() {
        return Err(format!(
            "厂商 endpoints.json 不存在: {}",
            path.display()
        ));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 endpoints.json 失败: {}", e))?;
    let spec: ApiSpec =
        serde_json::from_str(&content).map_err(|e| format!("解析 endpoints.json 失败: {}", e))?;

    // 安全：baseUrl 必须为 HTTPS（认证 token 经此通道传输）
    if !spec.base_url.starts_with("https://") {
        return Err(format!(
            "endpoints.json baseUrl 必须使用 HTTPS: {}",
            spec.base_url
        ));
    }

    Ok(spec)
}

/// 认证后调用厂商 API 拉取隧道列表
///
/// 流程：加载 spec → 加载 access_token → 调用 tunnels.list 端点 →
/// 按 envelope 判成功 → 按 itemsField + fields 映射为 TunnelInfo 列表。
pub async fn fetch_tunnels(
    state: &AppState,
    provider_id: &str,
) -> Result<(Vec<TunnelInfo>, AccountInfo), String> {
    let manifest = super::provider::read_provider_manifest(provider_id)?;
    let endpoints_file = manifest
        .api
        .as_ref()
        .map(|a| a.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = load_api_spec(provider_id, endpoints_file)?;

    let token = crate::commands::frp::auth::load_token(provider_id)
        .await
        .map_err(|e| format!("加载厂商 {} 的 access_token 失败: {}", provider_id, e))?;

    let device_id = crate::commands::sdk::get_device_id(state)
        .await
        .map_err(|e| format!("获取 device_id 失败: {}", e))?;

    // 拉取账号信息（可选，用于获取 account.token 供 fields 映射引用）
    let account = if let Some(ref acct_endpoint) = spec
        .endpoints
        .as_ref()
        .and_then(|e| e.account.as_ref())
    {
        log_info!(
            "[Frp] 拉取厂商 {} 账号信息: {} {}",
            provider_id,
            acct_endpoint.method,
            acct_endpoint.path
        );
        let resp = http::send_request(
            &spec.base_url,
            acct_endpoint,
            &token,
            &device_id,
            provider_id,
            spec.envelope.as_ref(),
        )
        .await?;
        map_account(&resp, acct_endpoint)?
    } else {
        AccountInfo::default()
    };

    // 拉取隧道列表
    let tunnels_endpoint = spec
        .endpoints
        .as_ref()
        .and_then(|e| e.tunnels.as_ref())
        .and_then(|t| t.list.as_ref())
        .ok_or_else(|| "endpoints.json 缺少 tunnels.list 端点定义".to_string())?;

    log_info!(
        "[Frp] 拉取厂商 {} 隧道列表: {} {}",
        provider_id,
        tunnels_endpoint.method,
        tunnels_endpoint.path
    );

    let resp = http::send_request(
        &spec.base_url,
        tunnels_endpoint,
        &token,
        &device_id,
        provider_id,
        spec.envelope.as_ref(),
    )
    .await?;

    let tunnels = map_tunnels(&resp, tunnels_endpoint, &account)?;

    log_info!(
        "[Frp] 厂商 {} 隧道列表拉取成功: {} 条",
        provider_id,
        tunnels.len()
    );

    Ok((tunnels, account))
}

/// 按 ResponseDef.fields 映射账号信息
fn map_account(
    response: &serde_json::Value,
    endpoint: &super::EndpointDef,
) -> Result<AccountInfo, String> {
    let data = envelope::extract_data(
        response,
        endpoint.envelope.as_ref(),
        endpoint.response.data_field.as_deref(),
    )?;

    let fields = &endpoint.response.fields;
    Ok(AccountInfo {
        id: get_field(data.as_ref().unwrap_or(response), fields.get("id")),
        username: get_field(data.as_ref().unwrap_or(response), fields.get("username")),
        email: get_field(data.as_ref().unwrap_or(response), fields.get("email")),
        token: get_field(data.as_ref().unwrap_or(response), fields.get("token")),
    })
}

/// 按 ResponseDef.itemsField + fields 映射隧道列表
fn map_tunnels(
    response: &serde_json::Value,
    endpoint: &super::EndpointDef,
    account: &AccountInfo,
) -> Result<Vec<TunnelInfo>, String> {
    // 提取列表项
    let items = if let Some(ref items_field) = endpoint.response.items_field {
        jsonpath::extract_array(response, items_field)?
    } else {
        // 无 itemsField 时尝试用 dataField
        let data = envelope::extract_data(
            response,
            endpoint.envelope.as_ref(),
            endpoint.response.data_field.as_deref(),
        )?;
        match data {
            Some(serde_json::Value::Array(arr)) => arr,
            Some(v) => vec![v],
            None => vec![],
        }
    };

    let fields = &endpoint.response.fields;
    let tunnels: Vec<TunnelInfo> = items
        .into_iter()
        .map(|item| {
            let item_ref = &item;
            TunnelInfo {
                id: resolve_field(item_ref, fields.get("id"), account),
                name: resolve_field(item_ref, fields.get("name"), account),
                tunnel_type: resolve_field(item_ref, fields.get("type"), account),
                status: resolve_field(item_ref, fields.get("status"), account),
                server_host: resolve_field(item_ref, fields.get("serverHost"), account),
                server_port: resolve_field(item_ref, fields.get("serverPort"), account),
                token: resolve_field(item_ref, fields.get("token"), account),
                local_host: resolve_field(item_ref, fields.get("localHost"), account),
                local_port: resolve_field(item_ref, fields.get("localPort"), account),
                remote_port: resolve_field(item_ref, fields.get("remotePort"), account),
                custom_domain: resolve_field(item_ref, fields.get("customDomain"), account),
            }
        })
        .collect();

    Ok(tunnels)
}

/// 从 JSON 对象按字段名取字符串值
fn get_field(value: &serde_json::Value, mapping: Option<&super::FieldMapping>) -> String {
    match mapping {
        Some(m) => resolve_field(value, Some(m), &AccountInfo::default()),
        None => String::new(),
    }
}

/// 解析字段映射（支持直接字段名、split 拆分、{account.token} 引用）
fn resolve_field(
    item: &serde_json::Value,
    mapping: Option<&super::FieldMapping>,
    account: &AccountInfo,
) -> String {
    let Some(m) = mapping else {
        return String::new();
    };

    // 优先处理 value（如 "{account.token}"）
    if let Some(ref val) = m.value {
        return resolve_value_template(val, account);
    }

    let Some(ref field_name) = m.field else {
        return String::new();
    };

    // 从 item 取原始值
    let raw = item
        .get(field_name)
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => v.to_string(),
        })
        .unwrap_or_default();

    // 处理 split 拆分（如 connectAddress "host:port" 按冒号拆分）
    if let Some(ref sep) = m.split {
        // 需要知道取第几部分 — 通过 field 名推断（serverHost 取 0，serverPort 取 1）
        // 但 FieldMapping 不包含索引信息，这里按约定：split 时返回完整值
        // 调用方应在 fields 映射中分别为 serverHost/serverPort 配置 split
        // 实际拆分在 config_gen 中按需处理
        return raw;
    }

    raw
}

/// 解析值模板（支持 {account.token} 等占位符）
fn resolve_value_template(template: &str, account: &AccountInfo) -> String {
    template
        .replace("{account.token}", &account.token)
        .replace("{account.id}", &account.id)
        .replace("{account.username}", &account.username)
        .replace("{account.email}", &account.email)
}
