//! 联机功能命令模块
//! 提供 MoLaunch 客户端联机能力 IPC 入口：设备认证（注册/登录/登出/状态查询/服务器时间）、
//! 房间管理（创建/加入/退出/关闭，阶段二）、信令流程（轮询/确认/踢人/保活，阶段二）、虚拟网卡
//! （Wintun/TUN，阶段二）、MC 端口探测（阶段三）。所有 action 通过 `online_manager` 单一
//! IPC 入口聚合，由 `manager::dispatch` 分发。

pub(crate) mod manager;

/// 统一联机 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn online_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}