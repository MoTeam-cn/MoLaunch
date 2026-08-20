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
mod manager;
mod metadata;
mod types;
pub mod update;
pub mod watcher;

// 对外暴露类型和辅助函数（保持向后兼容路径）
// ModMetadata 定义已下沉至 minecraft::community::types（消除架构倒置），此处 re-export 保持旧路径
#[allow(unused_imports)] // 向后兼容 re-export：preload 已直接引用领域层，保留旧路径供外部引用
pub(crate) use crate::minecraft::community::types::ModMetadata;
pub(crate) use helpers::get_mods_dir;
pub(crate) use metadata::read_mod_metadata;
pub use types::ModInfo;

/// 统一版本 Mod 管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn version_mods_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
