//! 版本启动命令
//! 模块结构：
//! - mod.rs: 共享类型 GameExitEvent + `version_launch_manager` 转发入口 + 子模块 re-export
//! - manager.rs: `utils::dispatcher::Dispatcher` 注册式分发（7 个 action 的注册表）
//! - build_config.rs: build_launch_config（从全局配置+版本设置+前端入参构建 LaunchConfig）
//! - build.rs: launch_game 编排 + 参数构建 helper（parse_server_enter / resolve_game_language）
//! - spawn.rs: 启动退出监视 spawn_exit_watcher + 运行状态/历史短命令
//! - failure.rs: handle_launch_failure（启动失败崩溃分析+事件通知+状态清理）

mod build;
mod build_config;
mod failure;
mod manager;
mod spawn;

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::State;

pub use build::*;
pub use spawn::*;

/// 游戏退出事件数据
#[derive(Clone, serde::Serialize)]
pub struct GameExitEvent {
    pub pid: u32,
    pub version_id: String,
    pub exit_code: i32,
    pub is_normal: bool,
    /// 崩溃详情（仅异常退出时可能有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_info: Option<crate::minecraft::launch::watcher::CrashInfo>,
}

/// 版本启动管理统一 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 原 7 个独立 Tauri 命令（6 个 launch + 1 个 script_export）均通过此入口聚合调用。
#[tauri::command]
pub async fn version_launch_manager(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}