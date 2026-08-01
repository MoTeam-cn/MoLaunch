//! Device Code 授权流程
//!
//! 流程（参见设计文档 §6.4）：申请设备码 → 前端显示用户码 + 验证链接 + 倒计时 →
//! 后端按 interval 轮询 token。请求/响应解析由 flows.rs 引擎按
//! endpoints.json authFlows.device_code.request/poll 配置驱动。

use super::super::api_spec::load_api_spec;
use super::super::provider::{read_provider_manifest, resolve_device_code_config};
use super::super::types::{DeviceCodeFlow, FieldExtractor, FlowRequest};
use super::flows::{send_flow_request, FlowContext, FlowResponse};
use super::storage::{now_secs, store_token_info};
use super::{DeviceCodePollResult, DeviceCodeResult};
use crate::log_error;
use crate::log_info;
use crate::state::AppState;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// Device Code 会话存储（内存，进程级）
/// Device Code 会话（start_device_code 写入，poll_device_code 读取）
#[derive(Clone)]
struct DeviceCodeSession {
    device_code: String,
    /// 会话过期时间（Unix 秒）
    expires_at: u64,
    /// 轮询间隔（秒，存储供前端查询，poll_device_code 不直接使用）
    #[allow(dead_code)]
    interval: u64,
    /// poll 端点的 FlowRequest（含 url/body/headers/response）
    poll_flow: FlowRequest,
    /// pendingError 字符串（如 "authorization_pending"）
    pending_error: Option<String>,
    /// clientId
    client_id: String,
    /// endpoints.json baseUrl（poll flow url 模板 {baseUrl} 需要）
    base_url: String,
}

static DEVICE_CODE_SESSIONS: Lazy<Mutex<HashMap<String, DeviceCodeSession>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 移除 Device Code 会话
pub(super) fn remove_device_code_session(provider_id: &str) {
    if let Ok(mut sessions) = DEVICE_CODE_SESSIONS.lock() {
        sessions.remove(provider_id);
    }
}

