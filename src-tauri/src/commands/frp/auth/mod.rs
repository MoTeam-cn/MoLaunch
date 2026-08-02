//! Frp 厂商认证模块：OAuth2 / Device Code / API Key 三种流程
//!
//! token 经 SDK 内置 DES 加密后存文件（`<base_dir>/frp/auth/{provider_id}.json`）。
//! 子模块：storage（加密存储辅助）/ oauth2 / device_code / api_key / flows（可配置流程引擎）。

use super::api_spec::load_api_spec;
use super::provider::{
    read_provider_manifest, resolve_auth_type, resolve_device_code_config, resolve_oauth2_config,
    SYSTEM_DEFAULT_ID,
};
use super::types::{FieldExtractor, FlowRequest, OAuth2Flow};
use crate::log_info;
use crate::sdk::SdkInstance;
use crate::state::AppState;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

mod api_key;
mod device_code;
mod flows;
mod oauth2;
mod storage;

/// 注入 SDK 引用（lib.rs 启动时调用，供 token 加密存储使用）
pub fn set_sdk(sdk: Arc<TokioMutex<Option<SdkInstance>>>) {
    storage::set_sdk(sdk);
}

// 返回类型
/// 认证状态（get_auth_status 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub provider_id: String,
    /// 是否已认证（有有效 token）
    pub authenticated: bool,
    /// 认证类型：none / oauth2 / device_code / api_key
    pub auth_type: String,
    /// token 过期时间（Unix 秒），已过期时仍返回供前端展示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    /// 续期中：token 已过期但存在 refresh_token，正在静默续期
    #[serde(default, skip_serializing_if = "is_false")]
    pub refreshing: bool,
}

/// serde 辅助：bool 默认值
fn is_false(b: &bool) -> bool {
    !*b
}

/// OAuth2 流程结果（start_oauth2 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Result {
    /// token 过期时间（Unix 秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// Device Code 流程启动结果（start_device_code 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResult {
    /// 用户码（前端显示给用户输入）
    pub user_code: String,
    /// 验证链接（用户访问此 URL 输入用户码）
    pub verification_uri: String,
    /// 过期时间（秒）
    pub expires_in: u64,
    /// 轮询间隔（秒）
    pub interval: u64,
}

/// Device Code 轮询结果（poll_device_code 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodePollResult {
    /// 状态：pending / success / expired / declined / slow_down
    pub status: String,
    /// token 过期时间（仅 status=success 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围（仅 status=success 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

// 内部辅助（refresh_token 用 flows 引擎提取字段）
/// 从 FlowRequest.response 取指定字段的 FieldExtractor
///
/// 字段名按 camelCase 约定：accessToken / refreshToken / expiresIn /
/// errorField / errorDescription
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
fn extract_flow_error(resp: &flows::FlowResponse, flow: &FlowRequest) -> String {
    let err = resp.extract_field(get_extractor(flow, "errorField"));
    let desc = resp.extract_field(get_extractor(flow, "errorDescription"));
    match (err, desc) {
        (Some(e), Some(d)) if !e.is_empty() && !d.is_empty() => format!("{}: {}", e, d),
        (Some(e), _) if !e.is_empty() => e,
        (Some(e), _) => e,
        _ => "未知错误".to_string(),
    }
}

