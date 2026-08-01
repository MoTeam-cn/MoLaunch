//! Device Code 授权流程
//!
//! 流程（参见设计文档 §6.4）：POST deviceCodeUrl 获取设备码，
//! 前端显示用户码 + 验证链接 + 倒计时，后端按 interval 轮询 tokenUrl。

use super::super::provider::read_provider_manifest;
use super::storage::{now_secs, parse_scopes, require_device_code_config, store_token_info};
use super::{DeviceCodePollResult, DeviceCodeResult, TokenResponse};
use crate::log_error;
use crate::log_info;
use crate::state::AppState;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;

// ============================================================
// Device Code 会话存储（内存，进程级）
// ============================================================

/// Device Code 会话（start_device_code 写入，poll_device_code 读取）
#[derive(Clone)]
struct DeviceCodeSession {
    device_code: String,
    /// 会话过期时间（Unix 秒）
    expires_at: u64,
    /// 轮询间隔（秒，存储供前端查询，poll_device_code 不直接使用）
    #[allow(dead_code)]
    interval: u64,
    /// tokenUrl（从 manifest 读取，避免轮询时重复读取）
    token_url: String,
    /// clientId
    client_id: String,
}

static DEVICE_CODE_SESSIONS: Lazy<Mutex<HashMap<String, DeviceCodeSession>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 移除 Device Code 会话
pub(super) fn remove_device_code_session(provider_id: &str) {
    if let Ok(mut sessions) = DEVICE_CODE_SESSIONS.lock() {
        sessions.remove(provider_id);
    }
}

// ============================================================
// 公共函数
// ============================================================

/// 启动 Device Code 流程
pub(super) async fn start_device_code(
    _state: &AppState,
    provider_id: &str,
) -> Result<DeviceCodeResult, String> {
    let manifest = read_provider_manifest(provider_id)?;
    let config = require_device_code_config(&manifest.auth, provider_id)?;

    log_info!("[Frp Auth] 启动 Device Code 流程: provider={}", provider_id);

    // 1. 请求设备码
    let client = crate::http::get_client();
    let scope_str = config.scopes.join(" ");
    let resp = client
        .post(&config.device_code_url)
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("scope", scope_str.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("请求设备码失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("请求设备码失败: HTTP {}", resp.status()));
    }

    let body: DeviceCodeResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析设备码响应失败: {}", e))?;

    // 2. 存入内存会话
    let interval = body.interval.unwrap_or(config.poll_interval);
    let expires_at = now_secs() + body.expires_in;
    {
        let mut sessions = DEVICE_CODE_SESSIONS
            .lock()
            .map_err(|e| format!("会话锁 poisoned: {}", e))?;
        sessions.insert(
            provider_id.to_string(),
            DeviceCodeSession {
                device_code: body.device_code.clone(),
                expires_at,
                interval,
                token_url: config.token_url.clone(),
                client_id: config.client_id.clone(),
            },
        );
    }

    log_info!(
        "[Frp Auth] Device Code 已获取: provider={}, user_code={}, expires_in={}s",
        provider_id,
        body.user_code,
        body.expires_in
    );

    Ok(DeviceCodeResult {
        user_code: body.user_code,
        verification_uri: body.verification_uri,
        expires_in: body.expires_in,
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

    // 2. 轮询 token
    let client = crate::http::get_client();
    let resp = client
        .post(&session.token_url)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", session.device_code.as_str()),
            ("client_id", session.client_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("轮询 token 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("轮询 token 失败: HTTP {}", resp.status()));
    }

    let body: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析轮询响应失败: {}", e))?;

    // 3. 处理结果
    if let Some(err) = &body.error {
        let status = match err.as_str() {
            "authorization_pending" => "pending",
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

    // 4. 成功 -> 存储 token
    let access_token = body.access_token.ok_or("token 响应缺少 access_token")?;
    let expires_at = body.expires_in.map(|secs| now_secs() + secs);
    let scopes = body.scope.as_ref().map(|s| parse_scopes(s));

    // 读取 manifest 中的 scopes 作为回退
    let manifest = read_provider_manifest(provider_id)?;
    let config = require_device_code_config(&manifest.auth, provider_id)?;
    let scopes_for_store = scopes.as_ref().or(Some(&config.scopes));

    store_token_info(
        provider_id,
        &access_token,
        body.refresh_token.as_deref(),
        body.expires_in,
        scopes_for_store,
    )?;
    remove_device_code_session(provider_id);

    log_info!(
        "[Frp Auth] Device Code 认证成功: provider={}, expires_at={:?}",
        provider_id,
        expires_at
    );

    Ok(DeviceCodePollResult {
        status: "success".to_string(),
        expires_at,
        scopes,
    })
}

// ============================================================
// 内部类型
// ============================================================

/// Device Code 端点响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    /// 轮询间隔（秒），部分服务端可能不返回
    interval: Option<u64>,
}
