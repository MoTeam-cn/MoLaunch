//! 皮肤模块统一分发逻辑（skin_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 7 个 skin action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（7 个）：
//! - `get_skin_cape_info`：获取当前账号的皮肤/披风信息（需要 state + app）
//! - `get_skin_url`：获取皮肤 PNG URL（带本地缓存，需要 state + app + uuid）
//! - `get_cape_url`：获取当前已装备披风的下载 URL（带本地缓存，需要 state + app）
//! - `upload_skin`：上传/修改皮肤（需要 state + file_path + variant）
//! - `equip_cape`：装备披风（需要 state + cape_id）
//! - `unequip_cape`：取消披风（需要 state）
//! - `download_url_to_file`：下载指定 URL 的图片到本地文件（不需要 state，只用 url + path）
//!
//! 注意：`download_url_to_file` 不需要 state，handler 内用 `_state` / `_app` 忽略；
//! `get_skin_cape_info` / `get_skin_url` / `get_cape_url` 需要 `&app` 用于图片缓存。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::skin;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// action 参数
// ============================================================

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

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("get_skin_cape_info", handler!(state, app, _params, {
        let r = skin::get_skin_cape_info(&state, &app).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_skin_url", handler!(state, app, params, {
        let p: GetSkinUrlParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = skin::get_skin_url(&state, &app, p.uuid).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_cape_url", handler!(state, app, _params, {
        let r = skin::get_cape_url(&state, &app).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("upload_skin", handler!(state, _app, params, {
        let p: UploadSkinParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        skin::upload_skin(&state, p.file_path, p.variant).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("equip_cape", handler!(state, _app, params, {
        let p: EquipCapeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        skin::equip_cape(&state, p.cape_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("unequip_cape", handler!(state, _app, _params, {
        skin::unequip_cape(&state).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("download_url_to_file", handler!(_state, _app, params, {
        let p: DownloadUrlToFileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        skin::download_url_to_file(p.url, p.path).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

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
