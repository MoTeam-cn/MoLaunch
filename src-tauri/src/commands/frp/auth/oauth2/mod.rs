//! OAuth2 授权流程
//!
//! 流程（参见设计文档 §6.3）：本地启动 HTTP 服务监听 redirectPort 接收回调，
//! 浏览器跳转走 `crate::minecraft::system::shell::open_url`，
//! token 交换请求/响应解析由 flows.rs 引擎按 endpoints.json authFlows.oauth2.token 配置驱动。
//! 授权 URL 构建与回调 HTTP 处理在 `exchange` 子模块。

mod exchange;

use super::super::api_spec::load_api_spec;
use super::super::log_redact::redact_log;
use super::super::provider::{read_provider_manifest, resolve_oauth2_config};
use super::super::types::{FieldExtractor, FlowRequest, OAuth2Flow};
use super::flows::{send_flow_request, FlowContext, FlowResponse};
use super::storage::{generate_state, now_secs, store_token_info};
use super::OAuth2Result;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::state::AppState;

/// 启动 OAuth2 授权流程
pub(super) async fn start_oauth2(
    _state: &AppState,
    provider_id: &str,
) -> Result<OAuth2Result, String> {
    let manifest = read_provider_manifest(provider_id)?;
    let config = resolve_oauth2_config(provider_id, &manifest)?;

    // 加载 endpoints.json 取 authFlows.oauth2.token 配置
    let endpoints_file = manifest
        .api
        .as_ref()
        .map(|a| a.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = load_api_spec(provider_id, endpoints_file)?;
    let oauth2_flow: &OAuth2Flow = spec
        .auth_flows
        .as_ref()
        .and_then(|f| f.oauth2.as_ref())
        .ok_or_else(|| {
            format!(
                "厂商 {} endpoints.json 缺少 authFlows.oauth2 配置",
                provider_id
            )
        })?;
    let token_flow: &FlowRequest = &oauth2_flow.token;

    log_info!("[Frp Auth] 启动 OAuth2 流程: provider={}", provider_id);

    // 1. 启动本地 HTTP 服务接收 callback
    let bind_addr = format!("127.0.0.1:{}", config.redirect_port);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| format!("绑定回调端口 {} 失败: {}", config.redirect_port, e))?;
    log_debug!("[Frp Auth] 回调服务监听: {}", bind_addr);

    let redirect_uri = format!("http://localhost:{}", config.redirect_port);

    // 2. 生成 state（CSRF 防护）并构建授权 URL
    let state = generate_state();
    let authorize_url = exchange::build_authorize_url(
        &config.authorize_url,
        &config.client_id,
        &redirect_uri,
        &config.scopes,
        &state,
    );

    // 3. 打开浏览器（走 shell 模块）
    crate::minecraft::system::shell::open_url(&authorize_url)
        .map_err(|e| format!("打开浏览器失败: {}", e))?;

    // 4. 等待回调（5 分钟超时）
    let callback = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        exchange::wait_for_callback(&listener, &state),
    )
    .await
    .map_err(|_| {
        log_error!("[Frp Auth] OAuth2 回调超时: provider={}", provider_id);
        "OAuth2 授权超时（5 分钟内未完成）".to_string()
    })??;

    // 5. 用 code 换取 token（走 flows 引擎）
    let ctx = FlowContext {
        base_url: Some(spec.base_url.clone()),
        client_id: config.client_id.clone(),
        client_secret: config.client_secret.clone(),
        redirect_uri: Some(redirect_uri.clone()),
        code: Some(callback.code),
        scope: Some(config.scopes.join(" ")),
        ..Default::default()
    };
    let resp = send_flow_request(token_flow, &ctx).await?;

    if !resp.is_success() {
        let err = extract_flow_error(&resp, token_flow);
        log_error!(
            "[Frp Auth] OAuth2 token 交换失败: HTTP {} - {}",
            resp.status,
            err
        );
        return Err(format!("OAuth2 token 交换失败: {}", err));
    }

    let access_token = resp
        .extract_field(get_extractor(token_flow, "accessToken"))
        .ok_or_else(|| {
            log_error!(
                "[Frp Auth] OAuth2 响应缺少 access_token: HTTP {} - {}",
                resp.status,
                redact_log(&resp.body)
            );
            format!(
                "OAuth2 响应缺少 access_token（HTTP {}，响应: {}）",
                resp.status,
                redact_log(&resp.body)
            )
        })?;
    let refresh_token = resp.extract_field(get_extractor(token_flow, "refreshToken"));
    let expires_in = resp
        .extract_field(get_extractor(token_flow, "expiresIn"))
        .and_then(|s| s.parse::<u64>().ok());
    let scopes = config.scopes.clone();

    // 6. 存储 token
    let expires_at = expires_in.map(|secs| now_secs() + secs);
    store_token_info(
        provider_id,
        &access_token,
        refresh_token.as_deref(),
        expires_in,
        Some(&scopes),
    )
    .await?;

    log_info!(
        "[Frp Auth] OAuth2 认证成功: provider={}, expires_at={:?}",
        provider_id,
        expires_at
    );

    Ok(OAuth2Result {
        expires_at,
        scopes: Some(scopes),
    })
}

/// 从 FlowRequest.response 取指定字段的 FieldExtractor
///
/// 字段名按 camelCase 约定：accessToken / refreshToken / expiresIn / errorField / errorDescription
fn get_extractor<'a>(flow: &'a FlowRequest, key: &str) -> &'a FieldExtractor {
    static EMPTY: once_cell::sync::Lazy<FieldExtractor> =
        once_cell::sync::Lazy::new(|| FieldExtractor {
            from: "body".to_string(),
            path: None,
            name: None,
        });
    flow.response.get(key).unwrap_or(&EMPTY)
}

/// 从响应中提取错误消息（按 errorField / errorDescription 提取）
fn extract_flow_error(resp: &FlowResponse, flow: &FlowRequest) -> String {
    let err = resp.extract_field(get_extractor(flow, "errorField"));
    let desc = resp.extract_field(get_extractor(flow, "errorDescription"));
    match (err, desc) {
        (Some(e), Some(d)) if !e.is_empty() && !d.is_empty() => format!("{}: {}", e, d),
        (Some(e), _) if !e.is_empty() => e,
        (Some(e), _) => e,
        _ => "未知错误".to_string(),
    }
}

/// OAuth2 回调解析结果
struct OAuth2Callback {
    code: String,
}