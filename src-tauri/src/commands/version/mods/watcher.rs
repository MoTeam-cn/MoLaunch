//! Mods 目录文件监听
//! 用 `notify` crate 监听 mods 目录变化，通过 `mods-dir-changed` 事件通知前端自动刷新
//! mod 列表。实现委托 `pack_common::watch_dir`（mods / resourcepacks / shaderpacks 共用，
//! 含 500ms 防抖与全局单例 watcher）。

use crate::log_info;
use crate::state::AppState;
use tauri::AppHandle;

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::get_mods_dir;

/// 开始监听版本的 mods 目录变化
///
/// 如果已有监听中的 watcher，会先停止旧的（drop 后自动停止），再启动新的。
/// 文件变化通过 `mods-dir-changed` 事件通知前端，前端应监听此事件并调用 `list_mods` 刷新。
pub async fn watch_mods_dir(
    state: &AppState,
    app: &AppHandle,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    let mods_dir = get_mods_dir(state, &version_id).await?;
    log_info!("[ModsWatcher] 开始监听: {}", mods_dir.display());
    pack_common::watch_dir(app, mods_dir, "mods-dir-changed").await
}

/// 停止监听 mods 目录（ModTab 组件卸载时调用）
pub async fn unwatch_mods_dir() -> Result<(), String> {
    log_info!("[ModsWatcher] 停止监听");
    pack_common::unwatch_dir().await
}
