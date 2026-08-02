//! 插件模块统一分发逻辑（plugins 域 manager 模块）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，12 个 action 覆盖插件安装/卸载、
//! 子进程执行、子窗口创建、布局加载、个性化读写。所有 action 不需要 `AppState`；
//! `plugin_create_window` 需要 `&app` 用于创建 WebviewWindow。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use super::{export, install, layout, personalization, sandbox, spawn, window};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginIdParams {
    plugin_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadExternalPluginFileParams {
    plugin_id: String,
    file_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceDirParams {
    source_dir: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ZipPathParams {
    zip_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginSpawnProcessParams {
    plugin_id: String,
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginCreateWindowParams {
    plugin_id: String,
    label: String,
    url: String,
    title: String,
    width: Option<f64>,
    height: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadCustomLayoutParams {
    url: String,
    force_refresh: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadLayoutSampleParams {
    format: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportPluginSampleParams {
    dest_path: String,
    as_zip: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WritePersonalizationParams {
    data: serde_json::Value,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "list_external_plugins",
        handler!(_state, _app, _params, {
            let r = sandbox::list_external_plugins().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "read_external_plugin_file",
        handler!(_state, _app, params, {
            let p: ReadExternalPluginFileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = sandbox::read_external_plugin_file(p.plugin_id, p.file_path).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "uninstall_external_plugin",
        handler!(_state, _app, params, {
            let p: PluginIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            sandbox::uninstall_external_plugin(p.plugin_id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "install_external_plugin_from_dir",
        handler!(_state, _app, params, {
            let p: SourceDirParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = install::install_external_plugin_from_dir(p.source_dir).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "install_external_plugin_from_zip",
        handler!(_state, _app, params, {
            let p: ZipPathParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = install::install_external_plugin_from_zip(p.zip_path).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "plugin_spawn_process",
        handler!(_state, _app, params, {
            let p: PluginSpawnProcessParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = spawn::plugin_spawn_process(p.plugin_id, p.command, p.args, p.cwd).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "plugin_create_window",
        handler!(_state, app, params, {
            let p: PluginCreateWindowParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            window::plugin_create_window(
                &app,
                p.plugin_id,
                p.label,
                p.url,
                p.title,
                p.width,
                p.height,
            )
            .await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "load_custom_layout",
        handler!(_state, _app, params, {
            let p: LoadCustomLayoutParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = layout::load_custom_layout(p.url, p.force_refresh).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "read_layout_sample",
        handler!(_state, _app, params, {
            let p: ReadLayoutSampleParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = export::read_layout_sample(p.format).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "export_plugin_sample",
        handler!(_state, _app, params, {
            let p: ExportPluginSampleParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            export::export_plugin_sample(p.dest_path, p.as_zip).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "read_personalization",
        handler!(_state, _app, _params, {
            let r = personalization::read_personalization().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "write_personalization",
        handler!(_state, _app, params, {
            let p: WritePersonalizationParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            personalization::write_personalization(p.data).await?;
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