/// 启动 Device Code 流程
pub(super) async fn start_device_code(
    _state: &AppState,
    provider_id: &str,
) -> Result<DeviceCodeResult, String> {
    let manifest = read_provider_manifest(provider_id)?;
    let config = resolve_device_code_config(provider_id, &manifest)?;

    // 加载 endpoints.json 取 authFlows.device_code.request/poll 配置
    let endpoints_file = manifest
        .api
        .as_ref()
        .map(|a| a.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = load_api_spec(provider_id, endpoints_file)?;
    let dc_flow: &DeviceCodeFlow = spec
        .auth_flows
        .as_ref()
        .and_then(|f| f.device_code.as_ref())
        .ok_or_else(|| {
            format!(
                "厂商 {} endpoints.json 缺少 authFlows.device_code 配置",
                provider_id
            )
        })?;
    let request_flow: &FlowRequest = &dc_flow.request;
    let poll_flow: FlowRequest = dc_flow.poll.clone();

    log_info!("[Frp Auth] 启动 Device Code 流程: provider={}", provider_id);

    // 1. 请求设备码（走 flows 引擎）
    let ctx = FlowContext {
        base_url: Some(spec.base_url.clone()),
        client_id: config.client_id.clone(),
        client_secret: config.client_secret.clone(),
        scope: Some(config.scopes.join(" ")),
        ..Default::default()
    };
    let resp = send_flow_request(request_flow, &ctx).await?;

    if !resp.is_success() {
        let err = extract_flow_error(&resp, request_flow);
        log_error!("[Frp Auth] 请求设备码失败: HTTP {} - {}", resp.status, err);
        return Err(format!("请求设备码失败: {}", err));
    }

    let device_code = resp
        .extract_field(get_extractor(request_flow, "deviceCode"))
        .ok_or("设备码响应缺少 deviceCode")?;
    let user_code = resp
        .extract_field(get_extractor(request_flow, "userCode"))
        .ok_or("设备码响应缺少 userCode")?;
    let verification_uri = resp
        .extract_field(get_extractor(request_flow, "verificationUri"))
        .ok_or("设备码响应缺少 verificationUri")?;
    let expires_in: u64 = resp
        .extract_field(get_extractor(request_flow, "expiresIn"))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(600);
    let interval: u64 = resp
        .extract_field(get_extractor(request_flow, "pollInterval"))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(config.poll_interval);

    // 2. 存入内存会话
    let expires_at = now_secs() + expires_in;
    {
        let mut sessions = DEVICE_CODE_SESSIONS
            .lock()
            .map_err(|e| format!("会话锁 poisoned: {}", e))?;
        sessions.insert(
            provider_id.to_string(),
            DeviceCodeSession {
                device_code: device_code.clone(),
                expires_at,
                interval,
                poll_flow,
                pending_error: request_flow.pending_error.clone(),
                client_id: config.client_id.clone(),
                base_url: spec.base_url.clone(),
            },
        );
    }

    log_info!(
        "[Frp Auth] Device Code 已获取: provider={}, user_code={}, expires_in={}s",
        provider_id,
        user_code,
        expires_in
    );

    Ok(DeviceCodeResult {
        user_code,
        verification_uri,
        expires_in,
        interval,
    })
}

/// 轮询 Device Code token
pub(super) async fn poll_device_code(
    _state: &AppState,
    provider_id: &str,
) -> Result<DeviceCodePollResult, String> {
    // 1. 读取会话
    let session = {
        let sessions = DEVICE_CODE_SESSIONS
            .lock()
            .map_err(|e| format!("会话锁 poisoned: {}", e))?;
        sessions.get(provider_id).cloned().ok_or_else(|| {
            format!(
                "未找到 {} 的 Device Code 会话，请先调用 start_device_code",
                provider_id
            )
        })?
    };

    // 会话已过期
    if now_secs() > session.expires_at {
        remove_device_code_session(provider_id);
        return Ok(DeviceCodePollResult {
            status: "expired".to_string(),
            expires_at: None,
            scopes: None,
        });
    }

    // 2. 轮询 token（走 flows 引擎）
    let ctx = FlowContext {
        base_url: Some(session.base_url.clone()),
        client_id: session.client_id.clone(),
        device_code: Some(session.device_code.clone()),
        ..Default::default()
    };
    let resp = send_flow_request(&session.poll_flow, &ctx).await?;

    // 3. 处理结果（HTTP 错误或 envelope 错误）
    if !resp.is_success() {
        // 检查 errorField 是否为 pending 错误
        if let Some(err) = resp.extract_field(get_extractor(&session.poll_flow, "errorField")) {
            let pending = session
                .pending_error
                .as_deref()
                .unwrap_or("authorization_pending");
            let status = match err.as_str() {
                e if e == pending => "pending",
                "expired_token" => {
                    remove_device_code_session(provider_id);
                    "expired"
                }
                "access_denied" => {
                    remove_device_code_session(provider_id);
                    "declined"
                }
                "slow_down" => "slow_down",
                other => {
                    log_error!("[Frp Auth] 未知 device code 错误: {}", other);
                    remove_device_code_session(provider_id);
                    return Err(format!("设备码授权失败: {}", other));
                }
            };
            return Ok(DeviceCodePollResult {
                status: status.to_string(),
                expires_at: None,
                scopes: None,
            });
        }

        let err = extract_flow_error(&resp, &session.poll_flow);
        log_error!("[Frp Auth] 轮询 token 失败: HTTP {} - {}", resp.status, err);
        return Err(format!("轮询 token 失败: {}", err));
    }

    // 4. 成功 -> 提取并存储 token
    let access_token = resp
        .extract_field(get_extractor(&session.poll_flow, "accessToken"))
        .ok_or("token 响应缺少 accessToken")?;
    let refresh_token = resp.extract_field(get_extractor(&session.poll_flow, "refreshToken"));
    let expires_in = resp
        .extract_field(get_extractor(&session.poll_flow, "expiresIn"))
        .and_then(|s| s.parse::<u64>().ok());
    let expires_at = expires_in.map(|secs| now_secs() + secs);

    // 读取 auth.json 中的 scopes 作为存储值
    let manifest = read_provider_manifest(provider_id)?;
    let config = resolve_device_code_config(provider_id, &manifest)?;
    let scopes = config.scopes.clone();

    store_token_info(
        provider_id,
        &access_token,
        refresh_token.as_deref(),
        expires_in,
        Some(&scopes),
    )
    .await?;
    remove_device_code_session(provider_id);

    log_info!(
        "[Frp Auth] Device Code 认证成功: provider={}, expires_at={:?}",
        provider_id,
        expires_at
    );

    Ok(DeviceCodePollResult {
        status: "success".to_string(),
        expires_at,
        scopes: Some(scopes),
    })
}

// 内部辅助
/// 从 FlowRequest.response 取指定字段的 FieldExtractor
///
/// 字段名按 camelCase 约定：deviceCode / userCode / verificationUri / pollInterval /
/// expiresIn / accessToken / refreshToken / errorField / errorDescription
fn get_extractor<'a>(flow: &'a FlowRequest, key: &str) -> &'a FieldExtractor {
    static EMPTY: Lazy<FieldExtractor> = Lazy::new(|| FieldExtractor {
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
