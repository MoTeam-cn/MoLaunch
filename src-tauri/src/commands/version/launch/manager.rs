//! 版本启动统一分发逻辑（version_launch_manager 的命令层实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，7 个 action 覆盖
//! 启动 / 进度 / 停止 / 运行中实例 / 历史 / 取消 / 导出启动脚本。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

use super::super::script_export;
use super::{
    cancel_launch, get_launch_history, get_launch_progress, get_running_game, launch_game,
    stop_game,
};

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
    /// 临时追加的 JVM 参数（单次启动有效，不写入 setup.ini）
    /// 用途：联机模块启动 MC 时追加 -Djava.net.preferIPv4Stack=true
    extra_jvm_args: Option<Vec<String>>,
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

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "launch_game",
        handler!(state, app, params, {
            let p: LaunchGameParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = launch_game(
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
                p.extra_jvm_args,
            )
            .await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_launch_progress",
        handler!(state, _app, _params, {
            let r = get_launch_progress(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "cancel_launch",
        handler!(state, _app, _params, {
            cancel_launch(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "stop_game",
        handler!(state, _app, _params, {
            stop_game(&state).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_running_game",
        handler!(state, _app, _params, {
            let r = get_running_game(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "get_launch_history",
        handler!(state, _app, _params, {
            let r = get_launch_history(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "export_launch_script",
        handler!(state, _app, params, {
            let p: ExportLaunchScriptParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            script_export::export_launch_script(
                &state,
                p.version_id,
                p.username,
                p.uuid,
                p.login_type,
                p.java_path,
                p.save_path,
            )
            .await?;
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
