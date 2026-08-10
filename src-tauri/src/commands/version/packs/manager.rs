//! 版本 Pack 管理统一分发逻辑（version_packs_manager 的实现）
//! 仿 `version_mods_manager` 的注册式分发，11 个 action 在 Lazy 初始化时注册。
//! 每个 action 参数均携带 `kind`（resourcepack / shader）。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use super::{install, list, manage, preload, update, watcher};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

use super::types::PackKind;

/// 仅需 kind + versionId 的 action 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KindVersionIdParams {
    kind: PackKind,
    version_id: String,
}

/// toggle_pack 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TogglePackParams {
    kind: PackKind,
    version_id: String,
    file_name: String,
    enable: bool,
}

/// kind + versionId + file_name 参数（delete / reveal / get_pack_icon）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KindFileParams {
    kind: PackKind,
    version_id: String,
    file_name: String,
}

/// install_pack 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallPackParams {
    kind: PackKind,
    version_id: String,
    source_path: String,
}

/// update_pack 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePackParams {
    kind: PackKind,
    version_id: String,
    old_file_name: String,
    download_url: String,
    new_file_name: String,
    expected_size: i64,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "is_packs_available",
        handler!(state, _app, params, {
            let p: KindVersionIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = list::is_packs_available(&state, p.version_id, p.kind).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_packs",
        handler!(state, _app, params, {
            let p: KindVersionIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = list::list_packs(&state, p.version_id, p.kind).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "toggle_pack",
        handler!(state, _app, params, {
            let p: TogglePackParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r =
                manage::toggle_pack(&state, p.version_id, p.file_name, p.enable, p.kind).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "delete_pack",
        handler!(state, _app, params, {
            let p: KindFileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            manage::delete_pack(&state, p.version_id, p.file_name, p.kind).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "install_pack",
        handler!(state, _app, params, {
            let p: InstallPackParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            install::install_pack(&state, p.version_id, p.source_path, p.kind).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "open_packs_dir",
        handler!(state, _app, params, {
            let p: KindVersionIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            install::open_packs_dir(&state, p.version_id, p.kind).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "reveal_pack_file",
        handler!(state, _app, params, {
            let p: KindFileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            install::reveal_pack_file(&state, p.version_id, p.file_name, p.kind).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_pack_icon",
        handler!(state, _app, params, {
            let p: KindFileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = install::get_pack_icon(&state, p.version_id, p.file_name, p.kind).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "update_pack",
        handler!(state, _app, params, {
            let p: UpdatePackParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            update::update_pack(
                &state,
                p.version_id,
                p.old_file_name,
                p.download_url,
                p.new_file_name,
                p.expected_size,
                p.kind,
            )
            .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "watch_packs_dir",
        handler!(state, app, params, {
            let p: KindVersionIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            watcher::watch_packs_dir(&state, &app, p.version_id, p.kind).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "unwatch_packs_dir",
        handler!(_state, _app, _params, {
            watcher::unwatch_packs_dir().await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "preload_packs_detail",
        handler!(state, app, params, {
            let p: KindVersionIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            preload::preload_packs_detail_cmd(&app, &state, p.version_id, p.kind).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "cancel_preload_packs_detail",
        handler!(_state, _app, _params, {
            preload::cancel_preload_packs_detail_cmd().await?;
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
