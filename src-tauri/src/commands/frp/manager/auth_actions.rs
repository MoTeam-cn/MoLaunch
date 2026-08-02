//! 认证体系 action 注册（阶段三：OAuth2 / Device Code / API Key）。
//!
//! 包含：查询认证状态、启动/轮询各流程、刷新 token、撤销认证、保存 API Key。

use crate::commands::frp;
use crate::handler;
use crate::utils::dispatcher::Dispatcher;

use super::{ProviderIdParams, SaveApiKeyParams};

/// 注册认证体系相关 action
pub fn register(d: &mut Dispatcher) {
    d.register(
        "get_auth_status",
        handler!(state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::auth::get_auth_status(&state, &p.provider_id).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "start_oauth2",
        handler!(state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::auth::start_oauth2(&state, &p.provider_id).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "start_device_code",
        handler!(state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::auth::start_device_code(&state, &p.provider_id).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "poll_device_code",
        handler!(state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::auth::poll_device_code(&state, &p.provider_id).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "refresh_token",
        handler!(state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::auth::refresh_token(&state, &p.provider_id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "revoke_auth",
        handler!(_state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::auth::revoke_auth(&p.provider_id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "save_api_key",
        handler!(_state, _app, params, {
            let p: SaveApiKeyParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::auth::save_api_key(&p.provider_id, &p.api_key).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
}
