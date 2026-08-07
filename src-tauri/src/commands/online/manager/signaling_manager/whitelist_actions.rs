//! 白名单管理 action 注册（阶段三子任务 8 安全加强）：查询/增删/启停。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::utils::dispatcher::Dispatcher;

use super::{AddWhitelistParams, RemoveWhitelistParams, RoomCodeParams, SetWhitelistEnabledParams};

/// 注册白名单管理相关 action
pub fn register(d: &mut Dispatcher) {
    register_list_whitelist(d);
    register_add_whitelist(d);
    register_remove_whitelist(d);
    register_set_whitelist_enabled(d);
}

fn register_list_whitelist(d: &mut Dispatcher) {
    d.register(
        "room_list_whitelist",
        handler!(state, _app, params, {
            let p: RoomCodeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] room_list_whitelist: code={}", p.room_code);
            let result = client
                .signaling_list_whitelist(&creds, &p.room_code)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_list_whitelist 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_add_whitelist(d: &mut Dispatcher) {
    d.register(
        "room_add_whitelist",
        handler!(state, _app, params, {
            let p: AddWhitelistParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!(
                "[Online] room_add_whitelist: code={}, device_id={}",
                p.room_code,
                p.device_id
            );
            let result = client
                .signaling_add_whitelist(&creds, &p.room_code, &p.device_id)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_add_whitelist 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_remove_whitelist(d: &mut Dispatcher) {
    d.register(
        "room_remove_whitelist",
        handler!(state, _app, params, {
            let p: RemoveWhitelistParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!(
                "[Online] room_remove_whitelist: code={}, device_id={}",
                p.room_code,
                p.device_id
            );
            let result = client
                .signaling_remove_whitelist(&creds, &p.room_code, &p.device_id)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_remove_whitelist 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_set_whitelist_enabled(d: &mut Dispatcher) {
    d.register(
        "room_set_whitelist_enabled",
        handler!(state, _app, params, {
            let p: SetWhitelistEnabledParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_info!(
                "[Online] room_set_whitelist_enabled: code={}, enabled={}",
                p.room_code,
                p.enabled
            );
            let result = client
                .signaling_set_whitelist_enabled(&creds, &p.room_code, p.enabled)
                .await
                .map_err(|e| {
                    log_error!("[Online] room_set_whitelist_enabled 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}
