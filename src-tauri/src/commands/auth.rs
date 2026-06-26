//! 认证命令

use crate::sdk::AuthResult;
use crate::state::AppState;
use tauri::State;

/// 离线登录
#[tauri::command]
pub async fn login_offline(
    state: State<'_, AppState>,
    username: String,
) -> Result<AuthResult, String> {
    log::info!("Offline login attempt for user: {}", username);

    // 验证用户名
    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }
    if username.len() > 16 {
        return Err("Username must be 16 characters or less".to_string());
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username can only contain letters, numbers, and underscores".to_string());
    }

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    let result = sdk.auth_offline(&username).map_err(|e| {
        log::error!("Offline login failed: {}", e);
        e.to_string()
    })?;

    // 保存认证状态
    let mut auth = state.auth.lock().await;
    auth.current_user = Some(result.clone());
    auth.is_logged_in = true;

    log::info!("Offline login successful for user: {}", username);
    Ok(result)
}

/// 获取当前登录状态
#[tauri::command]
pub async fn get_login_status(state: State<'_, AppState>) -> Result<Option<AuthResult>, String> {
    let auth = state.auth.lock().await;
    Ok(auth.current_user.clone())
}

/// 登出
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    let mut auth = state.auth.lock().await;
    auth.current_user = None;
    auth.is_logged_in = false;

    log::info!("User logged out");
    Ok(())
}
