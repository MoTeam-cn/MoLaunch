//! 微软账号管理命令（列表/删除/切换）
//! `switch_ms_account` 含 token 过期自动刷新：检测 `expires_at` 过期时调
//! `microsoft::login_with_refresh_token` 静默续期，刷新成功后回写 `auth_storage`。
//! 已聚合为 `meta_manager` IPC 入口，由 `meta_manager::dispatch` 分发调用。

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::microsoft;
use crate::state::{AppState, LocalAuthResult};

use super::MsAccountInfo;

/// 获取已存储的微软账号列表
///
/// token 过期时自动静默续期：检测 `expires_at` 过期后调 `login_with_refresh_token`
/// 刷新，成功则回写 `auth_storage` 并返回未过期状态；刷新失败（refresh_token 失效等）
/// 返回 `refreshing=true`，前端据此展示「续期中」而非「已过期」。
pub async fn get_ms_accounts(state: &AppState) -> Result<Vec<MsAccountInfo>, String> {
    log_info!("[Startup][IPC] get_ms_accounts called");
    let persisted = state
        .auth_storage
        .load()
        .await
        .map_err(log_err("Failed to load auth storage"))?;

    let mut result = Vec::with_capacity(persisted.ms_accounts.len());
    for a in &persisted.ms_accounts {
        let mut is_expired = microsoft::is_token_expired(a.expires_at);
        let mut refreshing = false;
        let mut expires_at = a.expires_at;

        if is_expired && !a.refresh_token.is_empty() {
            log_info!(
                "Token expired for MS account {}, refreshing...",
                a.username
            );
            match microsoft::login_with_refresh_token(&a.refresh_token, |_| {}).await {
                Ok(r) => {
                    if let Err(e) = state
                        .auth_storage
                        .update_ms_token(&a.uuid, &r.access_token, &r.refresh_token, r.expires_at)
                        .await
                    {
                        log_warn!("Failed to update persisted token: {}", e);
                    }
                    is_expired = false;
                    expires_at = r.expires_at;
                }
                Err(e) => {
                    log_warn!(
                        "Auto-refresh failed for MS account {}: {}",
                        a.username,
                        e
                    );
                    refreshing = true;
                }
            }
        }

        result.push(MsAccountInfo {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
            expires_at,
            is_expired,
            refreshing,
        });
    }

    Ok(result)
}

/// 删除已存储的微软账号
pub async fn remove_ms_account(state: &AppState, uuid: String) -> Result<(), String> {
    log_info!("Removing Microsoft account: {}", uuid);
    state
        .auth_storage
        .remove_ms_account(&uuid)
        .await
        .map_err(log_err("Failed to remove MS account"))
}

/// 切换到已存储的微软账号
pub async fn switch_ms_account(state: &AppState, uuid: String) -> Result<LocalAuthResult, String> {
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
        server_url: None,
        server_name: None,
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
            server_url: None,
            server_name: None,
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
