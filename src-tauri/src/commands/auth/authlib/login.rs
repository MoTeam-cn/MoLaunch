//! authlib 登录流程：服务器元数据拉取 / 账号密码登录 / 多角色选角

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::authlib::{
    fetch_server_metadata, login_with_password, refresh_with_profile, LoginOutcome, Profile,
};
use crate::state::{AppState, LocalAuthResult};

use super::types::{AuthlibLoginResult, AuthlibServerMeta, PendingAuthlibLogin};

/// 获取 yggdrasil 服务器元数据
///
/// 前端登录页输入 server_url 后调用，用于显示服务器名/注册链接。
/// 失败时返回错误（前端提示用户检查地址或网络）。
pub async fn authlib_fetch_server_meta(server_url: String) -> Result<AuthlibServerMeta, String> {
    log_info!("[Authlib] Fetching server metadata: {}", server_url);
    let meta = fetch_server_metadata(&server_url)
        .await
        .map_err(|e| e.to_string())?;
    Ok(AuthlibServerMeta::from(meta))
}

/// 账号密码登录
///
/// 流程：
/// 1. 调用 `login_with_password` 走 yggdrasil `/authserver/authenticate`
/// 2. 单角色或服务器已选定 → 返回 `Success`，前端拿到 `LocalAuthResult` 直接登录
/// 3. 多角色 → 返回 `NeedSelect`，前端弹窗让用户选择，再调用 `authlib_select_profile`
///
/// `password` 会随账号一起持久化（加密），用于 token 失效后自动重新登录。
pub async fn authlib_login(
    state: &AppState,
    server_url: String,
    username: String,
    password: String,
) -> Result<AuthlibLoginResult, String> {
    log_info!(
        "[Authlib] Login attempt: server={}, user={}",
        server_url,
        username
    );

    // 拉取服务器元数据（用于服务器显示名）
    let server_name = match fetch_server_metadata(&server_url).await {
        Ok(meta) => meta.server_name(),
        Err(e) => {
            log_warn!("[Authlib] 获取服务器元数据失败，使用占位名: {}", e);
            "未知服务器".to_string()
        }
    };

    let outcome = login_with_password(&server_url, &username, &password)
        .await
        .map_err(log_err("authlib login failed"))?;

    match outcome {
        LoginOutcome::Success(_) => {
            // 单角色，直接持久化
            let current = state
                .auth_storage
                .save_authlib_login(&server_url, &server_name, &username, &password, &outcome)
                .await
                .map_err(log_err("Failed to persist authlib login"))?;

            let user = LocalAuthResult {
                name: current.name,
                uuid: current.uuid,
                access_token: current.access_token,
                client_token: current.client_token,
                login_type: "AuthlibInjector".to_string(),
                profile_json: None,
                server_url: current.server_url,
                server_name: current.server_name,
            };

            // 同步内存状态
            {
                let mut auth_state = state.auth.lock().await;
                auth_state.current_user = Some(user.clone());
                auth_state.is_logged_in = true;
            }

            log_info!("[Authlib] Login success: {}", user.name);
            Ok(AuthlibLoginResult::Success { user })
        }
        LoginOutcome::NeedSelect {
            access_token,
            client_token,
            available_profiles,
        } => {
            // 多角色：不持久化，等前端选定后调用 authlib_select_profile
            // 但需要把 username/password/server_url 暂存到内存，select_profile 时取出
            log_info!(
                "[Authlib] Multi-profile detected, need select: count={}",
                available_profiles.len()
            );
            let mut pending = state.authlib_pending.lock().await;
            *pending = Some(PendingAuthlibLogin {
                server_url: server_url.clone(),
                server_name: server_name.clone(),
                username: username.clone(),
                password: password.clone(),
                access_token: access_token.clone(),
                client_token: client_token.clone(),
            });
            Ok(AuthlibLoginResult::NeedSelect {
                access_token,
                client_token,
                available_profiles,
            })
        }
    }
}

/// 多角色场景下选定 profile 完成登录
///
/// 前端拿到 `NeedSelect` 后弹窗让用户选择 profile，选定后调用此命令。
/// 内部调用 yggdrasil `/authserver/refresh` 指定 selected_profile，
/// 成功后持久化账号并设为当前用户。
pub async fn authlib_select_profile(
    state: &AppState,
    profile: Profile,
) -> Result<LocalAuthResult, String> {
    log_info!(
        "[Authlib] Selecting profile: id={}, name={}",
        profile.id,
        profile.name
    );

    let pending = {
        let mut pending_lock = state.authlib_pending.lock().await;
        pending_lock
            .take()
            .ok_or_else(|| "没有待处理的 authlib 登录，请重新登录".to_string())?
    };

    let resp = refresh_with_profile(
        &pending.server_url,
        &pending.access_token,
        &pending.client_token,
        profile.clone(),
    )
    .await
    .map_err(log_err("authlib select_profile failed"))?;

    // 构造 LoginOutcome::Success 让 save_authlib_login 处理持久化
    let outcome = LoginOutcome::Success(resp);
    let current = state
        .auth_storage
        .save_authlib_login(
            &pending.server_url,
            &pending.server_name,
            &pending.username,
            &pending.password,
            &outcome,
        )
        .await
        .map_err(log_err("Failed to persist authlib login after select"))?;

    let user = LocalAuthResult {
        name: current.name,
        uuid: current.uuid,
        access_token: current.access_token,
        client_token: current.client_token,
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

    log_info!("[Authlib] Profile selected, login success: {}", user.name);
    Ok(user)
}
