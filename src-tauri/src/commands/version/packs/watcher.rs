//! Packs 目录文件监听
//! 委托 `pack_common::watch_dir`（含 500ms 防抖与全局单例 watcher），
//! 事件 `packs-dir-changed` 通知前端刷新。

use crate::log_info;
use crate::state::AppState;
use tauri::AppHandle;

use super::super::pack_common;
use super::super::sanitize_version_id;
use super::helpers::resolve_packs_dir;
use super::types::PackKind;

/// 开始监听版本内容目录变化
pub async fn watch_packs_dir(
    state: &AppState,
    app: &AppHandle,
    version_id: String,
    kind: PackKind,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    let dir = resolve_packs_dir(state, &version_id, kind).await?;
    log_info!("[PackWatcher] 开始监听: {}", dir.display());
    pack_common::watch_dir(app, dir, "packs-dir-changed").await
}

/// 停止监听内容目录（组件卸载时调用）
pub async fn unwatch_packs_dir() -> Result<(), String> {
    log_info!("[PackWatcher] 停止监听");
    pack_common::unwatch_dir().await
}
