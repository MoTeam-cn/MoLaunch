//! SDK 管理命令（lite 版本）

use crate::error_util::log_err;
use crate::log_error;
use crate::log_info;
use crate::state::AppState;
use tauri::State;

/// SDK 状态信息
#[derive(serde::Serialize)]
pub struct SdkStatus {
    pub loaded: bool,
    pub version: Option<String>,
    pub platform: String,
    pub library_path: String,
}

/// 获取当前平台信息
#[tauri::command]
pub async fn get_platform_info() -> Result<SdkStatus, String> {
    log_info!("[Startup][IPC] get_platform_info called");
    let platform = if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macos-aarch64"
        } else {
            "macos-x86_64"
        }
    } else if cfg!(target_os = "linux") {
        "linux-x86_64"
    } else {
        "unknown"
    };

    let library_path = crate::sdk::get_sdk_library_path()
        .to_string_lossy()
        .to_string();

    Ok(SdkStatus {
        loaded: false,
        version: None,
        platform: platform.to_string(),
        library_path,
    })
}

/// 获取 SDK 版本
#[tauri::command]
pub async fn get_sdk_version(state: State<'_, AppState>) -> Result<Option<String>, String> {
    log_info!("[Startup][IPC] get_sdk_version called");
    let sdk_guard = state.sdk.lock().await;
    match sdk_guard.as_ref() {
        Some(sdk) => Ok(Some(sdk.version().map_err(log_err("Failed to get SDK version"))?)),
        None => Ok(None),
    }
}

/// 检查 SDK 是否已初始化
#[tauri::command]
pub async fn is_sdk_initialized(state: State<'_, AppState>) -> Result<bool, String> {
    let sdk_guard = state.sdk.lock().await;
    Ok(sdk_guard.is_some())
}

/// 获取设备 ID
#[tauri::command]
pub async fn get_device_id(state: State<'_, AppState>) -> Result<String, String> {
    log_info!("[Startup][IPC] get_device_id called");
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not loaded")?;

    sdk.get_device_id().map_err(|e| {
        log_error!("Failed to get device ID: {}", e);
        e.to_string()
    })
}

/// 检查更新（轻量版）
#[tauri::command]
pub async fn check_update_lite(
    state: State<'_, AppState>,
) -> Result<crate::sdk::UpdateInfoLite, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not loaded")?;

    sdk.update_check_lite().map_err(|e| {
        log_error!("Failed to check update: {}", e);
        e.to_string()
    })
}
