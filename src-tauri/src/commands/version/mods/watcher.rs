//! Mods 目录文件监听（参考 PCL2 PageInstanceMod 中的 FileSystemWatcher）
//!
//! 使用 `notify` crate 监听 mods 目录的文件变化，通过 `mods-dir-changed` 事件
//! 通知前端自动刷新 mod 列表。实现「拖入新 mod 几秒后自动出现在列表中」的体验。
//!
//! ## 防抖设计
//!
//! 文件写入/复制会触发多次事件（如 `.jar` 下载过程中连续触发多次 Modify 事件），
//! 使用「静默期」防抖：收到事件后等待 `DEBOUNCE_QUIET_MS` 内无新事件才 emit，
//! 避免前端在文件还在写入时就开始重新加载导致读到不完整的文件。
//!
//! ## 生命周期
//!
//! - `watch_mods_dir(version_id)` 启动监听，**替换**之前的 watcher（旧 watcher drop 后自动停止）
//! - `unwatch_mods_dir` 主动停止监听（ModTab 组件卸载时调用）
//! - 全局 `Mutex<Option<RecommendedWatcher>>` 持有当前 watcher，保证同一时间只有一个监听
//! - watcher drop 时 channel 关闭，防抖线程自动退出

use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;
use crate::{log_error, log_info};

use super::helpers::get_mods_dir;
use super::sanitize_version_id;

/// 防抖静默期（ms）：收到事件后等待此时长无新事件才 emit
const DEBOUNCE_QUIET_MS: u64 = 500;

/// 全局 watcher 持有者（同一时间只有一个 mods 目录监听）
static CURRENT_WATCHER: OnceLock<Mutex<Option<RecommendedWatcher>>> = OnceLock::new();

fn current_watcher() -> &'static Mutex<Option<RecommendedWatcher>> {
    CURRENT_WATCHER.get_or_init(|| Mutex::new(None))
}

/// 开始监听版本的 mods 目录变化
///
/// 如果已有监听中的 watcher，会先停止旧的（drop 后自动停止），再启动新的。
/// 文件变化通过 `mods-dir-changed` 事件通知前端，前端应监听此事件并调用 `list_mods` 刷新。
#[tauri::command]
pub async fn watch_mods_dir(
    state: State<'_, AppState>,
    app: AppHandle,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    let mods_dir = get_mods_dir(&state, &version_id).await?;

    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    }

    log_info!("[ModsWatcher] 开始监听: {}", mods_dir.display());

    // 创建事件 channel（notify 回调 -> 防抖线程）
    let (tx, rx) = std::sync::mpsc::channel::<()>();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            // 只关心文件创建/修改/删除（忽略目录变化和属性变化）
            if matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            ) {
                let _ = tx.send(());
            }
        }
    })
    .map_err(|e| {
        log_error!("[ModsWatcher] 创建 watcher 失败: {}", e);
        e.to_string()
    })?;

    watcher
        .watch(&mods_dir, RecursiveMode::NonRecursive)
        .map_err(|e| {
            log_error!("[ModsWatcher] 启动监听失败: {}", e);
            e.to_string()
        })?;

    // 替换旧 watcher（drop 后旧 channel 关闭，旧防抖线程自动退出）
    {
        let mut guard = current_watcher()
            .lock()
            .expect("[ModsWatcher] watcher mutex poisoned");
        *guard = Some(watcher);
    }

    // 启动防抖线程：收到事件后等待静默期再 emit
    let app_clone = app.clone();
    std::thread::spawn(move || {
        loop {
            // 等待第一个事件（阻塞）
            if rx.recv().is_err() {
                break; // channel 关闭（watcher 被 drop），退出线程
            }

            // 防抖：持续接收事件直到静默期超时
            while rx
                .recv_timeout(Duration::from_millis(DEBOUNCE_QUIET_MS))
                .is_ok()
            {}

            // 静默期已过，emit 事件通知前端刷新
            if let Err(e) = app_clone.emit("mods-dir-changed", ()) {
                log_error!("[ModsWatcher] emit mods-dir-changed 失败: {}", e);
            }
        }
    });

    Ok(())
}

/// 停止监听 mods 目录（ModTab 组件卸载时调用）
#[tauri::command]
pub async fn unwatch_mods_dir() -> Result<(), String> {
    let mut guard = current_watcher()
        .lock()
        .map_err(|e| format!("watcher mutex poisoned: {}", e))?;
    if guard.is_some() {
        log_info!("[ModsWatcher] 停止监听");
        *guard = None; // drop watcher，channel 关闭，防抖线程退出
    }
    Ok(())
}
