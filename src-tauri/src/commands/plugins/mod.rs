//! 插件系统命令模块（编排层）
//! 外部插件存放于 `<base_dir>/plugins/<plugin_id>/`，每个插件目录包含
//! manifest.json 和入口 HTML。子模块按职责拆分：install / sandbox / spawn /
//! window / layout / export / personalization / types / helpers。

pub mod export;
mod helpers;
pub mod install;
pub mod layout;
pub(crate) mod manager;
pub mod personalization;
pub mod sandbox;
pub mod spawn;
mod types;
pub mod window;

pub(crate) use helpers::{is_valid_plugin_id, plugins_root, read_plugin_manifest};
pub use types::{
    ExternalPluginEntry, ExternalPluginManifest, ProcessPermissions, WindowPermissions,
};

/// 统一插件系统 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn plugins_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
