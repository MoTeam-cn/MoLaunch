//! Version management commands

pub mod download;
pub mod export;
pub mod export_manager;
pub mod folder;
pub mod install;
pub mod install_manager;
pub mod launch;
pub mod list;
pub mod list_manager;
pub mod loaders;
pub mod manage;
pub mod mods;
pub mod personalization;
pub mod preload;
pub mod progress;
pub mod sanitize;
pub mod script_export;
pub mod types;

// Re-export types
pub use types::{DownloadProgressSnapshot, DownloadStageSnapshot, VersionInfo, VersionListResult};
// Re-export commands (保持 lib.rs 中 commands::version::* 路径兼容)
pub use list::{
    detect_version_type_from_dir, get_version_effective_dir, list_installed_versions,
    list_installed_versions_with_type, list_versions, resolve_isolation_mode, uninstall_version,
    InstalledVersionInfo,
};
pub use manage::{fix_version_files, get_selected_version, rename_version, set_selected_version};
pub use personalization::{
    get_version_personalization, update_version_personalization, VersionPersonalization,
};
pub use script_export::export_launch_script;
pub use sanitize::{sanitize_mc_version, sanitize_version_id};

/// 版本列表/文件夹/管理/个性化统一 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `list_manager::dispatch` 进行 action 分发。
/// 原 17 个独立 Tauri 命令（6 list + 5 folder + 4 manage + 2 personalization）
/// 均通过此入口聚合调用。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn version_list_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    list_manager::dispatch(state, app, req).await
}

/// 统一版本安装管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `install_manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn version_install_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    install_manager::dispatch(state, app, req).await
}

/// 统一版本导出管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `export_manager::dispatch` 进行 action 分发。
///
/// 聚合 4 个 action：get_export_options / export_modpack /
/// save_export_config / load_export_config。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn version_export_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    export_manager::dispatch(state, app, req).await
}
