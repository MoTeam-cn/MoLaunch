//! 认证模块统一分发逻辑（meta_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，28 个 auth action 覆盖
//! offline / microsoft / authlib 登录及账号管理、会话通用操作。
//! `ms_login_web_start` / `ms_login_web_exchange` / `ms_login_poll` 需要 `&app`。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::auth::{
    account::ms, account::offline, account::session, authlib, microsoft, offline as auth_offline,
};
use crate::handler;
use crate::minecraft::auth::authlib::Profile;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};


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
struct AuthlibFetchServerMetaParams {
    server_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthlibLoginParams {
    server_url: String,
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct AuthlibSelectProfileParams {
    profile: Profile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwitchAuthlibAccountParams {
    server_url: String,
    uuid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveAuthlibAccountParams {
    server_url: String,
    uuid: String,
}

// 命名遵循"action + Params"约定，字段使用 camelCase（与前端 params 对象一致）。

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthlibSkinInfoParams {
    server_url: String,
    uuid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthlibUploadSkinParams {
    server_url: String,
    uuid: String,
    /// PNG 文件本地路径（后端读取，避免前端引入 plugin-fs）
    file_path: String,
    /// "slim" 或 "default"
    model: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthlibDeleteSkinParams {
    server_url: String,
    uuid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthlibUploadCapeParams {
    server_url: String,
    uuid: String,
    file_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthlibDeleteCapeParams {
    server_url: String,
    uuid: String,
}


static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    d.register("login_offline", handler!(state, _app, params, {
        let p: LoginOfflineParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = auth_offline::login_offline(&state, p.username).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("get_offline_accounts", handler!(state, _app, _params, {
        let r = offline::get_offline_accounts(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("remove_offline_account", handler!(state, _app, params, {
        let p: UuidParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        offline::remove_offline_account(&state, p.uuid).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("switch_offline_account", handler!(state, _app, params, {
        let p: UuidParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = offline::switch_offline_account(&state, p.uuid).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("set_offline_skin", handler!(state, _app, params, {
        let p: SetOfflineSkinParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        offline::set_offline_skin(&state, p.uuid, p.skin).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("save_custom_skin", handler!(state, _app, params, {
        let p: SaveCustomSkinParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = offline::save_custom_skin(&state, p.uuid, p.file_path, p.variant).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("ms_login_get_config", handler!(_state, _app, _params, {
        let r = microsoft::ms_login_get_config().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("ms_login_web_start", handler!(_state, app, _params, {
        microsoft::ms_login_web_start(&app).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("ms_login_web_exchange", handler!(state, app, params, {
        let p: MsLoginWebExchangeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = microsoft::ms_login_web_exchange(&app, &state, p.code).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("ms_login_request_device_code", handler!(_state, _app, _params, {
        let r = microsoft::ms_login_request_device_code().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("ms_login_poll", handler!(state, app, params, {
        let p: MsLoginPollParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = microsoft::ms_login_poll(&app, &state, p.device_code).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("ms_login_refresh", handler!(state, _app, _params, {
        let r = microsoft::ms_login_refresh(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("get_ms_accounts", handler!(state, _app, _params, {
        let r = ms::get_ms_accounts(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("remove_ms_account", handler!(state, _app, params, {
        let p: UuidParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        ms::remove_ms_account(&state, p.uuid).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("switch_ms_account", handler!(state, _app, params, {
        let p: UuidParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = ms::switch_ms_account(&state, p.uuid).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("authlib_fetch_server_meta", handler!(_state, _app, params, {
        let p: AuthlibFetchServerMetaParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = authlib::authlib_fetch_server_meta(p.server_url).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("authlib_login", handler!(state, _app, params, {
        let p: AuthlibLoginParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = authlib::authlib_login(&state, p.server_url, p.username, p.password).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("authlib_select_profile", handler!(state, _app, params, {
        let p: AuthlibSelectProfileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = authlib::authlib_select_profile(&state, p.profile).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("switch_authlib_account", handler!(state, _app, params, {
        let p: SwitchAuthlibAccountParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = authlib::switch_authlib_account(&state, p.server_url, p.uuid).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("get_authlib_accounts", handler!(state, _app, _params, {
        let r = authlib::get_authlib_accounts(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("remove_authlib_account", handler!(state, _app, params, {
        let p: RemoveAuthlibAccountParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        authlib::remove_authlib_account(&state, p.server_url, p.uuid).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("authlib_get_skin_info", handler!(state, _app, params, {
        let p: AuthlibSkinInfoParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = authlib::authlib_get_skin_info(&state, p.server_url, p.uuid).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("authlib_upload_skin", handler!(state, _app, params, {
        let p: AuthlibUploadSkinParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        authlib::authlib_upload_skin(&state, p.server_url, p.uuid, p.file_path, p.model).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("authlib_delete_skin", handler!(state, _app, params, {
        let p: AuthlibDeleteSkinParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        authlib::authlib_delete_skin(&state, p.server_url, p.uuid).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("authlib_upload_cape", handler!(state, _app, params, {
        let p: AuthlibUploadCapeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        authlib::authlib_upload_cape(&state, p.server_url, p.uuid, p.file_path).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("authlib_delete_cape", handler!(state, _app, params, {
        let p: AuthlibDeleteCapeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        authlib::authlib_delete_cape(&state, p.server_url, p.uuid).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));
    d.register("get_login_status", handler!(state, _app, _params, {
        let r = session::get_login_status(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("logout", handler!(state, _app, _params, {
        session::logout(&state).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d
});

/// 按 `req.action` 分发到对应 handler
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}
