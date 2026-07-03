//! 认证命令

use crate::minecraft::auth;
use crate::state::{AppState, LocalAuthResult};
use tauri::State;

/// 离线登录
#[tauri::command]
pub async fn login_offline(
    state: State<'_, AppState>,
    username: String,
) -> Result<LocalAuthResult, String> {
    log::info!("Offline login attempt for user: {}", username);

    // 验证用户名
    if !auth::validate_username(&username) {
        return Err("Username must be 3-16 characters and contain only letters, numbers, and underscores".to_string());
    }

    // 使用本地实现进行离线登录
    let result = auth::login_offline(&username);
    
    // 转换为本地认证结果
    let auth_result = LocalAuthResult {
        name: result.name,
        uuid: result.uuid,
        access_token: result.access_token,
        client_token: result.client_token,
        login_type: "Legacy".to_string(),
        profile_json: None,
    };

    // 保存认证状态
    let mut auth = state.auth.lock().await;
    auth.current_user = Some(auth_result.clone());
    auth.is_logged_in = true;

    log::info!("Offline login successful for user: {}", username);
    Ok(auth_result)
}

/// 获取当前登录状态
#[tauri::command]
pub async fn get_login_status(state: State<'_, AppState>) -> Result<Option<LocalAuthResult>, String> {
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
