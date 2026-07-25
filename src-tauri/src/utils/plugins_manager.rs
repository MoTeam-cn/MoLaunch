//! 插件模块统一分发逻辑（plugins_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 12 个 plugins action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（12 个）：
//! - `list_external_plugins`：列出所有已安装的外部插件（无参数）
//! - `read_external_plugin_file`：读取插件文件内容（plugin_id + file_path）
//! - `uninstall_external_plugin`：卸载插件（plugin_id）
//! - `install_external_plugin_from_dir`：从源目录安装插件（source_dir）
//! - `install_external_plugin_from_zip`：从 ZIP 文件路径安装插件（zip_path）
//! - `plugin_spawn_process`：执行插件子进程命令（plugin_id + command + args + cwd?）
//! - `plugin_create_window`：创建插件子窗口（需要 app + plugin_id + label + url + title + width? + height?）
//! - `load_custom_layout`：加载 URL 自定义布局内容（url + force_refresh?）
//! - `read_layout_sample`：读取示例布局内容（format）
//! - `export_plugin_sample`：导出插件示例模板（dest_path + as_zip）
//! - `read_personalization`：读取个性化配置（无参数）
//! - `write_personalization`：写入个性化配置（data: Value）
//!
//! 注意：所有 action 均不需要 `AppState`，handler 内用 `_state` 忽略；
//! `plugin_create_window` 需要 `&app` 用于创建 WebviewWindow。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::plugins;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// action 参数
// ============================================================

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

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("list_external_plugins", handler!(_state, _app, _params, {
        let r = plugins::sandbox::list_external_plugins().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("read_external_plugin_file", handler!(_state, _app, params, {
        let p: ReadExternalPluginFileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = plugins::sandbox::read_external_plugin_file(p.plugin_id, p.file_path).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("uninstall_external_plugin", handler!(_state, _app, params, {
        let p: PluginIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        plugins::sandbox::uninstall_external_plugin(p.plugin_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("install_external_plugin_from_dir", handler!(_state, _app, params, {
        let p: SourceDirParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = plugins::install::install_external_plugin_from_dir(p.source_dir).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("install_external_plugin_from_zip", handler!(_state, _app, params, {
        let p: ZipPathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = plugins::install::install_external_plugin_from_zip(p.zip_path).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("plugin_spawn_process", handler!(_state, _app, params, {
        let p: PluginSpawnProcessParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = plugins::spawn::plugin_spawn_process(
            p.plugin_id,
            p.command,
            p.args,
            p.cwd,
        )
        .await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("plugin_create_window", handler!(_state, app, params, {
        let p: PluginCreateWindowParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        plugins::window::plugin_create_window(
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
    }));

    d.register("load_custom_layout", handler!(_state, _app, params, {
        let p: LoadCustomLayoutParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = plugins::layout::load_custom_layout(p.url, p.force_refresh).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("read_layout_sample", handler!(_state, _app, params, {
        let p: ReadLayoutSampleParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = plugins::export::read_layout_sample(p.format).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("export_plugin_sample", handler!(_state, _app, params, {
        let p: ExportPluginSampleParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        plugins::export::export_plugin_sample(p.dest_path, p.as_zip).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("read_personalization", handler!(_state, _app, _params, {
        let r = plugins::personalization::read_personalization().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("write_personalization", handler!(_state, _app, params, {
        let p: WritePersonalizationParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        plugins::personalization::write_personalization(p.data).await?;
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
