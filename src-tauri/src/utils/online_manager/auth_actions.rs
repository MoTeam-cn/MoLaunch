//! 认证 action 注册（轻量类）：设备状态查询、服务器时间、登出、清除凭证、手动续期。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::online::storage::OnlineStorage;
use crate::utils::dispatcher::Dispatcher;

use super::{DeviceStatus, ServerTimeInfo};

/// 注册轻量认证相关 action
pub fn register(d: &mut Dispatcher) {
    register_auth_status(d);
    register_auth_get_server_time(d);
    register_auth_logout(d);
    register_auth_clear(d);
    register_auth_refresh(d);
}

// 查询当前设备状态（不发起网络请求，仅读本地凭证）
fn register_auth_status(d: &mut Dispatcher) {
    d.register(
        "auth_status",
        handler!(state, _app, _params, {
            let storage = super::make_storage(&state);
            let creds = storage.load().await.unwrap_or(None).unwrap_or_default();
            let api_server_url = super::read_api_server_url(&state).await;

            let status = DeviceStatus {
                registered: creds.is_registered(),
                logged_in: !creds.device_token.is_empty(),
                token_expired: creds.is_token_expired(),
                device_pk: creds.device_pk,
                device_id: creds.device_id,
                token_expires_at: creds.token_expires_at,
                last_login_at: creds.last_login_at,
                api_server_url,
            };
            serde_json::to_value(status).map_err(|e| e.to_string())
        }),
    );
}

// 获取服务器时间（用于测试 api-server 连通性 + 校准本地时间）
fn register_auth_get_server_time(d: &mut Dispatcher) {
    d.register(
        "auth_get_server_time",
        handler!(state, _app, _params, {
            let api_url = super::read_api_server_url(&state).await;
            log_debug!(
                "[Online] auth_get_server_time 开始, api_server_url={}",
                api_url
            );
            let client = super::make_client(&state).await;
            let time_data = client.get_server_time().await.map_err(|e| {
                log_error!("[Online] auth_get_server_time 失败: {}", e);
                e.to_string()
            })?;
            let info = ServerTimeInfo {
                server_time: time_data.server_time,
                rfc3339: time_data.rfc3339,
                timezone: time_data.timezone,
                offset_seconds: time_data.offset_seconds,
            };
            log_debug!(
                "[Online] auth_get_server_time 成功, server_time={}, timezone={}",
                info.server_time,
                info.timezone
            );
            serde_json::to_value(info).map_err(|e| e.to_string())
        }),
    );
}

// 登出设备（撤销 JWT，不清除本地密钥）
fn register_auth_logout(d: &mut Dispatcher) {
    d.register(
        "auth_logout",
        handler!(state, _app, _params, {
            log_info!("[Online] auth_logout 开始");
            let storage = super::make_storage(&state);
            let mut creds = storage
                .load()
                .await
                .map_err(|e| e.to_string())?
                .unwrap_or_default();
            if creds.device_token.is_empty() {
                log_warn!("[Online] auth_logout 拒绝: 未登录");
                return Err("未登录，无需登出".to_string());
            }

            let client = super::make_client(&state).await;
            client.logout(&creds.device_token).await.map_err(|e| {
                log_error!("[Online] 登出请求失败: {}", e);
                format!("登出请求失败: {}", e)
            })?;

            // 清除 JWT（保留密钥和 device_pk，下次登录直接用）
            creds.device_token.clear();
            creds.token_expires_at = 0;
            storage.save(&creds).await.map_err(|e| e.to_string())?;

            log_info!("[Online] 设备已登出");
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );
}

// 清除设备凭证（注销设备，删除本地密钥）
fn register_auth_clear(d: &mut Dispatcher) {
    d.register(
        "auth_clear",
        handler!(_state, _app, _params, {
            log_info!("[Online] auth_clear 开始");
            OnlineStorage::clear().map_err(|e| {
                log_error!("[Online] 清除设备凭证失败: {}", e);
                e.to_string()
            })?;
            log_info!("[Online] 设备凭证已清除");
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );
}

// 用 refresh_token 续期 access token
//
// 前置条件：本地凭证已注册且持有未过期的 refresh_token。
// 供前端「手动续期」按钮或 auth_init 内部流程调用。
fn register_auth_refresh(d: &mut Dispatcher) {
    d.register(
        "auth_refresh",
        handler!(state, _app, _params, {
            log_info!("[Online] auth_refresh 开始");
            let storage = super::make_storage(&state);
            let creds = storage
                .load()
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "设备未注册，请先注册".to_string())?;
            if !creds.is_registered() {
                return Err("设备未注册，请先注册".to_string());
            }

            let updated = super::refresh_credentials(&state, creds)
                .await
                .map_err(|e| {
                    log_warn!("[Online] auth_refresh 续期失败: {}", e);
                    e
                })?;

            serde_json::to_value(super::build_device_status(
                &updated,
                super::read_api_server_url(&state).await,
            ))
            .map_err(|e| e.to_string())
        }),
    );
}
