//! 配置管理统一分发逻辑（system 域 config_manager 模块）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，2 个 action：
//! `get_config` / `apply_config`。两个命令均不需要 `AppHandle`。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use super::apply_config::{apply_config, get_config, ConfigEntry};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetConfigParams {
    keys: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplyConfigParams {
    entries: Vec<ConfigEntry>,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "get_config",
        handler!(state, _app, params, {
            let p: GetConfigParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = get_config(&state, p.keys).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "apply_config",
        handler!(state, _app, params, {
            let p: ApplyConfigParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            apply_config(&state, p.entries).await?;
            Ok(serde_json::Value::Null)
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
