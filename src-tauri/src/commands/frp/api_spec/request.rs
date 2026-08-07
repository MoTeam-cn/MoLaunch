//! 隧道配置与详情端点请求。

use super::TunnelInfo;
use crate::commands::frp::api_spec::{envelope, http};
use crate::commands::frp::{EndpointDef, Envelope};
use crate::log_info;
use crate::state::AppState;

#[allow(clippy::too_many_arguments)]
pub(super) async fn fetch_tunnel_config(
    base_url: &str,
    endpoint: &EndpointDef,
    tunnel_id: &str,
    tunnel_name: &str,
    token: &str,
    device_id: &str,
    provider_id: &str,
    global_envelope: Option<&Envelope>,
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
        serde_json::Value::String(value) => value,
        value => value.to_string(),
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

pub async fn fetch_raw_tunnel_config(
    state: &AppState,
    provider_id: &str,
    tunnel_id: &str,
    tunnel_name: &str,
) -> Result<String, String> {
    let manifest = crate::commands::frp::provider::read_provider_manifest(provider_id)?;
    let endpoints_file = manifest
        .api
        .as_ref()
        .map(|api| api.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = super::load_api_spec(provider_id, endpoints_file)?;
    let config_endpoint = spec
        .endpoints
        .as_ref()
        .and_then(|endpoints| endpoints.tunnels.as_ref())
        .and_then(|tunnels| tunnels.config.as_ref())
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

pub(super) async fn fetch_tunnel_detail(
    base_url: &str,
    endpoint: &EndpointDef,
    tunnel: &TunnelInfo,
    token: &str,
    device_id: &str,
    provider_id: &str,
    global_envelope: Option<&Envelope>,
) -> Option<serde_json::Value> {
    let mut endpoint = endpoint.clone();
    for (placeholder, field) in &endpoint.path_params {
        let value = match field.as_str() {
            "id" => &tunnel.id,
            "name" => &tunnel.name,
            _ => continue,
        };
        endpoint.path = endpoint
            .path
            .replace(&format!("{{{}}}", placeholder), value);
    }
    match http::send_request(
        base_url,
        &endpoint,
        token,
        device_id,
        provider_id,
        &tunnel.id,
        &tunnel.name,
        global_envelope,
    )
    .await
    {
        Ok(response) => {
            let data = envelope::extract_data(
                &response,
                endpoint.envelope.as_ref(),
                endpoint.response.data_field.as_deref(),
            )
            .ok()
            .flatten();
            log_info!(
                "[Frp] 厂商 {} detail 端点已获取隧道 {}",
                provider_id,
                tunnel.id
            );
            Some(data.unwrap_or(response))
        }
        Err(error) => {
            log_info!(
                "[Frp] 厂商 {} detail 端点获取失败（忽略，继续用列表字段）: {}",
                provider_id,
                error
            );
            None
        }
    }
}
