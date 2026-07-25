//! 版本导出命令统一分发逻辑（version_export_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 4 个 action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER：
//!
//! 命令清单（4 个）：
//! - `get_export_options`：获取当前版本可用的导出选项列表（含动态子选项扫描）
//! - `export_modpack`：执行整合包导出（生成 Modrinth 格式 zip）
//! - `save_export_config`：保存当前导出配置到 .ini 文件
//! - `load_export_config`：从 .ini 文件读取导出配置

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::version::export::{config, get_export_options, export_modpack};
use crate::commands::version::export::types::{
    ExportModpackParams, LoadConfigResult, SaveConfigParams,
};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// 各 action 的强类型参数
// ============================================================

/// get_export_options 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetExportOptionsParams {
    version_id: String,
}

/// save_export_config 参数（直接复用 SaveConfigParams）
// SaveConfigParams 已在 types.rs 中定义，这里直接复用

/// load_export_config 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadConfigParams {
    config_path: String,
}

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // 获取导出选项列表（进入导出 Tab 时调用）
    d.register("get_export_options", handler!(state, app, params, {
        let p: GetExportOptionsParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = get_export_options(&state, &app, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // 执行整合包导出
    d.register("export_modpack", handler!(state, app, params, {
        let p: ExportModpackParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = export_modpack(&state, &app, p).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // 保存导出配置到 .ini 文件
    d.register("save_export_config", handler!(_state, _app, params, {
        let p: SaveConfigParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        config::save_config(&p)?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // 从 .ini 文件读取导出配置
    d.register("load_export_config", handler!(_state, _app, params, {
        let p: LoadConfigParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r: LoadConfigResult = config::load_config(&p.config_path)?;
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
