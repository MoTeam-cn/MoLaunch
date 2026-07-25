//! 会话命令：登录状态恢复 + 登出
//!
//! `get_login_status` 实现优先级：内存 → 磁盘恢复（含微软 token 静默刷新）。
//! 启动时前端首次调用以恢复登录态，若微软 token 过期会尝试 `login_with_refresh_token`
//! 静默续期，失败则回退到磁盘中的旧 token（让后续操作再触发交互式登录）。
//!
//! 注：原 `#[tauri::command]` 标注已移除，函数改为接收 `&AppState`，
//! 由 `commands::auth::meta_manager` 统一 IPC 入口通过
//! `utils::meta_manager::dispatch` 分发调用。

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::microsoft;
use crate::state::{AppState, LocalAuthResult};

/// 获取当前登录状态（优先内存，其次磁盘恢复）
pub async fn get_login_status(
    state: &AppState,
) -> Result<Option<LocalAuthResult>, String> {
    log_info!("[Startup][IPC] get_login_status called");
    {
        let auth = state.auth.lock().await;
        if auth.current_user.is_some() {
            log_info!("[Startup][IPC] get_login_status: returning in-memory user");
            return Ok(auth.current_user.clone());
        }
    }

    let persisted = state
        .auth_storage
        .load()
        .await
        .map_err(log_err("Failed to load auth storage"))?;

    if let Some(user) = persisted.current_user {
        if user.login_type == "Microsoft" {
            if let (Some(expires_at), Some(refresh_token)) = (user.expires_at, &user.refresh_token)
            {
                if microsoft::is_token_expired(expires_at) {
                    log_info!("Token expired on restore, attempting silent refresh...");
                    match microsoft::login_with_refresh_token(refresh_token, |_| {}).await {
                        Ok(r) => {
                            if let Err(e) = state
                                .auth_storage
                                .update_ms_token(
                                    &user.uuid,
                                    &r.access_token,
                                    &r.refresh_token,
                                    r.expires_at,
                                )
                                .await
                            {
                                log_warn!("Failed to update persisted token: {}", e);
                            }
                            let auth_result = LocalAuthResult {
                                name: r.username.clone(),
                                uuid: r.uuid.clone(),
                                access_token: r.access_token.clone(),
                                client_token: String::new(),
                                login_type: "Microsoft".to_string(),
                                profile_json: Some(r.profile_json.clone()),
                                server_url: None,
                                server_name: None,
                            };
                            let mut auth = state.auth.lock().await;
                            auth.current_user = Some(auth_result.clone());
                            auth.is_logged_in = true;
                            return Ok(Some(auth_result));
                        }
                        Err(e) => log_warn!("Silent refresh failed on restore: {}", e),
                    }
                }
            }
        }

        let auth_result = LocalAuthResult {
            name: user.name,
            uuid: user.uuid,
            access_token: user.access_token,
            client_token: user.client_token,
            login_type: user.login_type,
            profile_json: user.profile_json,
            server_url: user.server_url,
            server_name: user.server_name,
        };
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
        Ok(Some(auth_result))
    } else {
        Ok(None)
    }
}

/// 登出
pub async fn logout(state: &AppState) -> Result<(), String> {
    {
        let mut auth = state.auth.lock().await;
        auth.current_user = None;
        auth.is_logged_in = false;
    }
    if let Err(e) = state.auth_storage.clear_current_user().await {
        log_warn!("Failed to clear persisted auth: {}", e);
    }
    log_info!("User logged out");
    Ok(())
}
