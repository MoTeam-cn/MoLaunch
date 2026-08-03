//! API 请求执行（fetch_tunnels + 响应字段映射 + 统一隧道/账号 DTO）

use serde::Serialize;

use crate::commands::frp::api_spec::{envelope, http, jsonpath};
use crate::commands::frp::{EndpointDef, FieldMapping};
use crate::log_info;
use crate::state::AppState;

// 统一隧道数据（API 响应映射后的标准格式）

/// 隧道信息（从厂商 API 响应映射后的统一格式）
///
/// 对应 endpoints.json 中 tunnelFields 定义的字段。
/// fields 模式下启动器按这些字段拼装 frpc 配置。
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub token: String,
}

/// 认证后调用厂商 API 拉取隧道列表
///
/// 流程：加载 spec → 确保 token 有效（过期自动续期）→ 调用 tunnels.list 端点 →
/// 按 envelope 判成功 → 按 itemsField + fields 映射为 TunnelInfo 列表。
///
/// 自动续期：`ensure_valid_token` 在 token 过期且存在 refresh_token 时静默刷新；
/// 若请求仍返回 HTTP 401（token 被厂商主动吊销/刷新失败），则再次刷新并重试一次。
pub async fn fetch_tunnels(
    state: &AppState,
    provider_id: &str,
) -> Result<(Vec<TunnelInfo>, AccountInfo), String> {
    let manifest = crate::commands::frp::provider::read_provider_manifest(provider_id)?;
    let endpoints_file = manifest
        .api
        .as_ref()
        .map(|a| a.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = super::load_api_spec(provider_id, endpoints_file)?;

    let token = crate::commands::frp::auth::ensure_valid_token(state, provider_id)
        .await
        .map_err(|e| format!("厂商 {} token 校验失败: {}", provider_id, e))?;

    let device_id = crate::commands::sdk::get_device_id(state)
        .await
        .map_err(|e| format!("获取 device_id 失败: {}", e))?;

    // 拉取账号信息（可选，用于获取 account.token 供 fields 映射引用）
    let account =
        if let Some(acct_endpoint) = spec.endpoints.as_ref().and_then(|e| e.account.as_ref()) {
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

    let resp = match http::send_request(
        &spec.base_url,
        tunnels_endpoint,
        &token,
        &device_id,
        provider_id,
        spec.envelope.as_ref(),
    )
    .await
    {
        Ok(resp) => resp,
        // HTTP 401：token 失效（可能被厂商吊销或已过期刷新失败），自动刷新后重试一次
        Err(e) if is_unauthorized_err(&e) => {
            log_info!(
                "[Frp] 厂商 {} 隧道列表返回 401，刷新 token 后重试",
                provider_id
            );
            crate::commands::frp::auth::refresh_token(state, provider_id).await?;
            let new_token = crate::commands::frp::auth::ensure_valid_token(state, provider_id)
                .await
                .map_err(|e| format!("厂商 {} token 校验失败: {}", provider_id, e))?;
            http::send_request(
                &spec.base_url,
                tunnels_endpoint,
                &new_token,
                &device_id,
                provider_id,
                spec.envelope.as_ref(),
            )
            .await?
        }
        Err(e) => return Err(e),
    };

    let tunnels = map_tunnels(&resp, tunnels_endpoint, &account)?;

    log_info!(
        "[Frp] 厂商 {} 隧道列表拉取成功: {} 条",
        provider_id,
        tunnels.len()
    );

    Ok((tunnels, account))
}

/// 判断错误是否由 HTTP 401 引起（厂商 token 失效）
fn is_unauthorized_err(e: &str) -> bool {
    e.contains("HTTP 401") || e.contains("HTTP 403")
}

/// 按 ResponseDef.fields 映射账号信息
fn map_account(
    response: &serde_json::Value,
    endpoint: &EndpointDef,
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
    endpoint: &EndpointDef,
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
fn get_field(value: &serde_json::Value, mapping: Option<&FieldMapping>) -> String {
    match mapping {
        Some(m) => resolve_field(value, Some(m), &AccountInfo::default()),
        None => String::new(),
    }
}

/// 解析字段映射（支持直接字段名、split 拆分、{account.token} 引用）
fn resolve_field(
    item: &serde_json::Value,
    mapping: Option<&FieldMapping>,
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
    // 需要知道取第几部分 — 通过 field 名推断（serverHost 取 0，serverPort 取 1）
    // 但 FieldMapping 不包含索引信息，这里按约定：split 时返回完整值
    // 调用方应在 fields 映射中分别为 serverHost/serverPort 配置 split
    // 实际拆分在 config_gen 中按需处理
    if m.split.is_some() {
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