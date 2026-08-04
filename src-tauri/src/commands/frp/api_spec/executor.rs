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
    /// 隧道显示名（厂商返回的 remark 等，用户可读的名字；name 多为真实隧道 id）
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
    /// 厂商 config 端点返回的完整 frpc 配置（已解码），导入时原样持久化。
    pub raw_config: Option<String>,
    /// 厂商 detail/list 等接口返回的原始数据，用于声明式配置字段映射。
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
                "",
                "",
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
        "",
        "",
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
                "",
                "",
                spec.envelope.as_ref(),
            )
            .await?
        }
        Err(e) => return Err(e),
    };

    let mut tunnels = map_tunnels(&resp, tunnels_endpoint, &account)?;

    // 隧道详情/配置回填：部分厂商的列表/详情接口不含完整字段
    // （frps 连接端口 server_port、远端端口 remote_port 等，如 Lolia 的
    // frps 端口只在 config 接口返回的配置里），需调用 detail/config 端点补全。
    // 优先 config 端点（frpc 配置中含 serverAddr/serverPort/remotePort），
    // 其次 detail 端点。
    if let Some(tunnels_def) = spec.endpoints.as_ref().and_then(|e| e.tunnels.as_ref()) {
        let config_endpoint = tunnels_def.config.as_ref();
        let detail_endpoint = tunnels_def.detail.as_ref();
        for t in tunnels.iter_mut() {
            // 即使列表字段完整，只要有 config 端点也要获取完整原文并持久化。
            // detail 端点仅在仍有字段缺失时调用。
            if let Some(cfg_ep) = config_endpoint {
                match fetch_tunnel_config(
                    &spec.base_url,
                    cfg_ep,
                    &t.id,
                    &t.name,
                    &token,
                    &device_id,
                    provider_id,
                    spec.envelope.as_ref(),
                )
                .await
                {
                    Ok(cfg) => {
                        let (addr, port, remote) = parse_config_fields(&cfg);
                        t.raw_config = Some(cfg);
                        if t.server_host.is_empty() {
                            if let Some(addr) = addr {
                                t.server_host = addr;
                            }
                        }
                        if t.server_port.is_empty() {
                            if let Some(port) = port {
                                t.server_port = port;
                            }
                        }
                        if t.remote_port.is_empty() {
                            if let Some(remote) = remote {
                                t.remote_port = remote;
                            }
                        }
                    }
                    Err(e) => {
                        log_info!(
                            "[Frp] 厂商 {} config 端点获取失败，继续使用字段映射: {}",
                            provider_id,
                            e
                        );
                    }
                }
            }

            // detail 端点回填其余缺失字段（server_host/server_port/remote_port 等）
            if let Some(detail_ep) = detail_endpoint {
                let still_missing = t.remote_port.is_empty()
                    || t.server_host.is_empty()
                    || t.server_port.is_empty()
                    || t.token.is_empty()
                    || t.local_port.is_empty();
                if !still_missing {
                    continue;
                }
                if let Some(detail_item) = fetch_tunnel_detail(
                    &spec.base_url,
                    detail_ep,
                    t,
                    &token,
                    &device_id,
                    provider_id,
                    spec.envelope.as_ref(),
                )
                .await
                {
                    let detail_fields = &detail_ep.response.fields;
                    if t.remote_port.is_empty() {
                        t.remote_port = resolve_field(
                            &detail_item,
                            detail_fields.get("remotePort"),
                            &account,
                            None,
                        );
                    }
                    if t.server_host.is_empty() {
                        t.server_host = resolve_field(
                            &detail_item,
                            detail_fields.get("serverHost"),
                            &account,
                            Some(0),
                        );
                    }
                    if t.server_port.is_empty() {
                        t.server_port = resolve_field(
                            &detail_item,
                            detail_fields.get("serverPort"),
                            &account,
                            Some(1),
                        );
                    }
                    if t.token.is_empty() {
                        t.token =
                            resolve_field(&detail_item, detail_fields.get("token"), &account, None);
                    }
                    if t.local_port.is_empty() {
                        t.local_port = resolve_field(
                            &detail_item,
                            detail_fields.get("localPort"),
                            &account,
                            None,
                        );
                    }
                }
            }
        }
    }

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