/// 查询指定厂商的认证状态
///
/// - auth_type=none：始终 authenticated=true
/// - auth_type=oauth2/device_code：检查 access_token 是否存在且未过期
///   - 已过期且存在 refresh_token → 自动续期（成功则 authenticated=true，失败则 refreshing=true）
/// - auth_type=api_key：检查 access_token（即 API Key）是否存在
///
/// expires_at 即使已过期也会返回，前端据此区分「即将过期」/「已过期」。
pub async fn get_auth_status(state: &AppState, provider_id: &str) -> Result<AuthStatus, String> {
    // 系统默认厂商无需认证
    if provider_id == SYSTEM_DEFAULT_ID {
        return Ok(AuthStatus {
            provider_id: provider_id.to_string(),
            authenticated: true,
            auth_type: "none".to_string(),
            expires_at: None,
            scopes: None,
            refreshing: false,
        });
    }

    let manifest = read_provider_manifest(provider_id)?;
    let auth_type = resolve_auth_type(provider_id, &manifest);

    if auth_type == "none" {
        return Ok(AuthStatus {
            provider_id: provider_id.to_string(),
            authenticated: true,
            auth_type,
            expires_at: None,
            scopes: None,
            refreshing: false,
        });
    }

    // 检查 access_token 是否存在（SDK DES 解密读取）
    let record = storage::load_token_record(provider_id).await?;
    let authenticated = record.is_some();

    // 检查是否过期（仅 oauth2 / device_code 有过期时间）
    let expires_at = if matches!(auth_type.as_str(), "oauth2" | "device_code") {
        record.as_ref().and_then(|r| r.expires_at)
    } else {
        None
    };

    // token 存在但已过期 -> authenticated=false
    let (authenticated, refreshing) = if authenticated {
        match expires_at {
            Some(exp) if exp <= storage::now_secs() => {
                // 已过期：有 refresh_token 则尝试静默续期
                let has_refresh = record
                    .as_ref()
                    .and_then(|r| r.refresh_token.as_ref())
                    .is_some();
                if has_refresh {
                    match refresh_token(state, provider_id).await {
                        Ok(()) => {
                            crate::log_info!(
                                "[Frp Auth] get_auth_status 自动续期成功: provider={}",
                                provider_id
                            );
                            (true, false)
                        }
                        Err(e) => {
                            crate::log_warn!(
                                "[Frp Auth] get_auth_status 自动续期失败: provider={} - {}",
                                provider_id,
                                e
                            );
                            (false, true)
                        }
                    }
                } else {
                    (false, false)
                }
            }
            Some(exp) => (exp > storage::now_secs(), false),
            None => (true, false), // api_key 无过期时间
        }
    } else {
        (false, false)
    };

    // 续期成功后重新读取记录，返回新 token 的过期时间（避免前端显示旧的过期时间）
    let (expires_at, scopes) = if authenticated {
        let new_record = storage::load_token_record(provider_id).await?;
        match new_record {
            Some(r) => {
                let exp = if matches!(auth_type.as_str(), "oauth2" | "device_code") {
                    r.expires_at
                } else {
                    None
                };
                (exp, r.scopes.clone())
            }
            None => (expires_at, record.as_ref().and_then(|r| r.scopes.clone())),
        }
    } else {
        (expires_at, record.as_ref().and_then(|r| r.scopes.clone()))
    };

    Ok(AuthStatus {
        provider_id: provider_id.to_string(),
        authenticated,
        auth_type,
        expires_at,
        scopes,
        refreshing,
    })
}

/// 启动 OAuth2 授权流程
///
/// 流程：启动本地 HTTP 服务监听 redirectPort → 打开浏览器跳转授权页 →
/// 等待回调 → 用 code 换取 token → 存储 token 到 OS 密钥存储。
pub async fn start_oauth2(state: &AppState, provider_id: &str) -> Result<OAuth2Result, String> {
    oauth2::start_oauth2(state, provider_id).await
}

/// 启动 Device Code 流程
///
/// 流程：POST deviceCodeUrl 获取设备码 → 返回用户码 + 验证链接 + 倒计时 →
/// 将 device_code 存入内存会话（供 poll_device_code 使用）。
pub async fn start_device_code(
    state: &AppState,
    provider_id: &str,
) -> Result<DeviceCodeResult, String> {
    device_code::start_device_code(state, provider_id).await
}

/// 轮询 Device Code token
///
/// 前端按 interval 调用，后端用 device_code 向 tokenUrl 发起请求：
/// pending → 继续轮询；success → 存储 token；expired/declined → 终止；slow_down → 增大间隔。
pub async fn poll_device_code(
    state: &AppState,
    provider_id: &str,
) -> Result<DeviceCodePollResult, String> {
    device_code::poll_device_code(state, provider_id).await
}

