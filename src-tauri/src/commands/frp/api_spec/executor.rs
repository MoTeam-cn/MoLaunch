//! API 请求执行（请求编排与隧道配置回填）

use crate::commands::frp::api_spec::{dto, http, request};
use crate::log_info;
use crate::state::AppState;

pub use dto::{AccountInfo, TunnelInfo};

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
            dto::map_account(&resp, acct_endpoint)?
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

    let mut tunnels = dto::map_tunnels(&resp, tunnels_endpoint, &account)?;

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
                match request::fetch_tunnel_config(
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
                        let (addr, port, remote) = dto::parse_config_fields(&cfg);
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
                if let Some(detail_item) = request::fetch_tunnel_detail(
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
                        t.remote_port = dto::resolve_field(
                            &detail_item,
                            detail_fields.get("remotePort"),
                            &account,
                            None,
                        );
                    }
                    if t.server_host.is_empty() {
                        t.server_host = dto::resolve_field(
                            &detail_item,
                            detail_fields.get("serverHost"),
                            &account,
                            Some(0),
                        );
                    }
                    if t.server_port.is_empty() {
                        t.server_port = dto::resolve_field(
                            &detail_item,
                            detail_fields.get("serverPort"),
                            &account,
                            Some(1),
                        );
                    }
                    if t.token.is_empty() {
                        t.token = dto::resolve_field(
                            &detail_item,
                            detail_fields.get("token"),
                            &account,
                            None,
                        );
                    }
                    if t.local_port.is_empty() {
                        t.local_port = dto::resolve_field(
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
