//! 版本安装管理统一分发逻辑（version_install_manager 的命令层实现）
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，聚合 `download` / `install` /
//! `loaders` / `preload` 共 12 个 action。`download_version` / `install_merged` /
//! `preload_mods_detail_cmd` 同时需要 state 和 app；`list_fabric_api_versions` /
//! `validate_loaders` 不需要 state；其余 loaders 命令仅需 state。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

use super::{download, install, loaders, preload};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionIdParams {
    version_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct McVersionParams {
    mc_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallMergedParams {
    mc_version: String,
    forge_version: Option<String>,
    neoforge_version: Option<String>,
    fabric_version: Option<String>,
    optifine_version: Option<String>,
    liteloader_version: Option<String>,
    instance_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ValidateLoadersParams {
    mc_version: String,
    forge_version: Option<String>,
    neoforge_version: Option<String>,
    fabric_version: Option<String>,
    optifine_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallFabricApiParams {
    version_id: String,
    download_url: String,
    file_name: String,
    hash: Option<String>,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    d.register(
        "download_version",
        handler!(state, app, params, {
            let p: VersionIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            download::download_version(&app, &state, p.version_id).await?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "install_merged",
        handler!(state, app, params, {
            let p: InstallMergedParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            install::install_merged(
                &app,
                &state,
                p.mc_version,
                p.forge_version,
                p.neoforge_version,
                p.fabric_version,
                p.optifine_version,
                p.liteloader_version,
                p.instance_name,
            )
            .await?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "list_forge_versions",
        handler!(state, _app, params, {
            let p: McVersionParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = loaders::list_forge_versions(&state, p.mc_version).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_neoforge_versions",
        handler!(state, _app, params, {
            let p: McVersionParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = loaders::list_neoforge_versions(&state, p.mc_version).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_fabric_versions",
        handler!(state, _app, _params, {
            let r = loaders::list_fabric_versions(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_optifine_versions",
        handler!(state, _app, _params, {
            let r = loaders::list_optifine_versions(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_liteloader_versions",
        handler!(state, _app, params, {
            let p: McVersionParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = loaders::list_liteloader_versions(&state, p.mc_version).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "validate_loaders",
        handler!(_state, _app, params, {
            let p: ValidateLoadersParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = loaders::validate_loaders(
                p.mc_version,
                p.forge_version,
                p.neoforge_version,
                p.fabric_version,
                p.optifine_version,
            )
            .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_fabric_api_versions",
        handler!(_state, _app, params, {
            let p: McVersionParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = loaders::list_fabric_api_versions(p.mc_version).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "install_fabric_api_for_version",
        handler!(state, _app, params, {
            let p: InstallFabricApiParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            loaders::install_fabric_api_for_version(
                &state,
                p.version_id,
                p.download_url,
                p.file_name,
                p.hash,
            )
            .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "preload_mods_detail_cmd",
        handler!(state, app, params, {
            let p: VersionIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            preload::preload_mods_detail_cmd(&app, &state, p.version_id).await?;
            Ok(serde_json::Value::Null)
        }),
    );

    d.register(
        "cancel_preload_mods_detail_cmd",
        handler!(_state, _app, _params, {
            preload::cancel_preload_mods_detail_cmd().await?;
            Ok(serde_json::Value::Null)
        }),
    );

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