/// 刷新 token
///
/// access_token 过期前 5 分钟用 refresh_token 刷新。也可由用户手动触发。
///
/// 流程：加载 endpoints.json → 取 authFlows.oauth2.refresh（不存在时回退到 oauth2.token）
/// → 走 flows 引擎发送请求 → 解析响应并存储新 token。
pub async fn refresh_token(_state: &AppState, provider_id: &str) -> Result<(), String> {
    let manifest = read_provider_manifest(provider_id)?;
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
    // 优先使用 refresh，缺失时回退到 token（部分厂商用同一端点刷新）
    let refresh_flow: &FlowRequest = oauth2_flow.refresh.as_ref().unwrap_or(&oauth2_flow.token);

    let refresh_token = storage::load_token_record(provider_id)
        .await?
        .and_then(|r| r.refresh_token)
        .ok_or_else(|| format!("厂商 {} 无 refresh_token，请重新认证", provider_id))?;

    // 取 clientId/clientSecret（从 auth.json 按 authType 读取，oauth2 或 device_code 必居其一）
    let auth_type = resolve_auth_type(provider_id, &manifest);
    let (client_id, client_secret) = match auth_type.as_str() {
        "oauth2" => {
            let cfg = resolve_oauth2_config(provider_id, &manifest)?;
            (cfg.client_id, cfg.client_secret)
        }
        "device_code" => {
            let cfg = resolve_device_code_config(provider_id, &manifest)?;
            (cfg.client_id, cfg.client_secret)
        }
        _ => return Err(format!("厂商 {} 不支持 token 刷新", provider_id)),
    };

    log_info!("[Frp Auth] 刷新 token: provider={}", provider_id);

    let ctx = flows::FlowContext {
        base_url: Some(spec.base_url.clone()),
        client_id,
        client_secret,
        refresh_token: Some(refresh_token),
        ..Default::default()
    };
    let resp = flows::send_flow_request(refresh_flow, &ctx).await?;

    if !resp.is_success() {
        let err = extract_flow_error(&resp, refresh_flow);
        crate::log_error!("[Frp Auth] 刷新 token 失败: HTTP {} - {}", resp.status, err);
        return Err(format!("刷新 token 失败: {}", err));
    }

    let access_token = resp
        .extract_field(get_extractor(refresh_flow, "accessToken"))
        .ok_or("刷新响应缺少 accessToken")?;
    let new_refresh_token = resp.extract_field(get_extractor(refresh_flow, "refreshToken"));
    let expires_in = resp
        .extract_field(get_extractor(refresh_flow, "expiresIn"))
        .and_then(|s| s.parse::<u64>().ok());

    storage::store_token_info(
        provider_id,
        &access_token,
        new_refresh_token.as_deref(),
        expires_in,
        None, // 刷新不改变 scopes
    )
    .await?;

    log_info!("[Frp Auth] token 刷新成功: provider={}", provider_id);
    Ok(())
}

/// 撤销认证（删除所有存储的 token）
pub async fn revoke_auth(provider_id: &str) -> Result<(), String> {
    log_info!("[Frp Auth] 撤销认证: provider={}", provider_id);

    // 删除加密 token 文件
    storage::delete_provider_auth(provider_id).await?;

    // 清除 Device Code 会话
    device_code::remove_device_code_session(provider_id);

    Ok(())
}

/// 读取 access_token（供 api_spec 模块调用厂商 API 时使用）
///
/// 仅读取已存储的 access_token，不检查过期、不自动刷新。
/// 调用方（api_spec::fetch_tunnels）应先调用 [`ensure_valid_token`] 确保有效。
pub async fn load_token(provider_id: &str) -> Result<String, String> {
    storage::load_token_record(provider_id)
        .await?
        .map(|r| r.access_token)
        .ok_or_else(|| format!("厂商 {} 未认证，请先完成认证", provider_id))
}

/// 确保厂商 token 有效（过期时自动刷新）
///
/// 供调用厂商 API 前统一使用：
/// 1. 无 token 记录 → 返回未认证错误（调用方提示用户先认证）
/// 2. token 未过期 → 直接返回
/// 3. token 已过期且存在 refresh_token → 自动调用 [`refresh_token`] 刷新后返回新 token
/// 4. token 已过期但无 refresh_token（api_key / 厂商未下发）→ 返回过期错误
///
/// 由 `get_auth_status` 与 `fetch_tunnels` 共用，保证「认证中心」与
/// 「拉取隧道列表」两条链路都能自动续期。
pub async fn ensure_valid_token(state: &AppState, provider_id: &str) -> Result<String, String> {
    let record = storage::load_token_record(provider_id).await?;
    let Some(record) = record else {
        return Err(format!("厂商 {} 未认证，请先完成认证", provider_id));
    };

    // token 未过期 → 直接返回
    let valid = match record.expires_at {
        Some(exp) => exp > storage::now_secs(),
        None => true, // api_key / 无过期时间字段
    };
    if valid {
        return Ok(record.access_token);
    }

    // 已过期：有 refresh_token 则自动续期
    if record.refresh_token.is_some() {
        crate::log_info!(
            "[Frp Auth] token 已过期，自动续期: provider={}",
            provider_id
        );
        refresh_token(state, provider_id).await?;
        return load_token(provider_id).await;
    }

    // 已过期且无 refresh_token → 无法续期
    Err(format!(
        "厂商 {} token 已过期且无 refresh_token，请重新认证",
        provider_id
    ))
}

/// 保存 API Key（auth_type=api_key 时由前端调用）
///
/// API Key 直接作为 access_token 存储，无过期时间、无 refresh_token。
pub async fn save_api_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    api_key::save_api_key(provider_id, api_key).await
}
