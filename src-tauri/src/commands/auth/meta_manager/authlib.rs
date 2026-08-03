//! meta_manager authlib 域 register（外置登录与皮肤管理 11 个 action）

use serde::Deserialize;

use crate::handler;
use crate::minecraft::auth::authlib::Profile;
use crate::utils::dispatcher::Dispatcher;

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

/// 注册 authlib 域 action
pub(super) fn register(d: &mut Dispatcher) {
    d.register(
        "authlib_fetch_server_meta",
        handler!(_state, _app, params, {
            let p: AuthlibFetchServerMetaParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::commands::auth::authlib::authlib_fetch_server_meta(p.server_url).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "authlib_login",
        handler!(state, _app, params, {
            let p: AuthlibLoginParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::commands::auth::authlib::authlib_login(
                &state,
                p.server_url,
                p.username,
                p.password,
            )
            .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "authlib_select_profile",
        handler!(state, _app, params, {
            let p: AuthlibSelectProfileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r =
                crate::commands::auth::authlib::authlib_select_profile(&state, p.profile).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "switch_authlib_account",
        handler!(state, _app, params, {
            let p: SwitchAuthlibAccountParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::commands::auth::authlib::switch_authlib_account(
                &state,
                p.server_url,
                p.uuid,
            )
            .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "get_authlib_accounts",
        handler!(state, _app, _params, {
            let r = crate::commands::auth::authlib::get_authlib_accounts(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "remove_authlib_account",
        handler!(state, _app, params, {
            let p: RemoveAuthlibAccountParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::auth::authlib::remove_authlib_account(&state, p.server_url, p.uuid)
                .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "authlib_get_skin_info",
        handler!(state, _app, params, {
            let p: AuthlibSkinInfoParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r =
                crate::commands::auth::authlib::authlib_get_skin_info(&state, p.server_url, p.uuid)
                    .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "authlib_upload_skin",
        handler!(state, _app, params, {
            let p: AuthlibUploadSkinParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::auth::authlib::authlib_upload_skin(
                &state,
                p.server_url,
                p.uuid,
                p.file_path,
                p.model,
            )
            .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "authlib_delete_skin",
        handler!(state, _app, params, {
            let p: AuthlibDeleteSkinParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::auth::authlib::authlib_delete_skin(&state, p.server_url, p.uuid)
                .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "authlib_upload_cape",
        handler!(state, _app, params, {
            let p: AuthlibUploadCapeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::auth::authlib::authlib_upload_cape(
                &state,
                p.server_url,
                p.uuid,
                p.file_path,
            )
            .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "authlib_delete_cape",
        handler!(state, _app, params, {
            let p: AuthlibDeleteCapeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::auth::authlib::authlib_delete_cape(&state, p.server_url, p.uuid)
                .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
}
