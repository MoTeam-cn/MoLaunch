//! meta_manager microsoft 域 register（微软登录与账号管理 9 个 action）

use serde::Deserialize;

use crate::handler;
use crate::utils::dispatcher::Dispatcher;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MsLoginWebExchangeParams {
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MsLoginPollParams {
    device_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UuidParams {
    uuid: String,
}

/// 注册 microsoft 域 action
pub(super) fn register(d: &mut Dispatcher) {
    d.register(
        "ms_login_get_config",
        handler!(_state, _app, _params, {
            let r = crate::commands::auth::microsoft::ms_login_get_config().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "ms_login_web_start",
        handler!(_state, app, _params, {
            crate::commands::auth::microsoft::ms_login_web_start(&app).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "ms_login_web_exchange",
        handler!(state, app, params, {
            let p: MsLoginWebExchangeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r =
                crate::commands::auth::microsoft::ms_login_web_exchange(&app, &state, p.code)
                    .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "ms_login_request_device_code",
        handler!(_state, _app, _params, {
            let r = crate::commands::auth::microsoft::ms_login_request_device_code().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "ms_login_poll",
        handler!(state, app, params, {
            let p: MsLoginPollParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::commands::auth::microsoft::ms_login_poll(&app, &state, p.device_code)
                .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "ms_login_refresh",
        handler!(state, _app, _params, {
            let r = crate::commands::auth::microsoft::ms_login_refresh(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "get_ms_accounts",
        handler!(state, _app, _params, {
            let r = crate::commands::auth::account::ms::get_ms_accounts(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "remove_ms_account",
        handler!(state, _app, params, {
            let p: UuidParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::auth::account::ms::remove_ms_account(&state, p.uuid).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "switch_ms_account",
        handler!(state, _app, params, {
            let p: UuidParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::commands::auth::account::ms::switch_ms_account(&state, p.uuid).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
}