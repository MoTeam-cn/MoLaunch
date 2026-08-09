//! SDK 管理命令（lite 版本）
//! `ActionRequest` 与 `meta_manager` / `tools_manager` / `image_cache_manager` 共用同一请求体结构。

mod manager;

use crate::error_util::log_err;
use crate::log_error;
use crate::log_info;
use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// SDK 状态信息
#[derive(serde::Serialize)]
pub struct SdkStatus {
    pub loaded: bool,
    pub version: Option<String>,
    pub platform: String,
    pub library_path: String,
}

/// 统一 SDK IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn sdk_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}

/// 获取当前平台信息
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
pub async fn get_sdk_version(state: &AppState) -> Result<Option<String>, String> {
    log_info!("[Startup][IPC] get_sdk_version called");
    let sdk_guard = state.sdk.lock().await;
    match sdk_guard.as_ref() {
        Some(sdk) => Ok(Some(
            sdk.version()
                .map_err(log_err("Failed to get SDK version"))?,
        )),
        None => Ok(None),
    }
}

/// 检查 SDK 是否已初始化
pub async fn is_sdk_initialized(state: &AppState) -> Result<bool, String> {
    let sdk_guard = state.sdk.lock().await;
    Ok(sdk_guard.is_some())
}

/// 获取设备 ID
pub async fn get_device_id(state: &AppState) -> Result<String, String> {
    log_info!("[Startup][IPC] get_device_id called");
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not loaded")?;

    sdk.get_device_id().map_err(|e| {
        log_error!("Failed to get device ID: {}", e);
        e.to_string()
    })
}
