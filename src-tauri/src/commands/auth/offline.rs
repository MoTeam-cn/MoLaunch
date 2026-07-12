//! 离线登录命令

use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth;
use crate::state::{AppState, LocalAuthResult};
use tauri::State;

/// 离线登录
#[tauri::command]
pub async fn login_offline(
    state: State<'_, AppState>,
    username: String,
) -> Result<LocalAuthResult, String> {
    log_info!("Offline login attempt for user: {}", username);

    if !auth::validate_username(&username) {
        return Err("用户名长度需为 1-16 个字符，仅支持中文、字母、数字、下划线和连字符".to_string());
    }

    let result = auth::login_offline(&username);

    let auth_result = LocalAuthResult {
        name: result.name.clone(),
        uuid: result.uuid.clone(),
        access_token: result.access_token,
        client_token: result.client_token,
        login_type: "Legacy".to_string(),
        profile_json: None,
    };

    {
        let mut auth_state = state.auth.lock().await;
        auth_state.current_user = Some(auth_result.clone());
        auth_state.is_logged_in = true;
    }

    if let Err(e) = state
        .auth_storage
        .save_offline_login(&username, &result.uuid)
        .await
    {
        log_warn!("Failed to persist offline login: {}", e);
    }

    log_info!("Offline login successful for user: {}", username);
    Ok(auth_result)
}
