//! meta_manager offline 域 register（离线登录与账号管理 6 个 action）

use serde::Deserialize;

use crate::handler;
use crate::utils::dispatcher::Dispatcher;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginOfflineParams {
    username: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UuidParams {
    uuid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetOfflineSkinParams {
    uuid: String,
    skin: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveCustomSkinParams {
    uuid: String,
    file_path: String,
    variant: Option<String>,
}

/// 注册 offline 域 action
pub(super) fn register(d: &mut Dispatcher) {
    d.register(
        "login_offline",
        handler!(state, _app, params, {
            let p: LoginOfflineParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::commands::auth::offline::login_offline(&state, p.username).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "get_offline_accounts",
        handler!(state, _app, _params, {
            let r = crate::commands::auth::account::offline::get_offline_accounts(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "remove_offline_account",
        handler!(state, _app, params, {
            let p: UuidParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::auth::account::offline::remove_offline_account(&state, p.uuid).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "switch_offline_account",
        handler!(state, _app, params, {
            let p: UuidParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::commands::auth::account::offline::switch_offline_account(&state, p.uuid)
                .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "set_offline_skin",
        handler!(state, _app, params, {
            let p: SetOfflineSkinParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::auth::account::offline::set_offline_skin(&state, p.uuid, p.skin)
                .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "save_custom_skin",
        handler!(state, _app, params, {
            let p: SaveCustomSkinParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::commands::auth::account::offline::save_custom_skin(
                &state,
                p.uuid,
                p.file_path,
                p.variant,
            )
            .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
}