/// 调用厂商 config 端点获取 frpc 配置字符串（已按 encoding 解码）
///
/// 流程：发送 config 端点请求 → 按 dataField 提取配置字段 → 按 encoding
/// 解码（text 原样 / base64 解码）。失败返回 Err（调用方忽略，回填兜底）。
#[allow(clippy::too_many_arguments)]
async fn fetch_tunnel_config(
    base_url: &str,
    endpoint: &EndpointDef,
    tunnel_id: &str,
    tunnel_name: &str,
    token: &str,
    device_id: &str,
    provider_id: &str,
    global_envelope: Option<&crate::commands::frp::Envelope>,
) -> Result<String, String> {
    let resp = http::send_request(
        base_url,
        endpoint,
        token,
        device_id,
        provider_id,
        tunnel_id,
        tunnel_name,
        global_envelope,
    )
    .await?;

    let raw = envelope::extract_data(
        &resp,
        endpoint.envelope.as_ref(),
        endpoint.response.data_field.as_deref(),
    )?
    .ok_or_else(|| "config 端点响应缺少 dataField".to_string())?;

    let raw_str = match raw {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
    };

    let encoding = endpoint.response.encoding.as_deref();
    let decoded = super::config_gen::decode_config(&raw_str, encoding)?;
    log_info!(
        "[Frp] 厂商 {} config 端点已获取配置（encoding={:?}，长度={}）",
        provider_id,
        encoding,
        decoded.len()
    );
    Ok(decoded)
}

