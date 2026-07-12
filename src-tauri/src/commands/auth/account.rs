//! 账号管理命令（列表/删除/切换/登出/状态恢复）

use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::microsoft;
use crate::state::{AppState, LocalAuthResult};
use serde::Serialize;
use tauri::State;

/// 已存储的微软账号信息
#[derive(Debug, Clone, Serialize)]
pub struct MsAccountInfo {
    pub username: String,
    pub uuid: String,
    pub expires_at: u64,
    pub is_expired: bool,
}

/// 获取已存储的微软账号列表
#[tauri::command]
pub async fn get_ms_accounts(state: State<'_, AppState>) -> Result<Vec<MsAccountInfo>, String> {
    let persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;
    Ok(persisted
        .ms_accounts
        .iter()
        .map(|a| MsAccountInfo {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
            expires_at: a.expires_at,
            is_expired: microsoft::is_token_expired(a.expires_at),
        })
        .collect())
}

/// 删除已存储的微软账号
#[tauri::command]
pub async fn remove_ms_account(state: State<'_, AppState>, uuid: String) -> Result<(), String> {
    log_info!("Removing Microsoft account: {}", uuid);
    state
        .auth_storage
        .remove_ms_account(&uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 切换到已存储的微软账号
#[tauri::command]
pub async fn switch_ms_account(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<LocalAuthResult, String> {
    log_info!("Switching to Microsoft account: {}", uuid);

    let account = state
        .auth_storage
        .get_ms_account(&uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Account not found".to_string())?;

    let (access_token, refresh_token, expires_at) =
        if microsoft::is_token_expired(account.expires_at) {
            log_info!("Token expired, refreshing...");
            let r = microsoft::login_with_refresh_token(&account.refresh_token, |_| {})
                .await
                .map_err(|e| e.to_string())?;
            if let Err(e) = state
                .auth_storage
                .update_ms_token(&uuid, &r.access_token, &r.refresh_token, r.expires_at)
                .await
            {
                log_warn!("Failed to update persisted token: {}", e);
            }
            (r.access_token, r.refresh_token, r.expires_at)
        } else {
            (account.access_token.clone(), account.refresh_token, account.expires_at)
        };

    let auth_result = LocalAuthResult {
        name: account.username.clone(),
        uuid: account.uuid.clone(),
        access_token: access_token.clone(),
        client_token: String::new(),
        login_type: "Microsoft".to_string(),
        profile_json: Some(account.profile_json.clone()),
    };

    // 更新当前用户（持久化）
    {
        let mut persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;
        persisted.current_user = Some(crate::minecraft::auth::storage::CurrentUser {
            name: account.username.clone(),
            uuid: account.uuid.clone(),
            access_token,
            client_token: String::new(),
            login_type: "Microsoft".to_string(),
            profile_json: Some(account.profile_json.clone()),
            refresh_token: Some(refresh_token),
            expires_at: Some(expires_at),
        });
        state
            .auth_storage
            .save(&persisted)
            .await
            .map_err(|e| e.to_string())?;
    }

    {
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
    }

    log_info!("Switched to Microsoft account: {}", account.username);
    Ok(auth_result)
}

/// 获取当前登录状态（优先内存，其次磁盘恢复）
#[tauri::command]
pub async fn get_login_status(state: State<'_, AppState>) -> Result<Option<LocalAuthResult>, String> {
    {
        let auth = state.auth.lock().await;
        if auth.current_user.is_some() {
            return Ok(auth.current_user.clone());
        }
    }

    let persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;

    if let Some(user) = persisted.current_user {
        if user.login_type == "Microsoft" {
            if let (Some(expires_at), Some(refresh_token)) = (user.expires_at, &user.refresh_token) {
                if microsoft::is_token_expired(expires_at) {
                    log_info!("Token expired on restore, attempting silent refresh...");
                    match microsoft::login_with_refresh_token(refresh_token, |_| {}).await {
                        Ok(r) => {
                            if let Err(e) = state
                                .auth_storage
                                .update_ms_token(&user.uuid, &r.access_token, &r.refresh_token, r.expires_at)
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
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
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
