//! 离线登录命令
//!
//! 注：原 `#[tauri::command]` 标注已移除，函数改为接收 `&AppState`，
//! 由 `commands::auth::meta_manager` 统一 IPC 入口通过
//! `utils::meta_manager::dispatch` 分发调用。

use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth;
use crate::state::{AppState, LocalAuthResult};

/// 离线登录
pub async fn login_offline(
    state: &AppState,
    username: String,
) -> Result<LocalAuthResult, String> {
    log_info!("Offline login attempt for user: {}", username);

    if !auth::validate_username(&username) {
        return Err(
            "用户名长度需为 1-16 个字符，仅支持中文、字母、数字、下划线和连字符".to_string(),
        );
    }

    let result = auth::login_offline(&username);

    let auth_result = LocalAuthResult {
        name: result.name.clone(),
        uuid: result.uuid.clone(),
        access_token: result.access_token,
        client_token: result.client_token,
        login_type: "Legacy".to_string(),
        profile_json: None,
        server_url: None,
        server_name: None,
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
