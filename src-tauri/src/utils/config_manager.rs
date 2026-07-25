//! 配置管理统一分发逻辑（config_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 4 个 config action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（4 个）：
//! - `get_config`：读取配置（扁平化数组，支持 keys 过滤）
//! - `apply_config`：统一配置更新（接收 ConfigEntry 数组）
//! - `get_config_value`：读取单个 INI 配置值（直读 storage）
//! - `set_config_value`：设置单个 INI 配置值（直写 INI + 同步内存 AppConfig）
//!
//! 注意：
//! - `get_config` / `apply_config` / `set_config_value` 需要 state
//! - `get_config_value` 不需要 state（直接读 storage），handler 内用 `_state`
//! - 4 个命令均不需要 AppHandle，handler 内用 `_app` 忽略
//! - `get_config_path` / `save_config_to_file` 不在本次聚合范围，仍保留为独立 Tauri 命令

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::system::apply_config::{apply_config, get_config, ConfigEntry};
use crate::commands::system::config::{get_config_value, set_config_value};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// action 参数
// ============================================================

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetConfigValueParams {
    section: String,
    key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetConfigValueParams {
    section: String,
    key: String,
    value: String,
}

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("get_config", handler!(state, _app, params, {
        let p: GetConfigParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = get_config(&state, p.keys).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("apply_config", handler!(state, _app, params, {
        let p: ApplyConfigParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        apply_config(&state, p.entries).await?;
        Ok(serde_json::Value::Null)
    }));

    d.register("get_config_value", handler!(_state, _app, params, {
        let p: GetConfigValueParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = get_config_value(p.section, p.key).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("set_config_value", handler!(state, _app, params, {
        let p: SetConfigValueParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        set_config_value(&state, p.section, p.key, p.value).await?;
        Ok(serde_json::Value::Null)
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