/// 拉取厂商 config 端点返回的原版 frpc 配置（启动隧道前调用）
///
/// 供 `process::start_tunnel` 使用：厂商配置了 `tunnels.config` 端点时，
/// 优先用其返回的原版配置启动（叠加逆向字段），而非本地拼装。
/// 未配置 config 端点或拉取失败时返回 Err，调用方回退本地生成。
pub async fn fetch_raw_tunnel_config(
    state: &crate::state::AppState,
    provider_id: &str,
    tunnel_id: &str,
    tunnel_name: &str,
) -> Result<String, String> {
    let manifest = crate::commands::frp::provider::read_provider_manifest(provider_id)?;
    let endpoints_file = manifest
        .api
        .as_ref()
        .map(|a| a.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = super::load_api_spec(provider_id, endpoints_file)?;

    let config_endpoint = spec
        .endpoints
        .as_ref()
        .and_then(|e| e.tunnels.as_ref())
        .and_then(|t| t.config.as_ref())
        .ok_or_else(|| format!("厂商 {} 未配置 config 端点", provider_id))?;

    let token = crate::commands::frp::auth::ensure_valid_token(state, provider_id)
        .await
        .map_err(|e| format!("厂商 {} token 校验失败: {}", provider_id, e))?;
    let device_id = crate::commands::sdk::get_device_id(state)
        .await
        .map_err(|e| format!("获取 device_id 失败: {}", e))?;

    fetch_tunnel_config(
        &spec.base_url,
        config_endpoint,
        tunnel_id,
        tunnel_name,
        &token,
        &device_id,
        provider_id,
        spec.envelope.as_ref(),
    )
    .await
}

/// 调用厂商 detail 端点获取单个隧道详情（返回原始 JSON 项）
///
/// pathParams 替换隧道字段（如 {tunnelId} → t.id）。失败返回 None（调用方兜底）。
async fn fetch_tunnel_detail(
    base_url: &str,
    endpoint: &EndpointDef,
    tunnel: &TunnelInfo,
    token: &str,
    device_id: &str,
    provider_id: &str,
    global_envelope: Option<&crate::commands::frp::Envelope>,
) -> Option<serde_json::Value> {
    // 构造带路径参数的端点副本
    let mut ep = endpoint.clone();
    let mut path = ep.path.clone();
    for (placeholder, field) in &ep.path_params {
        let value = match field.as_str() {
            "id" => &tunnel.id,
            "name" => &tunnel.name,
            _ => continue,
        };
        path = path.replace(&format!("{{{}}}", placeholder), value);
    }
    ep.path = path;

    match http::send_request(
        base_url,
        &ep,
        token,
        device_id,
        provider_id,
        &tunnel.id,
        &tunnel.name,
        global_envelope,
    )
    .await
    {
        Ok(resp) => {
            let data = envelope::extract_data(
                &resp,
                ep.envelope.as_ref(),
                ep.response.data_field.as_deref(),
            )
            .ok()
            .flatten();
            log_info!(
                "[Frp] 厂商 {} detail 端点已获取隧道 {}",
                provider_id,
                tunnel.id
            );
            Some(data.unwrap_or(resp))
        }
        Err(e) => {
            log_info!(
                "[Frp] 厂商 {} detail 端点获取失败（忽略，继续用列表字段）: {}",
                provider_id,
                e
            );
            None
        }
    }
}

/// 从 frpc 配置文本解析服务器连接与远端端口字段
///
/// 返回 `(server_addr, server_port, remote_port)`，支持两种格式：
///
/// - TOML（frpc v0.51+）：`serverAddr = 'hk-6.qwq.fan'`、`serverPort = 17000`、`remotePort = 30919`
/// - INI（旧版 frpc）：`server_addr = ...`、`server_port = ...`、`remote_port = ...`
///
/// 取第一个匹配的字段值；未匹配返回 None。
fn parse_config_fields(config: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut server_addr = None;
    let mut server_port = None;
    let mut remote_port = None;

    for line in config.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let Some(eq) = line.find('=') else {
            continue;
        };
        let key = line[..eq].trim();
        let value = line[eq + 1..].trim().trim_matches('"').trim_matches('\'');
        match key {
            "serverAddr" | "server_addr" => {
                if server_addr.is_none() && !value.is_empty() {
                    server_addr = Some(value.to_string());
                }
            }
            "serverPort" | "server_port" => {
                if server_port.is_none() && value.parse::<u16>().is_ok() {
                    server_port = Some(value.to_string());
                }
            }
            "remotePort" | "remote_port"
                if remote_port.is_none() && value.parse::<u16>().is_ok() =>
            {
                remote_port = Some(value.to_string());
            }
            _ => {}
        }
    }

    (server_addr, server_port, remote_port)
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
                id: resolve_field(item_ref, fields.get("id"), account, None),
                name: resolve_field(item_ref, fields.get("name"), account, None),
                remark: resolve_field(item_ref, fields.get("remark"), account, None),
                tunnel_type: resolve_field(item_ref, fields.get("type"), account, None),
                status: resolve_field(item_ref, fields.get("status"), account, None),
                server_host: resolve_field(item_ref, fields.get("serverHost"), account, Some(0)),
                server_port: resolve_field(item_ref, fields.get("serverPort"), account, Some(1)),
                token: resolve_field(item_ref, fields.get("token"), account, None),
                local_host: resolve_field(item_ref, fields.get("localHost"), account, None),
                local_port: resolve_field(item_ref, fields.get("localPort"), account, None),
                remote_port: resolve_field(item_ref, fields.get("remotePort"), account, None),
                custom_domain: resolve_field(item_ref, fields.get("customDomain"), account, None),
                raw_config: None,
                source_data: item,
            }
        })
        .collect();

    Ok(tunnels)
}

/// 从 JSON 对象按字段名取字符串值
fn get_field(value: &serde_json::Value, mapping: Option<&FieldMapping>) -> String {
    match mapping {
        Some(m) => resolve_field(value, Some(m), &AccountInfo::default(), None),
        None => String::new(),
    }
}

/// 解析字段映射（支持直接字段名、split 拆分、{account.token} 引用）
///
/// `split_index`：当映射配置了 split 分隔符时，取拆分后第几段
/// （如 connectAddress "host:port" 拆分后 serverHost 取第 0 段、serverPort 取第 1 段）。
fn resolve_field(
    item: &serde_json::Value,
    mapping: Option<&FieldMapping>,
    account: &AccountInfo,
    split_index: Option<usize>,
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
    // 配置了 split 时按 split_index 取对应段；未配置 split 或取不到时返回完整值
    if let Some(ref sep) = m.split {
        if let Some(idx) = split_index {
            let parts: Vec<&str> = raw.split(sep).collect();
            if let Some(part) = parts.get(idx) {
                return part.trim().to_string();
            }
        }
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
