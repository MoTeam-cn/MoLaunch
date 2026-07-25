//! 社区资源命令模块
//!
//! 提供搜索、详情、安装三大类功能。
//!
//! 注：原 13 个分散的 community Tauri 命令已聚合为 `community_manager` 单一 IPC 入口，
//! 通过请求体的 `action` 字段分发。子模块函数已去掉 `#[tauri::command]` 标注，
//! 参数签名改为 `&AppState` / `&AppHandle`，由 `utils::community_manager::dispatch`
//! 反序列化参数后调用。

pub mod community_config;
pub mod detail;
pub mod install;
pub mod search;
pub mod secure_config;

use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;

pub use detail::{get_mcmod_url, get_project_detail, get_project_versions};
pub use install::modpack::{install_modpack, install_local_modpack, preview_local_modpack};
pub use install::resource::{
    download_resource, download_resource_to_path, format_download_filename,
    get_resource_install_path, install_resource,
};
pub use search::{get_category_tags, search_resources};

/// 统一社区资源 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::community_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn community_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::community_manager::dispatch(state, app, req).await
}
