//! Java 管理命令

use crate::sdk::JavaRuntime;
use crate::state::AppState;
use tauri::State;

/// 检测 Java
#[tauri::command]
pub async fn detect_java(state: State<'_, AppState>) -> Result<JavaRuntime, String> {
    log::info!("Detecting Java...");

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    let java = sdk.detect_java().map_err(|e| {
        log::error!("Failed to detect Java: {}", e);
        e.to_string()
    })?;

    log::info!("Java detected: {} ({})", java.version, java.executable);
    Ok(java)
}

/// 列出所有 Java
#[tauri::command]
pub async fn list_java(state: State<'_, AppState>) -> Result<Vec<JavaRuntime>, String> {
    log::info!("Listing all Java runtimes...");

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    let java_list = sdk.list_java().map_err(|e| {
        log::error!("Failed to list Java: {}", e);
        e.to_string()
    })?;

    log::info!("Found {} Java runtimes", java_list.len());
    Ok(java_list)
}
