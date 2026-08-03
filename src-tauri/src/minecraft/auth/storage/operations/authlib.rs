//! authlib-injector 外置登录账号高层操作（独立 impl 块）

use super::super::super::authlib::LoginOutcome;
use super::super::types::{CurrentUser, StoredAuthlibAccount};
use super::super::AuthStorage;

impl AuthStorage {
    /// 保存 authlib 登录结果并设为当前用户
    ///
    /// 调用方需先通过 `authlib::login::*` 拿到 `LoginOutcome::Success`，
    /// 再调用此方法持久化。多角色场景下，前端选定 profile 后也走此方法。
    /// `password` 用于 token 失效后自动重新登录（明文存储，由注册表整体加密保护）。
    pub async fn save_authlib_login(
        &self,
        server_url: &str,
        server_name: &str,
        username: &str,
        password: &str,
        outcome: &LoginOutcome,
    ) -> Result<CurrentUser, String> {
        let (access_token, client_token, profile) = match outcome {
            LoginOutcome::Success(resp) => {
                let profile = resp.selected_profile.clone().ok_or_else(|| {
                    "authlib 登录响应缺少 selected_profile，请先调用 refresh_with_profile 选定角色"
                        .to_string()
                })?;
                (
                    resp.access_token.clone(),
                    resp.client_token.clone(),
                    profile,
                )
            }
            LoginOutcome::NeedSelect { .. } => {
                return Err("需要先选择角色才能保存登录".to_string());
            }
        };

        let mut state = self.load().await.unwrap_or_default();

        let account = StoredAuthlibAccount {
            username: username.to_string(),
            password: password.to_string(),
            access_token: access_token.clone(),
            client_token: client_token.clone(),
            uuid: profile.id.clone(),
            player_name: profile.name.clone(),
            server_url: server_url.to_string(),
            server_name: server_name.to_string(),
        };

        // 更新或添加到 authlib 账号列表（按 server_url + uuid 去重）
        if let Some(existing) = state
            .authlib_accounts
            .iter_mut()
            .find(|a| a.uuid == account.uuid && a.server_url == account.server_url)
        {
            *existing = account.clone();
        } else {
            state.authlib_accounts.push(account.clone());
        }

        let current = CurrentUser {
            name: profile.name.clone(),
            uuid: profile.id.clone(),
            access_token,
            client_token,
            login_type: "AuthlibInjector".to_string(),
            profile_json: None,
            refresh_token: None,
            expires_at: None,
            server_url: Some(server_url.to_string()),
            server_name: Some(server_name.to_string()),
        };
        state.current_user = Some(current.clone());

        self.save(&state).await?;
        Ok(current)
    }

    /// 删除指定 authlib 账号
    ///
    /// `server_url` + `uuid` 联合定位账号（同一 UUID 在不同服务器是不同账号）。
    pub async fn remove_authlib_account(&self, server_url: &str, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state
            .authlib_accounts
            .retain(|a| !(a.server_url == server_url && a.uuid == uuid));

        // 如果删除的是当前用户，也清除当前用户
        if let Some(ref current) = state.current_user {
            if current.login_type == "AuthlibInjector"
                && current.uuid == uuid
                && current.server_url.as_deref() == Some(server_url)
            {
                state.current_user = None;
            }
        }

        self.save(&state).await
    }

    /// 获取指定 authlib 账号（用于 token 失效后用账号密码重新登录）
    pub async fn get_authlib_account(
        &self,
        server_url: &str,
        uuid: &str,
    ) -> Result<Option<StoredAuthlibAccount>, String> {
        let state = self.load().await?;
        Ok(state
            .authlib_accounts
            .into_iter()
            .find(|a| a.server_url == server_url && a.uuid == uuid))
    }

    /// 更新 authlib 账号的 token（refresh 后调用）
    ///
    /// 同步更新账号列表和当前用户（如果是当前账号）。
    pub async fn update_authlib_token(
        &self,
        server_url: &str,
        uuid: &str,
        access_token: &str,
        client_token: &str,
    ) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        for account in state.authlib_accounts.iter_mut() {
            if account.server_url == server_url && account.uuid == uuid {
                account.access_token = access_token.to_string();
                account.client_token = client_token.to_string();
                break;
            }
        }

        if let Some(ref mut current) = state.current_user {
            if current.login_type == "AuthlibInjector"
                && current.uuid == uuid
                && current.server_url.as_deref() == Some(server_url)
            {
                current.access_token = access_token.to_string();
                current.client_token = client_token.to_string();
            }
        }

        self.save(&state).await
    }

    /// 切换到指定 authlib 账号（设为当前用户）
    ///
    /// 调用方负责先验证 token 有效性（`login_with_cached_token`），
    /// 验证通过后调用此方法更新 current_user。
    pub async fn switch_authlib_account(
        &self,
        server_url: &str,
        uuid: &str,
    ) -> Result<Option<CurrentUser>, String> {
        let mut state = self.load().await.unwrap_or_default();

        let account = state
            .authlib_accounts
            .iter()
            .find(|a| a.server_url == server_url && a.uuid == uuid)
            .cloned();

        let Some(account) = account else {
            return Ok(None);
        };

        state.current_user = Some(CurrentUser {
            name: account.player_name.clone(),
            uuid: account.uuid.clone(),
            access_token: account.access_token.clone(),
            client_token: account.client_token.clone(),
            login_type: "AuthlibInjector".to_string(),
            profile_json: None,
            refresh_token: None,
            expires_at: None,
            server_url: Some(account.server_url.clone()),
            server_name: Some(account.server_name.clone()),
        });

        let current = state.current_user.clone();
        self.save(&state).await?;
        Ok(current)
    }
}