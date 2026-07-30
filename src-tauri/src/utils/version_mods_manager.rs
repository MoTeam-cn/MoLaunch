//! 版本 Mod 管理统一分发逻辑（version_mods_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，11 个 version::mods action 在
//! `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//! `watch_mods_dir` 额外需要 `AppHandle`（emit `mods-dir-changed` 事件）。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::version::mods::{install, list, manage, update, watcher};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

/// 仅需 versionId 的 action 参数（is_version_modable / list_mods / open_mods_dir
/// / get_version_mods_dir / watch_mods_dir 共 5 个）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionIdParams {
    version_id: String,
}

/// toggle_mod 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToggleModParams {
    version_id: String,
    file_name: String,
    enable: bool,
}

/// delete_mod 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteModParams {
    version_id: String,
    file_name: String,
}

/// install_mod 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallModParams {
    version_id: String,
    source_path: String,
}

/// reveal_mod_file 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RevealModFileParams {
    version_id: String,
    file_name: String,
}

/// update_mod 参数（阶段 4 新增）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateModParams {
    version_id: String,
    old_file_name: String,
    download_url: String,
    new_file_name: String,
    expected_size: i64,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("is_version_modable", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = list::is_version_modable(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("list_mods", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = list::list_mods(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("toggle_mod", handler!(state, _app, params, {
        let p: ToggleModParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = manage::toggle_mod(&state, p.version_id, p.file_name, p.enable).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("delete_mod", handler!(state, _app, params, {
        let p: DeleteModParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        manage::delete_mod(&state, p.version_id, p.file_name).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("install_mod", handler!(state, _app, params, {
        let p: InstallModParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        install::install_mod(&state, p.version_id, p.source_path).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("open_mods_dir", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        install::open_mods_dir(&state, p.version_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("reveal_mod_file", handler!(state, _app, params, {
        let p: RevealModFileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        install::reveal_mod_file(&state, p.version_id, p.file_name).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("get_version_mods_dir", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = install::get_version_mods_dir(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("update_mod", handler!(state, _app, params, {
        let p: UpdateModParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        update::update_mod(
            &state,
            p.version_id,
            p.old_file_name,
            p.download_url,
            p.new_file_name,
            p.expected_size,
        )
        .await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("watch_mods_dir", handler!(state, app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        watcher::watch_mods_dir(&state, &app, p.version_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("unwatch_mods_dir", handler!(_state, _app, _params, {
        watcher::unwatch_mods_dir().await?;
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
