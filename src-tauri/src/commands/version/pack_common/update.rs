//! 版本内容目录更新与监听（原子更新 / notify 防抖监听）

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use notify::Watcher;
use tauri::{AppHandle, Emitter};

use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::download::DownloadSession;
use crate::state::AppState;
use crate::{log_error, log_info};

/// 原子更新：下载新版本 → 删除旧版本（下载失败保留旧文件）
pub(crate) async fn download_and_replace(
    state: &AppState,
    dir: &Path,
    old_file_name: &str,
    download_url: &str,
    new_file_name: &str,
    expected_size: i64,
    label: &str,
) -> Result<(), String> {
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let target_path = dir.join(new_file_name);
    let session = DownloadSession::start_grouped(
        state,
        label,
        vec![("下载新版本", 80.0), ("替换旧版本", 20.0)],
        false,
    )
    .await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = new_file_name.to_string();
    }
    let task = DownloadTask {
        id: format!("update_{}", new_file_name),
        urls: crate::minecraft::sources::cdn_urls(download_url),
        local_path: target_path.to_string_lossy().to_string(),
        expected_size,
        expected_hash: None,
    };
    let progress_callback = session.make_progress_callback(state, 0);
    let results = session
        .manager()
        .download_batch(vec![task], Some(progress_callback))
        .await;
    let result = results.first().ok_or("下载结果为空")?;
    if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
        let err = result
            .error
            .clone()
            .unwrap_or_else(|| "未知错误".to_string());
        session.mark_failed(state, 1);
        log_info!("[Packs] 更新下载失败，旧文件保留: {}", err);
        return Err(err);
    }
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_bytes(1, 1, 1);
    }
    if old_file_name != new_file_name {
        let old_path = dir.join(old_file_name);
        if old_path.exists() {
            if let Err(e) = std::fs::remove_file(&old_path) {
                log_info!("[Packs] 删除旧文件失败（不阻断）: {}", e);
            }
        }
    }
    session.mark_complete(state);
    Ok(())
}

const DEBOUNCE_QUIET_MS: u64 = 500;
static CURRENT_WATCHER: OnceLock<Mutex<Option<notify::RecommendedWatcher>>> = OnceLock::new();

fn current_watcher() -> &'static Mutex<Option<notify::RecommendedWatcher>> {
    CURRENT_WATCHER.get_or_init(|| Mutex::new(None))
}

/// 监听目录变化（notify 非递归，500ms 防抖），事件名由调用方指定
pub(crate) async fn watch_dir(
    app: &AppHandle,
    dir: PathBuf,
    event_name: &str,
) -> Result<(), String> {
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    log_info!(
        "[PackWatcher] 开始监听: {} -> {}",
        dir.display(),
        event_name
    );
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        if let Ok(event) = res {
            if matches!(
                event.kind,
                notify::EventKind::Create(_)
                    | notify::EventKind::Modify(_)
                    | notify::EventKind::Remove(_)
            ) {
                let _ = tx.send(());
            }
        }
    })
    .map_err(|e| {
        log_error!("[PackWatcher] 创建 watcher 失败: {}", e);
        e.to_string()
    })?;
    watcher
        .watch(&dir, notify::RecursiveMode::NonRecursive)
        .map_err(|e| {
            log_error!("[PackWatcher] 启动监听失败: {}", e);
            e.to_string()
        })?;
    {
        let mut guard = current_watcher()
            .lock()
            .expect("[PackWatcher] watcher mutex poisoned");
        *guard = Some(watcher);
    }
    let app_clone = app.clone();
    let event_name = event_name.to_string();
    std::thread::spawn(move || loop {
        if rx.recv().is_err() {
            break;
        }
        while rx
            .recv_timeout(Duration::from_millis(DEBOUNCE_QUIET_MS))
            .is_ok()
        {}
        if let Err(e) = app_clone.emit(&event_name, ()) {
            log_error!("[PackWatcher] emit {} 失败: {}", event_name, e);
        }
    });
    Ok(())
}

/// 停止当前目录监听
pub(crate) async fn unwatch_dir() -> Result<(), String> {
    let mut guard = current_watcher()
        .lock()
        .map_err(|e| format!("watcher mutex poisoned: {}", e))?;
    if guard.is_some() {
        log_info!("[PackWatcher] 停止监听");
        *guard = None;
    }
    Ok(())
}
