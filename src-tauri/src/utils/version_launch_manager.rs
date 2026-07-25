//! 版本启动相关命令的统一分发逻辑（version_launch_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 7 个 action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER，
//! 聚合原 `version::launch`（6 个）+ `version::script_export`（1 个）共 7 个独立 IPC 命令。
//!
//! 命令清单（7 个）：
//! - `launch_game`：启动游戏（需要 AppState + AppHandle，参数较多）
//! - `get_launch_progress`：获取启动进度
//! - `cancel_launch`：取消启动
//! - `stop_game`：停止游戏
//! - `get_running_game`：获取当前运行的游戏 PID
//! - `get_launch_history`：获取启动历史记录
//! - `export_launch_script`：导出 .bat 启动脚本（参数较多）
//!
//! 注意：所有子模块函数已去掉 `#[tauri::command]` 标注，参数签名改为 `&AppState` / `&AppHandle`，
//! 由本 dispatcher 反序列化 params 后调用。`launch_game` 需要 `AppHandle`（用于 emit `game-exited` 事件）。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::version::{launch, script_export};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// action 参数
// ============================================================

/// launch_game 参数（与原 launch_game 命令参数一一对应，字段名 camelCase）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LaunchGameParams {
    version_id: String,
    java_path: Option<String>,
    username: String,
    uuid: String,
    login_type: Option<String>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
}

/// export_launch_script 参数（与原 export_launch_script 命令参数一一对应，字段名 camelCase）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportLaunchScriptParams {
    version_id: String,
    username: String,
    uuid: String,
    login_type: Option<String>,
    java_path: Option<String>,
    save_path: String,
}

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("launch_game", handler!(state, app, params, {
        let p: LaunchGameParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = launch::launch_game(
            &state,
            &app,
            p.version_id,
            p.java_path,
            p.username,
            p.uuid,
            p.login_type,
            p.window_width,
            p.window_height,
            p.server_address,
            p.server_port,
        )
        .await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_launch_progress", handler!(state, _app, _params, {
        let r = launch::get_launch_progress(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("cancel_launch", handler!(state, _app, _params, {
        let r = launch::cancel_launch(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("stop_game", handler!(state, _app, _params, {
        let r = launch::stop_game(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_running_game", handler!(state, _app, _params, {
        let r = launch::get_running_game(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_launch_history", handler!(state, _app, _params, {
        let r = launch::get_launch_history(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("export_launch_script", handler!(state, _app, params, {
        let p: ExportLaunchScriptParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = script_export::export_launch_script(
            &state,
            p.version_id,
            p.username,
            p.uuid,
            p.login_type,
            p.java_path,
            p.save_path,
        )
        .await?;
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
