//! 版本 Mod 管理命令（模块入口 + 类型 re-export + version_mods_manager IPC 入口）
//! 子模块：types/helpers/metadata(jar 元数据读取)/watcher(目录监听)/list(查询)/
//! manage(toggle/delete)/install(安装+文件操作)/update(原子化下载+删旧)。原 10 个分散
//! Tauri 命令已聚合为 `version_mods_manager` 一个 IPC 入口通过 `action` 字段分发；子模块
//! 函数去 `#[tauri::command]` 标注改收 `&AppState`/`&AppHandle`，由 dispatch 反序列化参数后调用。

pub mod dependency_resolver;
pub(crate) mod helpers;
pub mod install;
pub mod list;
pub mod manage;
mod metadata;
mod types;
pub mod update;
pub mod watcher;

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// 统一版本 Mod 管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::version_mods_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn version_mods_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::version_mods_manager::dispatch(state, app, req).await
}

// 对外暴露类型和辅助函数（保持向后兼容路径）
// 注意：ModMetadata 在 metadata.rs 中是私有 use 引入的（use super::types::ModMetadata），
// 故必须从 types 直接重导出，不能走 metadata 中转
pub(crate) use helpers::get_mods_dir;
pub(crate) use metadata::read_mod_metadata;
pub use types::ModInfo;
pub(crate) use types::ModMetadata;
