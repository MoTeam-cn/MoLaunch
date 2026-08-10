//! 版本内容目录通用操作（mods / resourcepacks / shaderpacks 共用）
//! 提供目录解析、条目枚举、启停、删除、安装、原子更新、目录监听。

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use notify::Watcher;
use tauri::{AppHandle, Emitter};

use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::download::DownloadSession;
use crate::minecraft::isolation::{get_effective_game_dir, IsolationMode};
use crate::state::AppState;
use crate::{log_error, log_info};

/// 目录条目（枚举结果）
pub(crate) struct DirEntry {
    pub file_name: String,
    pub enabled_name: String,
    pub is_enabled: bool,
    pub is_dir: bool,
    pub size: u64,
}

/// 解析版本隔离目录（mods / resourcepacks / shaderpacks 所在层）
pub(crate) async fn resolve_effective_game_dir(
    state: &AppState,
    version_id: &str,
) -> Result<PathBuf, String> {
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let global_isolation_mode = state.config.lock().await.isolation_mode;
    let isolation_mode = crate::commands::version::list::resolve_isolation_mode(
        &game_dir,
        version_id,
        global_isolation_mode,
    );
    let version_type =
        crate::commands::version::list::detect_version_type_from_dir(&game_dir, version_id);
    let mode = IsolationMode::from_u32(isolation_mode);
    Ok(get_effective_game_dir(&game_dir, version_id, mode, version_type))
}

/// 解析版本隔离目录下的内容子目录（mods / resourcepacks / shaderpacks）
pub(crate) async fn resolve_version_subdir(
    state: &AppState,
    version_id: &str,
    subdir: &str,
) -> Result<PathBuf, String> {
    Ok(resolve_effective_game_dir(state, version_id).await?.join(subdir))
}

/// 去除启停后缀，得到启用时的文件名
pub(crate) fn enabled_name_of(file_name: &str) -> String {
    file_name
        .trim_end_matches(".disabled")
        .trim_end_matches(".old")
        .to_string()
}

/// 是否处于启用状态
fn is_enabled_name(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    !(lower.ends_with(".disabled") || lower.ends_with(".old"))
}

/// 是否匹配允许的扩展名（含 .disabled / .old 变体）
fn matches_suffixes(file_name: &str, suffixes: &[&str]) -> bool {
    let lower = file_name.to_lowercase();
    suffixes.iter().any(|s| {
        let suffix = format!(".{}", s);
        lower.ends_with(&suffix)
            || lower.ends_with(&format!("{}.disabled", suffix))
            || lower.ends_with(&format!("{}.old", suffix))
    })
}

fn dir_total_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_total_size(&path);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

/// 枚举目录条目（扩展名过滤 + 可选文件夹，按文件名排序）
pub(crate) fn list_entries(
    dir: &Path,
    suffixes: &[&str],
    include_dirs: bool,
) -> Result<Vec<DirEntry>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let read = std::fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;
    for entry in read.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let is_dir = path.is_dir();
        if is_dir {
            if !include_dirs {
                continue;
            }
        } else if !matches_suffixes(&file_name, suffixes) {
            continue;
        }
        let size = if is_dir {
            dir_total_size(&path)
        } else {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        };
        entries.push(DirEntry {
            enabled_name: enabled_name_of(&file_name),
            is_enabled: is_enabled_name(&file_name),
            file_name,
            is_dir,
            size,
        });
    }
    entries.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    Ok(entries)
}

/// 启停条目：重命名 .disabled / .old，返回新文件名
pub(crate) fn toggle_entry(dir: &Path, file_name: &str, enable: bool) -> Result<String, String> {
    let src_path = dir.join(file_name);
    if !src_path.exists() {
        return Err(format!("文件不存在: {}", file_name));
    }
    if is_enabled_name(file_name) == enable {
        return Ok(file_name.to_string());
    }
    let new_name = if enable {
        enabled_name_of(file_name)
    } else {
        let disabled_name = format!("{}.disabled", file_name);
        if !dir.join(&disabled_name).exists() {
            disabled_name
        } else {
            format!("{}.old", file_name)
        }
    };
    let dst_path = dir.join(&new_name);
    if dst_path.exists() && dst_path != src_path {
        return Err(format!("目标文件已存在: {}", new_name));
    }
    std::fs::rename(&src_path, &dst_path).map_err(|e| format!("重命名失败: {}", e))?;
    Ok(new_name)
}

/// 删除条目（文件或目录）
pub(crate) fn delete_entry(dir: &Path, file_name: &str) -> Result<(), String> {
    let path = dir.join(file_name);
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_name));
    }
    if path.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| format!("删除目录失败: {}", e))?;
    } else {
        std::fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }
    Ok(())
}

/// 从外部路径安装条目（文件或目录，自动去除启停后缀），返回安装后的文件名
pub(crate) fn install_entry(
    dir: &Path,
    source_path: &str,
    suffixes: &[&str],
) -> Result<String, String> {
    if !crate::utils::path::is_safe_relative_path(source_path) {
        return Err("源路径不能包含 ..".to_string());
    }
    let src = Path::new(source_path);
    if !src.is_absolute() {
        return Err("源路径必须是绝对路径".to_string());
    }
    if !src.exists() {
        return Err(format!("源文件不存在: {}", source_path));
    }
    let original_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无法获取文件名")?
        .to_string();
    let clean_name = enabled_name_of(&original_name);
    if !src.is_dir() && !matches_suffixes(&clean_name, suffixes) {
        return Err(format!("不支持的文件格式: {}", clean_name));
    }
    let dst = dir.join(&clean_name);
    if dst.exists() {
        return Err(format!("目标目录已存在同名文件: {}", clean_name));
    }
    if src.is_dir() {
        copy_dir_recursive(src, &dst)?;
    } else {
        std::fs::copy(src, &dst).map_err(|e| format!("复制文件失败: {}", e))?;
    }
    Ok(clean_name)
}

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
    log_info!("[PackWatcher] 开始监听: {} -> {}", dir.display(), event_name);
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, _>| {
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
