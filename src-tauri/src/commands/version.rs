//! 版本管理命令

use crate::sdk::VersionList;
use crate::state::AppState;
use tauri::{Manager, State};

/// 下载进度事件
#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub percentage: f64,
}

/// 获取版本列表
#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionList, String> {
    log::info!("Fetching version list");

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    let versions = sdk.list_versions().map_err(|e| {
        log::error!("Failed to list versions: {}", e);
        e.to_string()
    })?;

    log::info!("Found {} versions", versions.versions.len());
    Ok(versions)
}

/// 下载版本（带进度回调）
#[tauri::command]
pub async fn download_version(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    log::info!("Downloading version: {}", version_id);

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    // 克隆 app handle 用于回调
    let app_handle = app.clone();

    sdk.download_version_with_callback(&version_id, move |stage, current, total| {
        let percentage = if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let progress = DownloadProgress {
            stage,
            current,
            total,
            percentage,
        };

        // 发送进度事件到前端
        let _ = app_handle.emit_all("download-progress", &progress);

        log::debug!(
            "Download progress: {}/{} ({:.1}%)",
            current,
            total,
            percentage
        );
    })
    .map_err(|e| {
        log::error!("Failed to download version: {}", e);
        e.to_string()
    })?;

    // 发送完成事件
    let _ = app.emit_all(
        "download-complete",
        serde_json::json!({ "version_id": version_id }),
    );

    log::info!("Version {} downloaded successfully", version_id);
    Ok(())
}

/// 获取已安装版本列表
#[tauri::command]
pub async fn list_installed_versions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    log::info!("Fetching installed versions");

    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    drop(config);

    log::info!("Game directory: {}", game_dir);

    let versions_dir = std::path::Path::new(&game_dir).join("versions");
    log::info!("Versions directory: {}", versions_dir.display());

    if !versions_dir.exists() {
        log::warn!("Versions directory does not exist");
        return Ok(Vec::new());
    }

    // 读取版本目录
    let mut versions = Vec::new();
    match std::fs::read_dir(&versions_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy().to_string();
                        versions.push(name_str);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to read versions directory: {}", e);
            return Err(format!("Failed to read: {}", e));
        }
    }

    log::info!("Found {} version directories: {:?}", versions.len(), versions);
    Ok(versions)
}

/// 卸载版本
#[tauri::command]
pub async fn uninstall_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    log::info!("Uninstalling version: '{}'", version_id);
    log::info!("Version ID length: {}, bytes: {:?}", version_id.len(), version_id.as_bytes());

    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    drop(config);

    log::info!("Game directory: {}", game_dir);

    // 构建版本目录路径
    let version_dir = std::path::Path::new(&game_dir)
        .join("versions")
        .join(&version_id);

    log::info!("Version directory: {}", version_dir.display());
    log::info!("Version directory exists: {}", version_dir.exists());

    // 列出versions目录下的所有子目录
    let versions_dir = std::path::Path::new(&game_dir).join("versions");
    if versions_dir.exists() {
        log::info!("Listing versions directory:");
        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    log::info!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        }
    }

    if version_dir.exists() {
        std::fs::remove_dir_all(&version_dir).map_err(|e| {
            log::error!("Failed to remove version directory: {}", e);
            format!("Failed to remove version: {}", e)
        })?;
        log::info!("Version {} uninstalled successfully", version_id);
        Ok(())
    } else {
        log::warn!("Version directory not found: {}", version_dir.display());
        Err(format!(
            "Version directory not found: {}",
            version_dir.display()
        ))
    }
}
