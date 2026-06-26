//! 版本管理命令

use crate::sdk::VersionList;
use crate::state::AppState;
use tauri::{Manager, State};

/// 下载进度事件
#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub stage: u32,
    pub stage_name: String,
    pub current: u32,
    pub total: u32,
    pub percentage: f64,
}

/// 获取阶段名称
fn get_stage_name(stage: u32) -> &'static str {
    match stage {
        0 => "版本清单",
        1 => "版本 JSON",
        2 => "客户端 JAR",
        3 => "库文件",
        4 => "资源文件",
        5 => "解压 Natives",
        _ => "未知",
    }
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
            stage_name: get_stage_name(stage).to_string(),
            current,
            total,
            percentage,
        };

        // 发送进度事件到前端
        let _ = app_handle.emit_all("download-progress", &progress);

        log::debug!(
            "Download progress [{}]: {}/{} ({:.1}%)",
            get_stage_name(stage),
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

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    let versions = sdk.list_installed_versions().map_err(|e| {
        log::error!("Failed to list installed versions: {}", e);
        e.to_string()
    })?;

    log::info!("Found {} installed versions", versions.len());
    Ok(versions)
}
