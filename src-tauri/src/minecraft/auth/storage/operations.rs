//! AuthStorage 高层操作（独立 impl 块）
//!
//! 这些方法仅依赖 `self.load()` 和 `self.save()`，与注册表读写细节解耦。
//! Rust 允许同一结构体在多个文件中分散 impl 块。

use super::super::authlib::LoginOutcome;
use super::super::microsoft::MicrosoftLoginResult;
use super::types::{CurrentUser, StoredAuthlibAccount, StoredMsAccount, StoredOfflineAccount};
use super::AuthStorage;

impl AuthStorage {
    /// 保存微软登录结果并设为当前用户
    pub async fn save_ms_login(&self, result: &MicrosoftLoginResult) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 更新或添加到微软账号列表
        let account = StoredMsAccount::from(result);
        if let Some(existing) = state
            .ms_accounts
            .iter_mut()
            .find(|a| a.uuid == account.uuid)
        {
            *existing = account.clone();
        } else {
            state.ms_accounts.push(account.clone());
        }

        // 设为当前用户
        state.current_user = Some(CurrentUser {
            name: result.username.clone(),
            uuid: result.uuid.clone(),
            access_token: result.access_token.clone(),
            client_token: String::new(),
            login_type: "Microsoft".to_string(),
            profile_json: Some(result.profile_json.clone()),
            refresh_token: Some(result.refresh_token.clone()),
            expires_at: Some(result.expires_at),
            server_url: None,
            server_name: None,
        });

        self.save(&state).await
    }

    /// 保存离线登录并设为当前用户
    ///
    /// 同时把账号添加到离线账号列表（UUID 去重），保留已有的 skin 选择。
    pub async fn save_offline_login(&self, username: &str, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 添加到离线账号列表（UUID 去重，保留已有的 skin 选择）
        let account = StoredOfflineAccount {
            username: username.to_string(),
            uuid: uuid.to_string(),
            skin: state
                .offline_accounts
                .iter()
                .find(|a| a.uuid == uuid)
                .and_then(|a| a.skin.clone()),
        };
        if let Some(existing) = state
            .offline_accounts
            .iter_mut()
            .find(|a| a.uuid == account.uuid)
        {
            *existing = account.clone();
        } else {
            state.offline_accounts.push(account);
        }

        state.current_user = Some(CurrentUser {
            name: username.to_string(),
            uuid: uuid.to_string(),
            access_token: uuid.to_string(),
            client_token: uuid.to_string(),
            login_type: "Legacy".to_string(),
            profile_json: None,
            refresh_token: None,
            expires_at: None,
            server_url: None,
            server_name: None,
        });

        self.save(&state).await
    }

    /// 设置离线账号的皮肤选择
    pub async fn set_offline_skin(&self, uuid: &str, skin: Option<&str>) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        if let Some(account) = state.offline_accounts.iter_mut().find(|a| a.uuid == uuid) {
            account.skin = skin.map(|s| s.to_string());
            self.save(&state).await
        } else {
            Err("离线账号不存在".to_string())
        }
    }

    /// 删除指定离线账号
    pub async fn remove_offline_account(&self, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.offline_accounts.retain(|a| a.uuid != uuid);

        // 如果删除的是当前用户，也清除当前用户
        if let Some(ref current) = state.current_user {
            if current.uuid == uuid {
                state.current_user = None;
            }
        }

        self.save(&state).await
    }

    /// 获取指定离线账号
    pub async fn get_offline_account(
        &self,
        uuid: &str,
    ) -> Result<Option<StoredOfflineAccount>, String> {
        let state = self.load().await?;
        Ok(state.offline_accounts.into_iter().find(|a| a.uuid == uuid))
    }

    /// 清除当前用户（登出）
    pub async fn clear_current_user(&self) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.current_user = None;
        self.save(&state).await
    }

    /// 删除指定微软账号
    pub async fn remove_ms_account(&self, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.ms_accounts.retain(|a| a.uuid != uuid);

        // 如果删除的是当前用户，也清除当前用户
        if let Some(ref current) = state.current_user {
            if current.uuid == uuid {
                state.current_user = None;
            }
        }

        self.save(&state).await
    }

    /// 获取指定微软账号（用于刷新）
    pub async fn get_ms_account(&self, uuid: &str) -> Result<Option<StoredMsAccount>, String> {
        let state = self.load().await?;
        Ok(state.ms_accounts.into_iter().find(|a| a.uuid == uuid))
    }

    /// 获取当前微软账号的 refresh_token（用于静默刷新）
    pub async fn get_current_refresh_token(&self) -> Result<Option<String>, String> {
        let state = self.load().await?;
        match state.current_user {
            Some(ref user) if user.login_type == "Microsoft" => Ok(user.refresh_token.clone()),
            _ => Ok(None),
        }
    }

    /// 更新微软账号的 Token（刷新后调用）
    pub async fn update_ms_token(
        &self,
        uuid: &str,
        access_token: &str,
        refresh_token: &str,
        expires_at: u64,
    ) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 更新账号列表
        for account in state.ms_accounts.iter_mut() {
            if account.uuid == uuid {
                account.access_token = access_token.to_string();
                account.refresh_token = refresh_token.to_string();
                account.expires_at = expires_at;
                break;
            }
        }

        // 更新当前用户
        if let Some(ref mut current) = state.current_user {
            if current.uuid == uuid {
                current.access_token = access_token.to_string();
                current.refresh_token = Some(refresh_token.to_string());
                current.expires_at = Some(expires_at);
            }
        }

        self.save(&state).await
    }

    // ============================================================
    // authlib-injector 外置登录
    // ============================================================

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
                (resp.access_token.clone(), resp.client_token.clone(), profile)
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
    pub async fn remove_authlib_account(
        &self,
        server_url: &str,
        uuid: &str,
    ) -> Result<(), String> {
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
