//! Windows 便携版下载安装流程
//!
//! 1. 复用通用 DownloadManager 下载新 exe 到临时目录（分片/限速/失败重试，进度经回调转发）
//! 2. 释放 updater.exe 到 AppData
//! 3. 启动 updater.exe 子进程（传递旧 exe 路径、新 exe 路径、主进程 PID）
//! 4. 主程序退出（updater 接管替换文件）

use std::sync::Arc;
use tauri::AppHandle;
use tauri::Emitter;

use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask, GlobalProgress};
use crate::state::AppState;

use super::{UpdateInfo, PROGRESS_EVENT};

/// Windows 便携版下载安装流程
#[cfg(target_os = "windows")]
pub(super) async fn download_and_install_windows(
    app: &AppHandle,
    state: &AppState,
    info: UpdateInfo,
) -> Result<(), String> {
    if info.download_url.is_empty() {
        return Err("下载 URL 为空".into());
    }

    // 1. 下载新 exe 到临时目录（复用通用 DownloadManager：分片/限速/失败重试 + 进度回调）
    let temp_dir = std::env::temp_dir().join("molaunch_update");
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {e}"))?;
    let new_exe = temp_dir.join("MoLaunch_new.exe");

    log::info!("[Updater] 开始下载新版本: {}", info.download_url);
    let downloaded = download_with_manager(state, &info.download_url, &new_exe, Some(app))
        .await
        .map_err(|e| format!("下载安装包失败: {e}"))?;
    log::info!(
        "[Updater] 下载完成: {} ({} bytes)",
        new_exe.display(),
        downloaded
    );

    // 2. 释放 updater.exe 到 AppData
    let updater_path =
        crate::resources::extract_updater().map_err(|e| format!("释放 updater.exe 失败: {e}"))?;

    // 3. 获取当前 exe 路径和 PID
    let current_exe = std::env::current_exe().map_err(|e| format!("获取当前 exe 路径失败: {e}"))?;
    let pid = std::process::id();

    // 进度已推送至 100%（前端据此切换 installing），停留片刻让用户感知"安装中"再退出
    std::thread::sleep(std::time::Duration::from_secs(1));

    log::info!(
        "[Updater] 启动 updater.exe: old={}, new={}, pid={}",
        current_exe.display(),
        new_exe.display(),
        pid
    );

    // 4. 启动 updater.exe 子进程（传递 --signature 供 updater 二次校验）
    std::process::Command::new(&updater_path)
        .arg("--old-exe")
        .arg(&current_exe)
        .arg("--new-exe")
        .arg(&new_exe)
        .arg("--pid")
        .arg(pid.to_string())
        .arg("--signature")
        .arg(&info.signature)
        .spawn()
        .map_err(|e| format!("启动 updater.exe 失败: {e}"))?;

    // 5. 退出主程序（updater 接管）
    app.exit(0);

    Ok(())
}

// 后台静默下载 + 退出时替换（Windows 便携版自动更新）
//
// 流程：
// 1. 前端定时（10 分钟）调用 check_update，发现新版本后调用 download_update_to_appdata
// 2. download_update_to_appdata 将新版本 exe 下载到 %APPDATA%/.Molaunch/last.exe
// 3. 用户点击右上角退出时，前端调用 apply_pending_update
// 4. apply_pending_update 检查 last.exe 是否存在，存在则启动 updater.exe 替换主 exe
// 5. 主程序退出，updater.exe 接管替换，下次启动即为新版本

/// 获取 last.exe 路径（%APPDATA%/.Molaunch/last.exe）
#[cfg(target_os = "windows")]
pub(super) fn last_exe_path() -> Result<std::path::PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA 环境变量未设置".to_string())?;
    Ok(std::path::PathBuf::from(appdata)
        .join(".Molaunch")
        .join("last.exe"))
}

/// 获取 last.sig 路径（%APPDATA%/.Molaunch/last.sig，与 last.exe 配对的签名缓存）
///
/// 后台预下载时随 last.exe 一起写入，退出时由 `apply_pending_update_impl`
/// 读出并作为 `--signature` 传给 updater.exe 做验签（updater 参数解析要求必填）。
#[cfg(target_os = "windows")]
pub(super) fn last_signature_path() -> Result<std::path::PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA 环境变量未设置".to_string())?;
    Ok(std::path::PathBuf::from(appdata)
        .join(".Molaunch")
        .join("last.sig"))
}

