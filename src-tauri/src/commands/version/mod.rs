//! Version management commands

pub mod download;
pub mod export;
pub mod folder;
pub mod install;
pub mod launch;
pub mod list;
pub mod loaders;
pub mod manage;
pub mod mods;
pub mod personalization;
pub mod preload;
pub mod progress;
pub mod script_export;
pub mod types;

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

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

/// 版本列表/文件夹/管理/个性化统一 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::version_list_manager::dispatch` 进行 action 分发。
/// 原 17 个独立 Tauri 命令（6 list + 5 folder + 4 manage + 2 personalization）
/// 均通过此入口聚合调用。
#[tauri::command]
pub async fn version_list_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::version_list_manager::dispatch(state, app, req).await
}

/// 统一版本安装管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::version_install_manager::dispatch` 进行 action 分发。
///
/// 聚合的 11 个 action 来自 download / install / loaders / preload 四个子模块，
/// 详见 `utils::version_install_manager` 模块文档。
#[tauri::command]
pub async fn version_install_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::version_install_manager::dispatch(state, app, req).await
}

/// 统一版本导出管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::version_export_manager::dispatch` 进行 action 分发。
///
/// 聚合 4 个 action：get_export_options / export_modpack /
/// save_export_config / load_export_config。
#[tauri::command]
pub async fn version_export_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::version_export_manager::dispatch(state, app, req).await
}

/// 校验版本 ID / 实例名，防止路径遍历
pub fn sanitize_version_id(id: &str) -> Result<(), String> {
    crate::utils::path::sanitize_file_name(id)?;
    if id.contains(':') {
        return Err(format!("Invalid version id: {}", id));
    }
    // 额外用 components 验证只含 Normal 分量
    let path = std::path::Path::new(id);
    for comp in path.components() {
        if !matches!(comp, std::path::Component::Normal(_)) {
            return Err(format!("Invalid version id: {}", id));
        }
    }
    Ok(())
}

/// 校验 MC 版本号（与 version_id 同样规则）
pub fn sanitize_mc_version(v: &str) -> Result<(), String> {
    sanitize_version_id(v)
}
