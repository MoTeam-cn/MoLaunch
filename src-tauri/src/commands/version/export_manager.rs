//! 版本导出命令统一分发逻辑（version_export_manager 的命令层实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，4 个 action：
//! `get_export_options` / `export_modpack` / `save_export_config` / `load_export_config`。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

use super::export::types::{ExportModpackParams, LoadConfigResult, SaveConfigParams};
use super::export::{config, export_modpack, get_export_options};

/// get_export_options 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetExportOptionsParams {
    version_id: String,
}

// save_export_config 参数（直接复用 SaveConfigParams）
// SaveConfigParams 已在 types.rs 中定义，这里直接复用

/// load_export_config 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadConfigParams {
    config_path: String,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // 获取导出选项列表（进入导出 Tab 时调用）
    d.register(
        "get_export_options",
        handler!(state, app, params, {
            let p: GetExportOptionsParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = get_export_options(&state, &app, p.version_id).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    // 执行整合包导出
    d.register(
        "export_modpack",
        handler!(state, app, params, {
            let p: ExportModpackParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = export_modpack(&state, &app, p).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    // 保存导出配置到 .ini 文件
    d.register(
        "save_export_config",
        handler!(_state, _app, params, {
            let p: SaveConfigParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            config::save_config(&p)?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    // 从 .ini 文件读取导出配置
    d.register(
        "load_export_config",
        handler!(_state, _app, params, {
            let p: LoadConfigParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r: LoadConfigResult = config::load_config(&p.config_path)?;
            serde_json::to_value(r).map_err(|e| e.to_string())
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