/// 后台静默下载新版本到 `%APPDATA%/.Molaunch/last.exe`
///
/// 前端定时检查发现新版本后调用此命令，将安装包下载到 appdata。
/// 下载完成后不立即替换，等用户退出程序时由 `apply_pending_update` 触发替换。
/// 每次定时检查命中都会重新下载覆盖 last.exe（无 size/hash 元数据，不做重复跳过）。
#[cfg(target_os = "windows")]
pub(super) async fn download_update_to_appdata_impl(
    state: &AppState,
    info: UpdateInfo,
) -> Result<bool, String> {
    if info.download_url.is_empty() {
        return Err("下载 URL 为空".into());
    }
    if info.signature.is_empty() {
        return Err("更新签名缺失，无法后台预下载".into());
    }

    let last_exe = last_exe_path()?;

    // 确保父目录存在
    if let Some(parent) = last_exe.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    log::info!(
        "[Updater] 后台下载新版本 {} 到: {}",
        info.version,
        last_exe.display()
    );

    let downloaded = download_with_manager(state, &info.download_url, &last_exe, None)
        .await
        .map_err(|e| format!("后台下载失败: {e}"))?;

    // 签名随 last.exe 一起缓存（last.sig），退出时 apply_pending_update 读出传给 updater.exe
    // 写失败时回滚 last.exe，保证 last.exe/last.sig 配对一致性
    let last_sig = last_signature_path()?;
    if let Err(e) = std::fs::write(&last_sig, info.signature.as_bytes()) {
        let _ = std::fs::remove_file(&last_exe);
        return Err(format!("写入 last.sig 失败: {e}"));
    }

    log::info!(
        "[Updater] 后台下载完成: {} ({} bytes)",
        last_exe.display(),
        downloaded
    );

    Ok(true)
}

/// 退出时检查并应用待安装更新
///
/// 检查 `%APPDATA%/.Molaunch/last.exe` 是否存在：
/// - 存在：释放 updater.exe，启动替换子进程，返回 true（调用方应随后退出主程序）
/// - 不存在：无待安装更新，返回 false（正常退出）
///
/// 前端在窗口 close 事件中调用此命令，返回 true 则让主程序退出由 updater.exe 接管。
#[cfg(target_os = "windows")]
pub(super) async fn apply_pending_update_impl(_app: &AppHandle) -> Result<bool, String> {
    let last_exe = last_exe_path()?;
    if !last_exe.exists() {
        log::info!("[Updater] 无待安装更新（last.exe 不存在），正常退出");
        return Ok(false);
    }

    // last.exe 必须与 last.sig 配对：签名缺失/为空说明预下载不完整（旧版本残留或下载中断），
    // 清理待安装文件返回 false，等下次定时检查重新下载
    let last_sig = last_signature_path()?;
    let signature = match std::fs::read_to_string(&last_sig) {
        Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => {
            log::warn!("[Updater] last.sig 缺失或为空，清理不完整的待安装更新");
            let _ = std::fs::remove_file(&last_exe);
            let _ = std::fs::remove_file(&last_sig);
            return Ok(false);
        }
    };

    // 释放 updater.exe
    let updater_path =
        crate::resources::extract_updater().map_err(|e| format!("释放 updater.exe 失败: {e}"))?;

    let current_exe = std::env::current_exe().map_err(|e| format!("获取当前 exe 路径失败: {e}"))?;
    let pid = std::process::id();

    log::info!(
        "[Updater] 退出时应用更新: old={}, new={}, pid={}",
        current_exe.display(),
        last_exe.display(),
        pid
    );

    std::process::Command::new(&updater_path)
        .arg("--old-exe")
        .arg(&current_exe)
        .arg("--new-exe")
        .arg(&last_exe)
        .arg("--pid")
        .arg(pid.to_string())
        .arg("--signature")
        .arg(&signature)
        .spawn()
        .map_err(|e| format!("启动 updater.exe 失败: {e}"))?;

    Ok(true)
}

/// 复用通用 DownloadManager 下载单个文件到本地路径
///
/// 返回实际下载字节数；`emit_progress` 非 None 时把 DownloadManager 的
/// GlobalProgress 转发为 `update-download-progress` 事件（供更新弹窗进度条使用）。
/// expected_size 传 0（未知）：单流路径会从响应头回填 total，分片路径由收尾事件兜底。
async fn download_with_manager(
    state: &AppState,
    url: &str,
    local_path: &std::path::Path,
    emit_progress: Option<&AppHandle>,
) -> Result<u64, String> {
    let task = DownloadTask {
        id: "update-exe".to_string(),
        urls: vec![url.to_string()],
        local_path: local_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };
    let progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>> =
        emit_progress.map(|app| {
            let app = app.clone();
            Arc::new(move |p: GlobalProgress| {
                let _ = app.emit(
                    PROGRESS_EVENT,
                    serde_json::json!({ "downloaded": p.downloaded_bytes, "total": p.total_bytes }),
                );
            }) as Arc<dyn Fn(GlobalProgress) + Send + Sync>
        });
    let manager = DownloadManager::from_config(&DownloadManagerConfig::from_state(state).await);
    let results = manager.download_batch(vec![task], progress_callback).await;
    let result = results.first().ok_or_else(|| "下载结果为空".to_string())?;
    if result.status == DownloadStatus::Failed {
        return Err(result
            .error
            .clone()
            .unwrap_or_else(|| "下载失败".to_string()));
    }
    // 收尾推送一次最终进度（total 未知时前端依赖它切换 installing 状态）
    if let Some(app) = emit_progress {
        let _ = app.emit(
            PROGRESS_EVENT,
            serde_json::json!({ "downloaded": result.downloaded, "total": result.downloaded }),
        );
    }
    Ok(result.downloaded)
}
