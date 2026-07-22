//! 微软账号管理命令（列表 / 删除 / 切换）
//!
//! `switch_ms_account` 含 token 过期自动刷新逻辑：检测到 `expires_at` 过期时调用
//! `microsoft::login_with_refresh_token` 静默续期，刷新成功后回写 `auth_storage`。

use tauri::State;

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::microsoft;
use crate::state::{AppState, LocalAuthResult};

use super::MsAccountInfo;

/// 获取已存储的微软账号列表
#[tauri::command]
pub async fn get_ms_accounts(state: State<'_, AppState>) -> Result<Vec<MsAccountInfo>, String> {
    log_info!("[Startup][IPC] get_ms_accounts called");
    let persisted = state
        .auth_storage
        .load()
        .await
        .map_err(log_err("Failed to load auth storage"))?;
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
        .map_err(log_err("Failed to remove MS account"))
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
        .map_err(log_err("Failed to get MS account"))?
        .ok_or_else(|| "Account not found".to_string())?;

    let (access_token, refresh_token, expires_at) =
        if microsoft::is_token_expired(account.expires_at) {
            log_info!("Token expired, refreshing...");
            let r = microsoft::login_with_refresh_token(&account.refresh_token, |_| {})
                .await
                .map_err(log_err("Failed to refresh MS token"))?;
            if let Err(e) = state
                .auth_storage
                .update_ms_token(&uuid, &r.access_token, &r.refresh_token, r.expires_at)
                .await
            {
                log_warn!("Failed to update persisted token: {}", e);
            }
            (r.access_token, r.refresh_token, r.expires_at)
        } else {
            (
                account.access_token.clone(),
                account.refresh_token,
                account.expires_at,
            )
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
        let mut persisted = state
            .auth_storage
            .load()
            .await
            .map_err(log_err("Failed to load auth storage"))?;
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
            .map_err(log_err("Failed to save auth storage"))?;
    }

    {
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
    }

    log_info!("Switched to Microsoft account: {}", account.username);
    Ok(auth_result)
}
