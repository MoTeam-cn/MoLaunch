//! 皮肤模块统一分发逻辑（skin 域 manager 模块）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，7 个 action 覆盖皮肤/披风查询、
//! 上传、装备、下载。`download_url_to_file` 需要 state（路径校验用下载目录）；
//! `get_skin_cape_info` / `get_skin_url` / `get_cape_url` 需要 `&app` 用于图片缓存。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use super::*;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetSkinUrlParams {
    uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadSkinParams {
    file_path: String,
    variant: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EquipCapeParams {
    cape_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadUrlToFileParams {
    url: String,
    path: String,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "get_skin_cape_info",
        handler!(state, app, _params, {
            let r = get_skin_cape_info(&state, &app).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_skin_url",
        handler!(state, app, params, {
            let p: GetSkinUrlParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = get_skin_url(&state, &app, p.uuid).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_cape_url",
        handler!(state, app, _params, {
            let r = get_cape_url(&state, &app).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "upload_skin",
        handler!(state, _app, params, {
            let p: UploadSkinParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            upload_skin(&state, p.file_path, p.variant).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "equip_cape",
        handler!(state, _app, params, {
            let p: EquipCapeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            equip_cape(&state, p.cape_id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "unequip_cape",
        handler!(state, _app, _params, {
            unequip_cape(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "download_url_to_file",
        handler!(state, _app, params, {
            let p: DownloadUrlToFileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            download_url_to_file(&state, p.url, p.path).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d
});

/// 分发入口
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}
