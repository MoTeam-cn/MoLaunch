//! authlib 账号管理：列表 / 删除 / 切换（含三步降级 + 密码重登兜底）

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::authlib::{
    login_with_cached_token, login_with_password, LoginOutcome, Profile,
};
use crate::minecraft::auth::storage::StoredAuthlibAccount;
use crate::state::{AppState, LocalAuthResult};

use super::types::AuthlibAccountInfo;

/// 切换到已保存的 authlib 账号（三步降级：validate → refresh → 用密码重登）
///
/// 调用方仅传 `server_url` + `uuid`，内部从持久化存储读取账号信息。
/// 任何一步成功即返回 `LocalAuthResult`，全部失败则返回错误。
pub async fn switch_authlib_account(
    state: &AppState,
    server_url: String,
    uuid: String,
) -> Result<LocalAuthResult, String> {
    log_info!(
        "[Authlib] Switching account: server={}, uuid={}",
        server_url,
        uuid
    );

    let account = state
        .auth_storage
        .get_authlib_account(&server_url, &uuid)
        .await
        .map_err(log_err("Failed to load authlib account"))?
        .ok_or_else(|| "账号不存在".to_string())?;

    let cached_profile = Profile {
        id: account.uuid.clone(),
        name: account.player_name.clone(),
    };

    // 三步降级
    let outcome = login_with_cached_token(
        &account.server_url,
        &account.access_token,
        &account.client_token,
        Some(&cached_profile),
    )
    .await;

    let (access_token, client_token) = match outcome {
        Ok(LoginOutcome::Success(resp)) => {
            // 验证或刷新成功，token 可能已更新
            let new_access = resp.access_token.clone();
            let new_client = resp.client_token.clone();
            // 如果 token 变化，更新持久化存储
            if new_access != account.access_token || new_client != account.client_token {
                if let Err(e) = state
                    .auth_storage
                    .update_authlib_token(&server_url, &uuid, &new_access, &new_client)
                    .await
                {
                    log_warn!("[Authlib] 更新 token 失败: {}", e);
                }
            }
            (new_access, new_client)
        }
        Ok(LoginOutcome::NeedSelect { .. }) => {
            // 已有缓存 profile 但服务器要求重选，理论不应发生，回退到密码重登
            log_warn!("[Authlib] validate/refresh 返回 NeedSelect，回退到密码登录");
            authlib_relogin_with_password(state, &account).await?
        }
        Err(e) if e.is_network => {
            return Err(format!("网络错误，无法切换账号: {}", e));
        }
        Err(_) => {
            // token 完全失效，用密码重新登录
            log_info!("[Authlib] token 完全失效，用密码重新登录");
            authlib_relogin_with_password(state, &account).await?
        }
    };

    // 更新当前用户
    let current = state
        .auth_storage
        .switch_authlib_account(&server_url, &uuid)
        .await
        .map_err(log_err("Failed to switch authlib account"))?
        .ok_or_else(|| "切换账号失败：账号不存在".to_string())?;

    // 用最新的 token 覆盖（switch_authlib_account 用的是持久化的 token，
    // 但我们刚刚可能通过密码重登拿到了新 token，需要覆盖）
    let user = LocalAuthResult {
        name: current.name,
        uuid: current.uuid,
        access_token,
        client_token,
        login_type: "AuthlibInjector".to_string(),
        profile_json: None,
        server_url: current.server_url,
        server_name: current.server_name,
    };

    {
        let mut auth_state = state.auth.lock().await;
        auth_state.current_user = Some(user.clone());
        auth_state.is_logged_in = true;
    }

    log_info!("[Authlib] Switched to account: {}", user.name);
    Ok(user)
}

/// 用账号密码重新登录（token 完全失效时的兜底）
///
/// 登录成功后更新持久化的 token，并返回新的 access_token + client_token。
async fn authlib_relogin_with_password(
    state: &AppState,
    account: &StoredAuthlibAccount,
) -> Result<(String, String), String> {
    let outcome = login_with_password(&account.server_url, &account.username, &account.password)
        .await
        .map_err(log_err("authlib password relogin failed"))?;

    match outcome {
        LoginOutcome::Success(resp) => {
            let new_access = resp.access_token.clone();
            let new_client = resp.client_token.clone();
            // 检查 selected_profile 是否与缓存一致，不一致则警告（不强制更新）
            if let Some(ref profile) = resp.selected_profile {
                if profile.id != account.uuid {
                    log_warn!(
                        "[Authlib] 重新登录后角色变化: old={}, new={}",
                        account.uuid,
                        profile.id
                    );
                }
            }
            // 更新持久化的 token
            if let Err(e) = state
                .auth_storage
                .update_authlib_token(&account.server_url, &account.uuid, &new_access, &new_client)
                .await
            {
                log_warn!("[Authlib] 更新 token 失败: {}", e);
            }
            Ok((new_access, new_client))
        }
        LoginOutcome::NeedSelect { .. } => {
            Err("账号密码登录后需要重新选择角色，请重新登录".to_string())
        }
    }
}

/// 获取已保存的 authlib 账号列表
pub async fn get_authlib_accounts(state: &AppState) -> Result<Vec<AuthlibAccountInfo>, String> {
    let persisted = state
        .auth_storage
        .load()
        .await
        .map_err(log_err("Failed to load auth storage"))?;
    Ok(persisted
        .authlib_accounts
        .iter()
        .map(|a| AuthlibAccountInfo {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
            player_name: a.player_name.clone(),
            server_url: a.server_url.clone(),
            server_name: a.server_name.clone(),
        })
        .collect())
}

/// 删除指定 authlib 账号
pub async fn remove_authlib_account(
    state: &AppState,
    server_url: String,
    uuid: String,
) -> Result<(), String> {
    log_info!(
        "[Authlib] Removing account: server={}, uuid={}",
        server_url,
        uuid
    );
    state
        .auth_storage
        .remove_authlib_account(&server_url, &uuid)
        .await
        .map_err(log_err("Failed to remove authlib account"))
}
