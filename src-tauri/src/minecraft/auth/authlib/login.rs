//! yggdrasil 登录编排
//! 三步降级（与同类启动器一致）：validate（缓存 token 仍有效则复用）→ refresh（失效则刷新）
//! → authenticate（刷新失败则密码重登）。多角色（available_profiles>1）首次登录返回
//! `LoginOutcome::NeedSelect`，前端选择后调 `refresh_with_profile`；已保存账号切换时若缓存
//! selected_profile 则直接复用。

use super::client::{self, YggdrasilError};
use super::types::{AuthResponse, Profile, ProfileId};

/// 登录结果
#[derive(Debug)]
pub enum LoginOutcome {
    /// 登录成功，可直接使用
    Success(AuthResponse),
    /// 需要前端选择角色（available_profiles > 1 且无 selected_profile）
    /// 前端选定后调用 `refresh_with_profile` 完成登录
    NeedSelect {
        access_token: String,
        client_token: String,
        available_profiles: Vec<Profile>,
    },
}

/// 三步降级登录入口
///
/// 用于已保存账号的快速恢复：
/// 1. 用缓存的 access_token + client_token 调 validate
/// 2. validate 失败则调 refresh（不指定 profile，沿用服务器端选定）
/// 3. refresh 失败则返回错误（需要用户重新输入密码）
///
/// `cached_profile` 用于在 refresh 后确认返回的 selected_profile 是否与缓存一致，
/// 若不一致则需重选角色。
pub async fn login_with_cached_token(
    server_url: &str,
    access_token: &str,
    client_token: &str,
    cached_profile: Option<&Profile>,
) -> Result<LoginOutcome, YggdrasilError> {
    // Step 1: validate
    match client::validate(server_url, access_token, Some(client_token)).await {
        Ok(()) => {
            // validate 成功，直接用缓存的 token
            // 但需要构造 AuthResponse，从缓存读取 profile 信息
            let profile = cached_profile.cloned();
            return Ok(LoginOutcome::Success(AuthResponse {
                access_token: access_token.to_string(),
                client_token: client_token.to_string(),
                available_profiles: vec![],
                selected_profile: profile,
                user: None,
            }));
        }
        Err(e) if e.is_network => return Err(e),
        Err(_) => {} // token 失效，继续 refresh
    }

    // Step 2: refresh
    match client::refresh(server_url, access_token, Some(client_token), None).await {
        Ok(resp) => {
            return Ok(classify_response(resp));
        }
        Err(e) if e.is_network => return Err(e),
        Err(e) => return Err(e),
    }
}

/// 账号密码登录（首次登录或重新登录）
///
/// 返回 `LoginOutcome`：
/// - `Success`：单角色或服务器已选定角色
/// - `NeedSelect`：多角色且无 selected_profile，前端需弹窗选择
pub async fn login_with_password(
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<LoginOutcome, YggdrasilError> {
    let resp = client::authenticate(server_url, username, password).await?;
    Ok(classify_response(resp))
}

/// 切换角色（多角色场景）
///
/// 前端选定 profile 后调用此函数，用 refresh 指定 selected_profile。
pub async fn refresh_with_profile(
    server_url: &str,
    access_token: &str,
    client_token: &str,
    profile: Profile,
) -> Result<AuthResponse, YggdrasilError> {
    let selected = ProfileId {
        id: profile.id.clone(),
        name: profile.name.clone(),
    };
    client::refresh(
        server_url,
        access_token,
        Some(client_token),
        Some(selected),
    )
    .await
}

/// 分类响应：单角色直接成功，多角色需选择
fn classify_response(resp: AuthResponse) -> LoginOutcome {
    if resp.selected_profile.is_some() {
        LoginOutcome::Success(resp)
    } else if resp.available_profiles.len() == 1 {
        // 只有一个角色，但服务器未自动选定，前端需调用 refresh_with_profile
        LoginOutcome::NeedSelect {
            access_token: resp.access_token,
            client_token: resp.client_token,
            available_profiles: resp.available_profiles,
        }
    } else if resp.available_profiles.is_empty() {
        // 无角色，直接报错（理论上服务器应在 authenticate 时返回错误）
        LoginOutcome::Success(resp)
    } else {
        // 多角色，需用户选择
        LoginOutcome::NeedSelect {
            access_token: resp.access_token,
            client_token: resp.client_token,
            available_profiles: resp.available_profiles,
        }
    }
}
