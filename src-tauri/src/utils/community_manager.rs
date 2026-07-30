//! 社区资源模块统一分发逻辑（community_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，13 个 action 覆盖 search / detail /
//! install（resource / modpack）。search / detail / preview_local_modpack 不需要 state；
//! `req: SomeRequest` 类型参数直接反序列化为对应 Request 类型，避免冗余包裹结构体。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::community;
use crate::handler;
use crate::minecraft::community::types::{Platform, ResourceType};
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceTypeParams {
    resource_type: ResourceType,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectVersionsParams {
    platform: Platform,
    project_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct McmodUrlParams {
    platform: Platform,
    slug: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadToPathParams {
    url: String,
    file_name: String,
    save_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FormatFilenameParams {
    file_name: String,
    translated_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceInstallPathParams {
    resource_type: ResourceType,
    version_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilePathParams {
    file_path: String,
}


static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    d.register("search_resources", handler!(_state, _app, params, {
        let req: community::search::SearchRequest = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::search::search_resources(req).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("get_category_tags", handler!(_state, _app, params, {
        let p: ResourceTypeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::search::get_category_tags(p.resource_type).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("get_project_detail", handler!(_state, _app, params, {
        let req: community::detail::DetailRequest = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::detail::get_project_detail(req).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("get_project_versions", handler!(_state, _app, params, {
        let p: ProjectVersionsParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::detail::get_project_versions(p.platform, p.project_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("get_mcmod_url", handler!(_state, _app, params, {
        let p: McmodUrlParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::detail::get_mcmod_url(p.platform, p.slug).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("download_resource", handler!(state, _app, params, {
        let req: community::install::DownloadRequest = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::install::resource::download_resource(&state, req).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("download_resource_to_path", handler!(state, app, params, {
        let p: DownloadToPathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::install::resource::download_resource_to_path(
            &state,
            &app,
            p.url,
            p.file_name,
            p.save_path,
        )
        .await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("format_download_filename", handler!(state, _app, params, {
        let p: FormatFilenameParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::install::resource::format_download_filename(
            &state,
            p.file_name,
            p.translated_name,
        )
        .await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("install_resource", handler!(state, _app, params, {
        let req: community::install::DownloadRequest = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::install::resource::install_resource(&state, req).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("get_resource_install_path", handler!(state, _app, params, {
        let p: ResourceInstallPathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::install::resource::get_resource_install_path(
            &state,
            p.resource_type,
            p.version_id,
        )
        .await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("install_modpack", handler!(state, _app, params, {
        let req: community::install::InstallModpackRequest = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::install::modpack::install_modpack(&state, req).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("install_local_modpack", handler!(state, _app, params, {
        let req: community::install::InstallLocalModpackRequest = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::install::modpack::install_local_modpack(&state, req).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("preview_local_modpack", handler!(_state, _app, params, {
        let p: FilePathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = community::install::modpack::preview_local_modpack(p.file_path).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
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
