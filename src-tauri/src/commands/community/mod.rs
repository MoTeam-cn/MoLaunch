//! 社区资源命令模块
//! 提供搜索、详情、安装三大类功能，由 `manager::dispatch` 反序列化参数后调用。

pub mod community_config;
pub mod detail;
pub mod install;
pub mod manager;
pub mod search;
pub mod secure_config;

pub use detail::{get_mcmod_url, get_project_detail, get_project_versions};
pub use install::modpack::{install_local_modpack, install_modpack, preview_local_modpack};
pub use install::resource::{
    download_resource, download_resource_to_path, format_download_filename,
    get_resource_install_path, install_resource,
};
pub use search::{get_category_tags, search_resources};

/// 统一社区资源 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn community_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
