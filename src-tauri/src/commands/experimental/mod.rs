//! 实验性功能命令模块
//!
//! 实验性功能默认关闭（`config.ini [Experimental] enabled`），仅在设置页开启后：
//! 1. 顶部导航显示「实验性」入口；
//! 2. 启动时（或运行中开启时）挂载 SQLite 聊天库（`.Molaunch/chat.db`），连接由系统维护；
//! 3. 本模块的聊天 / 日志分析 / Agent 工具命令可用（未开启时一律返回错误）。
//!
//! 子模块：
//! - `db`：聊天库 schema 声明与数据访问（建表/迁移由 `crate::utils::sqlite` 公共工具负责）
//! - `agent`：Agent 工具定义与执行（版本隔离感知；list_installed_versions / ask_user 等）
//! - `manager`：action 分发（经 `experimental_manager` IPC 入口，AI action 亦并入）
//! - `types`：IPC 出入参类型

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;

pub mod agent;
pub mod db;
pub mod manager;
pub mod types;

/// 实验性功能命令统一入口
#[tauri::command]
pub async fn experimental_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
