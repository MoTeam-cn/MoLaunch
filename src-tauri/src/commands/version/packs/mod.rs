//! 版本资源包/光影管理命令（模块入口 + 类型 re-export + version_packs_manager IPC 入口）
//! 子模块：types/helpers/icon/watcher/list/manage/install/update/preload，共用 `pack_common` 公共抽象。
//! 13 个 action 已聚合为 `version_packs_manager` 一个 IPC 入口通过 `action` 字段分发。

mod helpers;
pub(crate) mod icon;
pub mod install;
pub mod list;
pub mod manage;
mod manager;
mod preload;
mod types;
pub mod update;
pub mod watcher;

pub use types::PackInfo;

/// 统一版本 Pack 管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn version_packs_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
