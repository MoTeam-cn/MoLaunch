//! 联机功能命令模块
//! 提供 MoLaunch 客户端联机能力 IPC 入口：设备认证（注册/登录/登出/状态查询/服务器时间）、
//! 房间管理（创建/加入/退出/关闭，阶段二）、信令流程（轮询/确认/踢人/保活，阶段二）、虚拟网卡
//! （Wintun/TUN，阶段二）、MC 端口探测（阶段三）。所有 action 通过 `online_manager` 单一
//! IPC 入口聚合，由 `manager::dispatch` 分发。

pub(crate) mod manager;

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;

/// 统一联机 IPC 入口
///
/// 接收 `ActionRequest { action, params }`，转发到
/// `manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn online_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
