//! 系统模块统一分发逻辑（system_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，替代每个 action 一条 Tauri 命令。
//! 18 个 system action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（18 个，按子模块分组）：
//! - game_dir（7 个）：`open_game_dir` / `open_path` / `reveal_in_explorer`
//!   / `get_game_dir` / `write_text_file` / `get_system_memory` / `set_game_dir`
//! - config（2 个）：`get_config_path` / `save_config_to_file`
//! - developer（5 个）：`is_developer_unlocked` / `unlock_developer_mode`
//!   / `get_storage_dirs` / `get_system_info` / `get_cache_stats`
//! - about（1 个）：`get_about_data`
//! - logger（3 个）：`get_log_path` / `list_log_files` / `read_log_file`
//!
//! 注意事项：
//! - 子模块函数接收 `&AppState`（或不接收 state），handler 内调用时用 `&state` / 忽略 `_state`
//! - `open_game_dir` / `get_game_dir` / `save_config_to_file` / `set_game_dir` 需要 state
//! - `open_path` / `reveal_in_explorer` / `write_text_file` / `read_log_file` 需要参数
//! - `is_developer_unlocked` 返回 bool（非 Result），handler 内用
//!   `Ok(serde_json::to_value(x).map_err(|e| e.to_string())?)` 包装
//! - `get_storage_dirs` 返回 `StorageDirs`（非 Result），同样包装
//! - `get_system_info` 返回 `SystemInfo`（非 Result），同样包装
//! - `get_log_path` 返回 `String`（非 Result），同样包装
//! - `list_log_files` 返回 `Vec<String>`（非 Result），同样包装

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::system::{
    about::get_about_data,
    config::{get_config_path, save_config_to_file},
    developer::{
        get_cache_stats, get_storage_dirs, get_system_info, is_developer_unlocked,
        unlock_developer_mode,
    },
    game_dir::{
        get_game_dir, get_system_memory, open_game_dir, open_path, reveal_in_explorer,
        set_game_dir, write_text_file,
    },
};
use crate::handler;
use crate::logger::{get_log_path, list_log_files, read_log_file};
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// action 参数
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathParams {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteTextFileParams {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadLogFileParams {
    filename: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetGameDirParams {
    game_dir: String,
}

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // === game_dir（6 个） ===
    d.register("open_game_dir", handler!(state, _app, _params, {
        open_game_dir(&state).await?;
        Ok(serde_json::Value::Null)
    }));
    d.register("open_path", handler!(_state, _app, params, {
        let p: PathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        open_path(p.path).await?;
        Ok(serde_json::Value::Null)
    }));
    d.register("reveal_in_explorer", handler!(_state, _app, params, {
        let p: PathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        reveal_in_explorer(p.path).await?;
        Ok(serde_json::Value::Null)
    }));
    d.register("get_game_dir", handler!(state, _app, _params, {
        let r = get_game_dir(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("write_text_file", handler!(_state, _app, params, {
        let p: WriteTextFileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        write_text_file(p.path, p.content).await?;
        Ok(serde_json::Value::Null)
    }));
    d.register("get_system_memory", handler!(_state, _app, _params, {
        let r = get_system_memory().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("set_game_dir", handler!(state, _app, params, {
        let p: SetGameDirParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        set_game_dir(&state, p.game_dir).await?;
        Ok(serde_json::Value::Null)
    }));

    // === config（2 个） ===
    d.register("get_config_path", handler!(_state, _app, _params, {
        let r = get_config_path().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("save_config_to_file", handler!(state, _app, _params, {
        save_config_to_file(&state).await?;
        Ok(serde_json::Value::Null)
    }));

    // === developer（5 个） ===
    // 注意：is_developer_unlocked / get_storage_dirs / get_system_info 返回非 Result，
    // handler 内用 Ok(to_value(r)?) 包装。
    d.register("is_developer_unlocked", handler!(_state, _app, _params, {
        let r = is_developer_unlocked();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("unlock_developer_mode", handler!(_state, _app, _params, {
        unlock_developer_mode()?;
        Ok(serde_json::Value::Null)
    }));
    d.register("get_storage_dirs", handler!(_state, _app, _params, {
        let r = get_storage_dirs();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("get_system_info", handler!(_state, _app, _params, {
        let r = get_system_info();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("get_cache_stats", handler!(_state, _app, _params, {
        let r = get_cache_stats().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // === about（1 个） ===
    d.register("get_about_data", handler!(_state, _app, _params, {
        let r = get_about_data().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // === logger（3 个） ===
    // 注意：get_log_path / list_log_files 返回非 Result（String / Vec<String>），
    // handler 内用 Ok(to_value(r)?) 包装。read_log_file 返回 Result<String>。
    d.register("get_log_path", handler!(_state, _app, _params, {
        let r = get_log_path();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("list_log_files", handler!(_state, _app, _params, {
        let r = list_log_files();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("read_log_file", handler!(_state, _app, params, {
        let p: ReadLogFileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = read_log_file(p.filename)?;
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
