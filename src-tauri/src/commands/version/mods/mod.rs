//! 版本 Mod 管理命令
//!
//! 模块结构：
//! - types.rs: ModInfo / ModMetadata / ModMeta 数据类型
//! - helpers.rs: get_mods_dir 共享辅助函数（sanitize_file_name 已迁移到 utils::path）
//! - metadata.rs: jar 内 mod 元数据读取流水线（read_mod_metadata + 8 个内部辅助）
//! - watcher.rs: mods 目录文件监听（notify crate + 防抖 + emit mods-dir-changed 事件）
//! - list.rs: 列表查询命令（list_mods / is_version_modable + infer_loader_type）
//! - manage.rs: 管理命令（toggle_mod / delete_mod）
//! - install.rs: 安装与文件操作命令（install_mod / open_mods_dir / get_version_mods_dir / reveal_mod_file）
//! - update.rs: Mod 更新命令（update_mod —— 阶段 4 新增，原子化下载+删旧）
//! - mod.rs: 模块入口 + pub mod 声明 + 类型 re-export + version_mods_manager IPC 入口
//!
//! 注意：原 10 个分散的 version::mods Tauri 命令已聚合为 `version_mods_manager` 一个 IPC 入口，
//! 通过请求体的 `action` 字段分发到各子模块函数。子模块函数已去掉 `#[tauri::command]` 标注，
//! 改为接收 `&AppState` / `&AppHandle`，由 `utils::version_mods_manager::dispatch`
//! 反序列化参数后调用。lib.rs 只需注册 `commands::version::mods::version_mods_manager` 一个命令。

pub(crate) mod helpers;
pub mod install;
pub mod list;
mod metadata;
pub mod manage;
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
