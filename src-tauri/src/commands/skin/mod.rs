//! 皮肤管理命令
//! 提供皮肤/披风管理子模块函数供 `manager::dispatch` dispatcher 调用：获取皮肤/披风
//! 信息（list）、上传皮肤（upload）、装备/取消披风（cape）、下载 URL 图片到本地。

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;

mod cape;
mod dispatcher;
mod list;
mod manager;
mod upload;

pub use cape::*;
pub use dispatcher::download_url_to_file;
pub use list::*;
pub use upload::*;

/// 统一皮肤管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `dispatcher::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn skin_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    dispatcher::dispatch(state, app, req).await
}
